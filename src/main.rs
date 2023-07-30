use aws_sdk_s3::{config::{Credentials, Region}, primitives::ByteStream};
use axum::{
    routing::{post, get},
    http::{StatusCode, Request},
    Json,
    Router,
    body::{Bytes},
    response::{Response, IntoResponse},
    middleware::{self, Next},
    extract::DefaultBodyLimit,
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::Serialize;
use tokio::signal;
use uuid::Uuid;
use std::{net::SocketAddr, io::Cursor};
use dotenv::dotenv;
use std::env;

const MAX_IMAGE_SIZE: usize = 5*1024*1024;

#[tokio::main]
async fn main() {
    dotenv().ok();

    if env::var("ENV").unwrap() == "dev" {
        tracing_subscriber::fmt()
            .compact()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_max_level(tracing::Level::INFO)
            .with_target(false)
            .init();
    }


    let app = Router::new()
        .route("/images", post(upload_image_route))
        .layer(DefaultBodyLimit::max(MAX_IMAGE_SIZE))
        .layer(middleware::from_fn(auth))
        .route("/", get(health_check_route));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));

    tracing::info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("received shutdown signal");
}


async fn auth<B>(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let token = auth.token();
    let key = match env::var("API_KEY") {
        Ok(key) => key,
        Err(e) => {
            tracing::error!("could not get api key: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    if token == key {
        let response = next.run(request).await;
        Ok(response)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

async fn health_check_route() -> StatusCode {
    return StatusCode::OK;
}

async fn upload_image_route(body: Bytes) -> Response {
    let file_name = match put_image_usecase(body.as_ref()).await {
        Ok(file_name) => file_name,
        Err(e) => {
            tracing::error!("could not put image: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(ResponseError { error: "could not put image".to_string() })
            ).into_response();
        }
    };


    return (StatusCode::CREATED, Json(UploadImageResponse {
        file_name,
    })).into_response();
}

#[derive(Serialize)]
#[derive(Debug)]
struct UploadImageResponse {
    file_name: String,
}

#[derive(Serialize)]
#[derive(Debug)]
struct ResponseError {
    error: String
}

#[derive(Debug)]
#[derive(Clone)]
enum ImageVariants {
    Thumbnail,
    Original
}

// ------ usecase ------

async fn put_image_usecase(data: &[u8]) -> Result<String, String> {
    let file_name = Uuid::new_v4();

    let (thumbnail_result, original_result) = tokio::join!(
        put_image_internal(
            file_name.to_string(),
            data,
            ImageVariants::Thumbnail
        ),
        put_image_internal(
            file_name.to_string(),
            data,
            ImageVariants::Original
        )
    );

    return match (thumbnail_result, original_result) {
        (Ok(()), Ok(())) => Ok(file_name.to_string()),
        (Err(e), _) => Err(format!("could not put thumbnail: {}", e)),
        (_, Err(e)) => Err(format!("could not put original: {}", e))
    };
}

async fn put_image_internal(id: String, data: &[u8], variant: ImageVariants) -> Result<(), String> {
    let (image, format) = match format_image(data, variant.clone()).await {
        Ok((image, format)) => (image, format),
        Err(e) => return Err(format!("could not format image: {}", e))
    };

    return upload_image(id, image, variant, format).await;
}

// ------ usecase end ------

// ------ image gateway ------

// TODO:
// - add avif support?
// - convert gif to webp?
// - only scale down if image is larger than 250x250
// - scale up if image is smaller than 250x250?
async fn format_image(data: &[u8], variant: ImageVariants) -> Result<(Vec<u8>, image::ImageFormat), String> {
    let format = match image::guess_format(data) {
        Ok(f) => match f {
            image::ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            image::ImageFormat::Png => image::ImageFormat::Png,
            image::ImageFormat::Gif => image::ImageFormat::Gif,
            image::ImageFormat::WebP => image::ImageFormat::WebP,
            // image::ImageFormat::Avif => image::ImageFormat::Avif,
            _ => return Err(format!("unsupported image format: {:?}", f))
        },
        Err(e) => return Err(format!("could not derive image format: {}", e))
    };
    
    match image::load_from_memory_with_format(data, format) {
        Ok(image) => {
            let resized_image = match variant {
                ImageVariants::Thumbnail => image.thumbnail(250, 250),
                ImageVariants::Original => image
            };
            
            // Generally, we want to keep the original in its existing format and convert thumbnails to webp for optimized size
            let output_format = match (format, variant) {
                (image::ImageFormat::Jpeg, ImageVariants::Original) => image::ImageFormat::Jpeg,
                (image::ImageFormat::Png, ImageVariants::Original) => image::ImageFormat::Png,
                (image::ImageFormat::Gif, ImageVariants::Original) => image::ImageFormat::Gif,
                (image::ImageFormat::WebP, ImageVariants::Original) => image::ImageFormat::WebP,
                // (image::ImageFormat::Avif, ImageVariants::Original) => image::ImageFormat::Avif,
                (image::ImageFormat::Jpeg, ImageVariants::Thumbnail) => image::ImageFormat::WebP,
                (image::ImageFormat::Png, ImageVariants::Thumbnail) => image::ImageFormat::WebP,
                (image::ImageFormat::Gif, ImageVariants::Thumbnail) => image::ImageFormat::Gif,
                (image::ImageFormat::WebP, ImageVariants::Thumbnail) => image::ImageFormat::WebP,
                // (image::ImageFormat::Avif, ImageVariants::Original) => image::ImageFormat::Avif,
                _ => return Err("unsupported output format".to_string())
            };

            let mut buffer = Cursor::new(Vec::new());
            match resized_image.write_to(&mut buffer, output_format) {
                Ok(_) => (),
                Err(e) => return Err(format!("could not write image: {}", e))
            }

            return Ok((buffer.into_inner(), output_format));
        }
        Err(e) => return Err(format!("could not load image: {}", e))
    }
}

// ------ image gateway end ------

// ------ s3 gateway ------

// TODO:
// - make s3 client singleton and figure out di with axum
// - read env variables into singleton settings and inject with di
async fn upload_image(id: String, image: Vec<u8>, variant: ImageVariants, format: image::ImageFormat) -> Result<(), String> {
    let region = match env::var("REGION") {
        Ok(r) => Region::new(r),
        Err(e) => return Err(format!("could not read REGION: {}", e))
    };
    let endpoint = match env::var("ENDPOINT") {
        Ok(e) => e,
        Err(e) => return Err(format!("could not read ENDPOINT: {}", e))
    };
    let credentials = match (env::var("ACCESS_KEY_ID"), env::var("SECRET_ACCESS_KEY")) {
        (Ok(id), Ok(key)) => Credentials::new(id, key, None, None, "Static"),
        (Err(e), _) => return Err(format!("could not read ACCESS_KEY_ID: {}", e)),
        (_, Err(e)) => return Err(format!("could not read SECRET_ACCESS_KEY: {}", e))
    };
    let bucket = match env::var("BUCKET") {
        Ok(b) => b,
        Err(e) => return Err(format!("could not read BUCKET: {}", e))
    };

    let config = aws_config::from_env()
        .region(region)
        .endpoint_url(endpoint)
        .credentials_provider(credentials)
        .load().await;
    let client = aws_sdk_s3::Client::new(&config);
    
    let content_type = format!("image/{:?}", format).to_lowercase();

    let path = match variant {
        ImageVariants::Thumbnail => format!("images/thumbnails/{}", id),
        ImageVariants::Original => format!("images/originals/{}", id)
    };

    return match client.put_object()
        .bucket(bucket)
        .key(path)
        .content_type(content_type)
        .body(ByteStream::from(image))
        .send()
        .await {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("could not upload image: {}", e))
    };
}

// ------ s3 gateway end ------