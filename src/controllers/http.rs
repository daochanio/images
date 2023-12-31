use crate::{common::variant::Variant, container::Container};
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::iter::once;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::{any::Any, time::Duration};
use tokio::signal::unix::{signal, SignalKind};
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer, sensitive_headers::SetSensitiveRequestHeadersLayer,
    timeout::TimeoutLayer, validate_request::ValidateRequestHeaderLayer,
};
use tracing::{event, info_span, log::info, Instrument, Level};
use uuid::Uuid;

const MAX_IMAGE_SIZE_BYTES: usize = 5 * 1024 * 1024;
const MAX_REQUEST_DURATION_SECONDS: u64 = 30;

pub async fn start(container: Arc<Container>) {
    let app = Router::new()
        .nest(
            "/v1",
            Router::new()
                .route("/images", post(upload_image_route))
                .route("/images/:file_name", get(get_image_route))
                .route("/avatars", put(upload_avatar_route))
                .layer(
                    ServiceBuilder::new()
                        .layer(SetSensitiveRequestHeadersLayer::new(once(
                            header::AUTHORIZATION,
                        )))
                        .layer(ValidateRequestHeaderLayer::bearer(
                            container.settings.api_key().as_str(),
                        ))
                        .layer(DefaultBodyLimit::max(MAX_IMAGE_SIZE_BYTES)),
                ),
        )
        .route("/", get(health_check_route))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(trace_id))
                .layer(middleware::from_fn(request_event))
                .layer(CatchPanicLayer::custom(handle_panic))
                .layer(TimeoutLayer::new(Duration::from_secs(
                    MAX_REQUEST_DURATION_SECONDS,
                ))),
        )
        .with_state(container);

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8081));

    info!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let mut interrupt_signal =
        signal(SignalKind::interrupt()).expect("Failed to register interrupt signal handler");
    let mut terminate_signal =
        signal(SignalKind::terminate()).expect("Failed to register terminate signal handler");

    tokio::select! {
        _ = interrupt_signal.recv() => {},
        _ = terminate_signal.recv() => {},
    }

    tracing::info!("received shutdown signal, exiting http");
}

fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic message".to_string()
    };

    tracing::error!("caught panic: {}", details);

    return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse { error: details }),
    )
        .into_response();
}

// TODO: Get trace id from headers if present
async fn trace_id<B>(request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let trace_id = Uuid::new_v4();
    let span = info_span!("request", %trace_id);
    async move {
        let response = next.run(request).await;
        Ok(response)
    }
    .instrument(span)
    .await
}

async fn request_event<B>(request: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let start = std::time::Instant::now();
    let method = request.method().to_string();
    let uri = request.uri().to_string();

    let response = next.run(request).await;

    let status = response.status().as_u16();
    let duration_ms = start.elapsed().as_millis();

    if status >= 500 {
        event!(Level::ERROR, status, method, uri, duration_ms, "response",);
    } else if status >= 400 {
        event!(Level::WARN, status, method, uri, duration_ms, "response",);
    } else {
        event!(Level::INFO, status, method, uri, duration_ms, "response",);
    };

    Ok(response)
}

async fn health_check_route() -> StatusCode {
    return StatusCode::OK;
}

async fn upload_image_route(State(container): State<Arc<Container>>, body: Bytes) -> Response {
    let file_name = Uuid::new_v4();

    return match container
        .upload_image
        .execute(file_name.to_string(), body.as_ref(), Variant::Thumbnail)
        .await
    {
        Ok(image) => (StatusCode::CREATED, Json(image)).into_response(),
        Err(e) => {
            tracing::warn!("could not put image: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: String::from("could not put image"),
                }),
            )
                .into_response()
        }
    };
}

async fn upload_avatar_route(
    State(container): State<Arc<Container>>,
    Json(body): Json<UploadAvatarRequest>,
) -> Response {
    return match container.upload_avatar.execute(body.url, body.is_nft).await {
        Ok(avatar) => (StatusCode::CREATED, Json(avatar)).into_response(),
        Err(e) => {
            tracing::warn!("could not put avatar: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: String::from("could not put avatar"),
                }),
            )
                .into_response()
        }
    };
}

async fn get_image_route(
    State(container): State<Arc<Container>>,
    Path(file_name): Path<String>,
) -> Response {
    return match container
        .get_image
        .execute(file_name, Variant::Thumbnail)
        .await
    {
        Ok(image) => match image {
            Some(image) => (StatusCode::OK, Json(image)).into_response(),
            None => (StatusCode::NOT_FOUND).into_response(),
        },
        Err(e) => {
            tracing::warn!("could not get image exists: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: String::from("could not get image exists"),
                }),
            )
                .into_response()
        }
    };
}

#[derive(Deserialize, Debug)]
struct UploadAvatarRequest {
    url: String,
    is_nft: bool,
}

#[derive(Serialize, Debug)]
struct ErrorResponse {
    error: String,
}
