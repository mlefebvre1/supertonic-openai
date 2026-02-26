mod app;
mod openai;
mod third_party;

use std::fs;

use std::sync::Arc;

use tokio::{signal, sync::Mutex};

use axum::{
    Extension, Router,
    extract::Path,
    response::Json,
    routing::{get, post},
};
use hf_hub::api::tokio::Api;
use ort::session::Session;

use openai::create_speech;

use crate::{
    app::SharedState,
    openai::list_voices,
    third_party::{Config, TextToSpeech},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api = Api::new()?;

    let repo = api.model("Supertone/supertonic-2".to_string());

    let config_path = repo.get("onnx/tts.json").await?;

    let config: Config = serde_json::from_str(&fs::read_to_string(config_path)?)?;

    println!("Config: {:?}", config);

    let duration_predictor = repo.get("onnx/duration_predictor.onnx").await?;
    let text_encoder = repo.get("onnx/text_encoder.onnx").await?;
    let vector_estimator = repo.get("onnx/vector_estimator.onnx").await?;
    let vocoder = repo.get("onnx/vocoder.onnx").await?;

    let unicode_indexer_path = repo.get("onnx/unicode_indexer.json").await?;

    println!("Done downloading models");

    let unicode_indexer: Vec<i64> =
        serde_json::from_str(&fs::read_to_string(unicode_indexer_path)?)?;

    let dp_ort = Session::builder()?.commit_from_file(&duration_predictor)?;
    let text_encoder_ort = Session::builder()?.commit_from_file(&text_encoder)?;
    let vector_est_ort = Session::builder()?.commit_from_file(&vector_estimator)?;
    let vocoder_ort = Session::builder()?.commit_from_file(&vocoder)?;

    let text_processor = third_party::UnicodeProcessor::new_from_indexer(unicode_indexer);

    let tts = TextToSpeech::new(
        config,
        text_processor,
        dp_ort,
        text_encoder_ort,
        vector_est_ort,
        vocoder_ort,
    );

    println!("TTS initialized");

    let shared_state = Arc::new(Mutex::new(SharedState { tts }));

    let app = Router::new()
        // /download/<file> ??
        // .route("/audio/voices", post(create_voice))
        .route("/audio/voices", get(list_voices))
        // .route("/audio/voices/combine", post(combine_voices))
        .layer(Extension(shared_state))
        .route("/audio/speech", post(create_speech));

    // .route("/models", get(list_models))
    // .route("/models/{model}", get(get_model));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Starting listener on {}:3000", listener.local_addr()?);

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
