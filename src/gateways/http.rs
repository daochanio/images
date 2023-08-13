use std::{sync::Arc, time::Duration};

use crate::{settings::Settings, usecases::gateways::Web};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use reqwest::{
    redirect::{self},
    Response,
};
use serde::Deserialize;

const MAX_BODY_SIZE: usize = 3 * 1024 * 1024;
const MAX_REQUEST_DURATION_SECONDS: u64 = 30;

struct Http {
    settings: Arc<Settings>,
    client: reqwest::Client,
}

pub fn new(settings: Arc<Settings>) -> impl Web {
    Http {
        settings,
        client: reqwest::Client::builder()
            .redirect(redirect::Policy::none())
            .timeout(Duration::from_secs(MAX_REQUEST_DURATION_SECONDS))
            .build()
            .unwrap(),
    }
}

#[derive(Debug, Deserialize)]
struct NFTMetadata {
    image: Option<String>,
    image_url: Option<String>,
    image_data: Option<String>,
}

#[async_trait]
impl Web for Http {
    async fn get_nft_image_url(&self, url: String) -> Result<String> {
        tracing::info!("requesting nft metadata from {}", url);

        let response = self
            .get_with_status_check(url)
            .await
            .context("could not get nft metadata")?;

        let body = self
            .read_body_with_limit(response, MAX_BODY_SIZE)
            .await
            .context("could not read body of nft metadata")?;

        let metadata = serde_json::from_slice::<NFTMetadata>(&body)
            .context("could not deserialize nft metadata as json")?;

        if let Some(image) = metadata.image {
            return Ok(image);
        } else if let Some(image_url) = metadata.image_url {
            return Ok(image_url);
        } else if let Some(image_data) = metadata.image_data {
            return Ok(image_data);
        }

        bail!("could not get nft image uri");
    }

    async fn get_image_data(&self, url: String) -> Result<Vec<u8>> {
        tracing::info!("requesting image from {}", url);

        let response = self
            .get_with_status_check(url)
            .await
            .context("could not get image")?;

        let body = self
            .read_body_with_limit(response, MAX_BODY_SIZE)
            .await
            .context("could not read image body")?;

        return Ok(body.to_vec());
    }
}

impl Http {
    async fn get_with_status_check(&self, url: String) -> Result<Response> {
        let url = self.parse_url(url);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("could not get {}", url))?;

        if !resp.status().is_success() {
            bail!("invalid status for get {}: {}", url, resp.status());
        }

        Ok(resp)
    }

    async fn read_body_with_limit(&self, mut resp: Response, limit: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        while let Some(chunk) = resp.chunk().await.context("could not read chunk")? {
            if buf.len() + chunk.len() > limit {
                bail!("response body too large {}", buf.len() + chunk.len());
            }

            buf.extend_from_slice(&chunk);
        }

        Ok(buf)
    }

    fn parse_url(&self, url: String) -> String {
        if let Some(suffix) = url.strip_prefix("ipfs://") {
            let mut suffix = suffix.to_string();

            if !suffix.starts_with("ipfs/") {
                suffix = format!("ipfs/{}", suffix);
            }

            return format!("{}/{}", self.settings.ipfs_gateway_url(), suffix);
        }

        if let Some(suffix) = url.strip_prefix("ipns://") {
            let mut suffix = suffix.to_string();

            if !suffix.starts_with("ipns/") {
                suffix = format!("ipns/{}", suffix);
            }

            return format!("{}/{}", self.settings.ipfs_gateway_url(), suffix);
        }

        url
    }
}
