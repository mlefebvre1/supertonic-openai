mod internal;
mod openai;
mod third_party;

use std::sync::Arc;

use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use axum::{
    Extension, Router,
    routing::{get, post},
};

use clap::Parser;

use openai::create_speech;

use crate::{
    internal::AppState,
    openai::{get_model, list_models, list_voices},
};

#[derive(Parser, Debug)]
#[command(name = "supertonic2-openai")]
#[command(about = "An OpenAI-compatible Text-to-Speech server", long_about = None)]
struct Args {
    /// Path to the assets directory containing ONNX models
    #[arg(short, long, default_value = "./assets")]
    assets_path: String,

    /// Server listening address in the format IP:PORT
    #[arg(short, long, default_value = "0.0.0.0:50051")]
    listen: String,

    /// Enable GPU acceleration for inference
    #[arg(long, default_value_t = false)]
    gpu: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "supertonic2_openai=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();

    tracing::info!("Starting supertonic-openai server");
    let app_state = Arc::new(AppState::new(args.assets_path, args.gpu)?);

    let openai_api_v1 = Router::new()
        .route("/audio/voices", get(list_voices))
        .route("/audio/voices/", get(list_voices))
        .route("/audio/speech", post(create_speech))
        .layer(Extension(app_state))
        .route("/models", get(list_models))
        .route("/models/", get(list_models))
        .route("/models/{model}", get(get_model));

    let app = Router::new()
        .nest("/v1", openai_api_v1)
        .fallback(get(|| async { "Not Found" }));

    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
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
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, shutting down gracefully");
        },
        _ = terminate => {
            tracing::info!("Received terminate signal, shutting down gracefully");
        },
    }
}
