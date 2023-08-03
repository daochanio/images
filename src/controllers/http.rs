use crate::container::Container;
use axum::{
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{header, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::iter::once;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::{any::Any, time::Duration};
use tokio::signal;
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
            "/api/v1",
            Router::new()
                .route("/images", post(upload_image_route))
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
        Json(ResponseError { error: details }),
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
    let file_name = match container.upload_image.execute(body.as_ref()).await {
        Ok(file_name) => file_name,
        Err(e) => {
            tracing::error!("could not put image: {}", e);
            return (
                StatusCode::BAD_REQUEST,
                Json(ResponseError {
                    error: String::from("could not put image"),
                }),
            )
                .into_response();
        }
    };

    return (StatusCode::CREATED, Json(UploadImageResponse { file_name })).into_response();
}

#[derive(Serialize, Debug)]
struct UploadImageResponse {
    file_name: String,
}

#[derive(Serialize, Debug)]
struct ResponseError {
    error: String,
}
