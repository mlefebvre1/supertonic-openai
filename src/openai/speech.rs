use std::io::Read;

use axum::{
    Json,
    extract::Extension,
    http::{HeaderMap, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::{
    internal::{AppState, ResponseFormat, create_response},
    openai::OpenAIError,
    third_party::write_wav_file,
};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpeechBodyParams {
    /// The text to generate audio for. The maximum length is 4096 characters.
    //TODO: check if we have character limit
    input: String,

    /// One of the available TTS models: supertonic.
    model: String,

    /// The voice to use when generating the audio.
    voice: Option<String>,

    /// Control the voice of your generated audio with additional instructions. (This is not
    /// supported by this model)
    instructions: Option<String>,

    // The format to audio in. Supported formats are normally mp3, opus, aac, flac, wav, and pcm,
    // but currently only wav is supported.
    response_format: Option<String>,

    /// The speed of the generated audio. Select a value from 0.25 to 4.0. 1.0 is the default.
    speed: Option<f32>,

    /// The format to stream the audio in. This is currently not supported.
    stream_format: Option<String>,

    // Quality vs Speed: Higher total-step values produce better quality but take longer
    total_step: Option<u8>,

    /// Duration between each sentence in seconds.
    silence_duration: Option<f32>,
}

#[tracing::instrument(skip(state,params), fields(input=%params.input, voice = ?params.voice, response_format = ?params.response_format))]
pub async fn create_speech(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<SpeechBodyParams>,
) -> Result<impl IntoResponse, OpenAIError> {
    tracing::info!("Creating speech.");

    let sample_rate = { state.tts.lock().await.sample_rate };
    let (audio_data, duration) = inference(&state, &params).await?;
    let actual_len = (sample_rate as f32 * duration) as usize;
    let audio_data = &audio_data[..actual_len.min(audio_data.len())];

    let response_format: ResponseFormat = params
        .response_format
        .clone()
        .unwrap_or("wav".to_string())
        .try_into()?;

    let response = create_response(audio_data, sample_rate as u32, &response_format)?;

    tracing::debug!(
        duration = %duration,
        response_len = %response.len(),
        "Successfully created speech audio.",
    );

    let mut headers = HeaderMap::new();

    match response_format {
        ResponseFormat::Wav => {
            headers.insert(header::CONTENT_TYPE, "audio/wav".parse().unwrap());
        }
        ResponseFormat::OpusOgg => {
            headers.insert(header::CONTENT_TYPE, "audio/opus".parse().unwrap());
        }
    }

    Ok((headers, response))
}

async fn inference(
    state: &AppState,
    params: &SpeechBodyParams,
) -> Result<(Vec<f32>, f32), OpenAIError> {
    let voice_name = params.voice.clone().unwrap_or("M1".to_string());
    tracing::debug!(voice_name=%voice_name, "Using voice style.");

    let voice = state
        .get_voice(&voice_name)
        .ok_or(OpenAIError::VoiceNotFound)?;

    let style = voice.data().await?;

    let mut tts = state.tts.lock().await;

    let total_step = params.total_step.unwrap_or(10) as usize;
    let speed = params.speed.unwrap_or(1.3);
    let silence_duration = params.silence_duration.unwrap_or(0.3);

    tracing::info!(total_step = %total_step, speed=%speed, silence_duration=%silence_duration, "Starting TTS inference.");

    Ok(tts
        .call(
            &params.input,
            "en", //TODO: make this configurable
            style,
            total_step,
            speed,
            silence_duration,
        )
        .inspect_err(|e| tracing::error!(error =%e,"an error occured during inference"))?)
}
