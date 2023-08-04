use std::time::Duration;

use crate::usecases::gateways::Web;
use async_trait::async_trait;
use reqwest::{
    redirect::{self},
    Response,
};
use serde::Deserialize;

const MAX_BODY_SIZE: usize = 3 * 1024 * 1024;
const MAX_REQUEST_DURATION_SECONDS: u64 = 30;

struct Http {
    client: reqwest::Client,
}

pub fn new() -> impl Web {
    Http {
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
    async fn get_nft_image_url(&self, url: String) -> Result<String, String> {
        tracing::info!("requesting nft metadata from {}", url);

        let body = match self.get_with_status_check(url).await {
            Ok(r) => match self.read_body_with_limit(r, MAX_BODY_SIZE).await {
                Ok(b) => b,
                Err(e) => return Err(format!("could not read nft metadata: {}", e)),
            },
            Err(e) => return Err(format!("could not get nft metadata: {}", e)),
        };

        let metadata = match serde_json::from_slice::<NFTMetadata>(&body) {
            Ok(j) => j,
            Err(e) => return Err(format!("could not parse nft metadata as json: {}", e)),
        };

        // let metadata = match self.client.get(&url).send().await {
        //     Ok(r) => match r.json::<NFTMetadata>().await {
        //         Ok(j) => j,
        //         Err(e) => return Err(format!("could not parse nft metada as json: {}", e)),
        //     },
        //     Err(e) => return Err(format!("could not get nft metadata: {}", e)),
        // };

        if let Some(image) = metadata.image {
            return Ok(image);
        } else if let Some(image_url) = metadata.image_url {
            return Ok(image_url);
        } else if let Some(image_data) = metadata.image_data {
            return Ok(image_data);
        }

        return Err(String::from("could not get nft image uri"));
    }

    async fn get_image_data(&self, url: String) -> Result<Vec<u8>, String> {
        tracing::info!("requesting image from {}", url);

        let body = match self.get_with_status_check(url).await {
            Ok(r) => match self.read_body_with_limit(r, MAX_BODY_SIZE).await {
                Ok(b) => b,
                Err(e) => return Err(format!("could not get image bytes: {}", e)),
            },
            Err(e) => return Err(format!("could not get image: {}", e)),
        };

        return Ok(body.to_vec());
    }
}

impl Http {
    async fn get_with_status_check(&self, url: String) -> Result<Response, String> {
        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => return Err(format!("could not get {}: {}", url, e)),
        };

        if !resp.status().is_success() {
            return Err(format!("invalid status for get {}: {}", url, resp.status()));
        }

        Ok(resp)
    }

    async fn read_body_with_limit(
        &self,
        mut resp: Response,
        limit: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buf = Vec::new();

        while let Some(chunk) = resp.chunk().await? {
            if buf.len() + chunk.len() > limit {
                return Err(format!("response body too large {}", buf.len() + chunk.len()).into());
            }
            buf.extend_from_slice(&chunk);
        }

        return Ok(buf);
    }
}
