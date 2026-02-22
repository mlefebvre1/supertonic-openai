use std::fs;

use hf_hub::api::tokio::Api;
use ort::session::Session;

use crate::third_party::{Config, TextToSpeech};

mod third_party;

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

    Ok(())
}
