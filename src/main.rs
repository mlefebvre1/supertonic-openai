mod app;
mod openai;
mod third_party;

use std::fs;

use std::sync::Arc;

use tokio::signal;

use axum::{
    Extension, Router,
    routing::{get, post},
};

use clap::Parser;

use openai::create_speech;

use crate::{
    app::SharedState,
    openai::{get_model, list_models, list_voices},
    third_party::load_text_to_speech,
};

#[derive(Parser, Debug)]
#[command(name = "supertonic2-openai")]
#[command(about = "An OpenAI-compatible Text-to-Speech server", long_about = None)]
struct Args {
    /// Path to the assets directory containing ONNX models
    #[arg(short, long, default_value = "./assets")]
    assets_path: String,

    /// Server listening address in the format IP:PORT
    #[arg(short, long, default_value = "0.0.0.0:3000")]
    listen: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let tts = load_text_to_speech(&format!("{}/onnx", args.assets_path), false)?;

    println!("TTS initialized");

    let shared_state = Arc::new(SharedState::new(&args.assets_path, tts));

    let app = Router::new()
        // /download/<file> ??
        // .route("/audio/voices", post(create_voice))
        .route("/audio/voices", get(list_voices))
        // .route("/audio/voices/combine", post(combine_voices))
        .route("/audio/speech", post(create_speech))
        .layer(Extension(shared_state))
        .route("/models", get(list_models))
        .route("/models/{model}", get(get_model));

    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    println!("Starting listener on {}", listener.local_addr()?);

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
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
