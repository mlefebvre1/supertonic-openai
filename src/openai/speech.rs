use std::io::Read;

use axum::{
    Json,
    extract::Extension,
    http::{HeaderMap, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::{internal::AppState, openai::OpenAIError, third_party::write_wav_file};

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

pub async fn create_speech(
    Extension(state): Extension<Arc<AppState>>,
    Json(params): Json<SpeechBodyParams>,
) -> Result<impl IntoResponse, OpenAIError> {
    // TODO: should use a cache to load voices, and should return better errors
    println!("Creating speech with params: {:?}", params); //TODO: using logging library

    let sample_rate = { state.tts.lock().await.sample_rate };
    let (audio_data, duration) = inference(&state, &params).await?;
    let actual_len = (sample_rate as f32 * duration) as usize;
    let audio_data = &audio_data[..actual_len.min(audio_data.len())];

    let wav = create_wav(audio_data, sample_rate)?;

    let mut headers = HeaderMap::new();

    //Unwrap SAFETY: this can't fail, because audio/wav is a valid header value, and we are not using any user input here.
    headers.insert(header::CONTENT_TYPE, "audio/wav".parse().unwrap());

    Ok((headers, wav))
}

fn create_wav(audio_data: &[f32], sample_rate: i32) -> anyhow::Result<Vec<u8>> {
    let mut tmp = tempfile::NamedTempFile::with_suffix(".wav")?;
    write_wav_file(tmp.path(), audio_data, sample_rate)?;

    let mut out = vec![];
    tmp.read_to_end(&mut out)?;
    Ok(out)
}

async fn inference(
    state: &AppState,
    params: &SpeechBodyParams,
) -> Result<(Vec<f32>, f32), OpenAIError> {
    let voice_name = params.voice.clone().unwrap_or("M1".to_string());

    let voice = state
        .get_voice(&voice_name)
        .ok_or(OpenAIError::VoiceNotFound)?;

    let style = voice.data().await?;

    let mut tts = state.tts.lock().await;

    Ok(tts.call(
        &params.input,
        "en",
        style,
        params.total_step.unwrap_or(10) as usize,
        params.speed.unwrap_or(1.3),
        params.silence_duration.unwrap_or(0.0),
    )?)
}
