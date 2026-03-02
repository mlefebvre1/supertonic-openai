use std::io::Read;

use axum::{
    Json,
    extract::Extension,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::{
    app::SharedState,
    third_party::{load_voice_style, write_wav_file},
};

/* Parameters
 * input: string
The text to generate audio for. The maximum length is 4096 characters.
maxLength4096 -> DO WE HAVE THE SAME LIMITATION?

model: string or SpeechModel
One of the available TTS models: supertonic

voice: string
The voice to use when generating the audio.

instructions: optional string
Control the voice of your generated audio with additional instructions. Does not work with tts-1 or tts-1-hd.
maxLength4096

response_format: optional "mp3" or "opus" or "aac" or 3 more -> TODO CHECK IF WE WANT TO SUPPORT MORE THAN WAV.
The format to audio in. Supported formats are mp3, opus, aac, flac, wav, and pcm.

speed: optional number
The speed of the generated audio. Select a value from 0.25 to 4.0. 1.0 is the default.

minimum0.25
maximum4
stream_format: optional "sse" or "audio"
The format to stream the audio in. Supported formats are sse and audio. sse is not supported for tts-1 or tts-1-hd.

total_step(2-15): specific to supertonic*/
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SpeechBodyParams {
    input: String,
    model: String,
    voice: Option<String>,
    instructions: Option<String>,
    response_format: Option<String>,
    speed: Option<f32>,
    stream_format: Option<String>,
    total_step: Option<u8>,
    silence_duration: Option<f32>,
}

pub async fn create_speech(
    Extension(state): Extension<Arc<SharedState>>,
    Json(params): Json<SpeechBodyParams>,
) -> Result<impl IntoResponse, StatusCode> {
    // TODO: should use a cache to load voices, and should return better errors
    println!("Creating speech with params: {:?}", params);
    let voice = format!(
        "{}/voice_styles/{}.json",
        state.asset_path,
        params.voice.unwrap_or("M1".to_string())
    );
    let style = load_voice_style(&[voice], true).map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    let sample_rate = { state.tts.lock().await.sample_rate };

    println!("Loaded voice");
    println!("Input is {}", params.input);
    let (audio_data, duration) = {
        state
            .tts
            .lock()
            .await
            .call(
                &params.input,
                "en",
                &style,
                params.total_step.unwrap_or(10) as usize,
                params.speed.unwrap_or(1.3),
                params.silence_duration.unwrap_or(0.0),
            )
            .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    let actual_len = (sample_rate as f32 * duration) as usize;
    let audio_data = &audio_data[..actual_len.min(audio_data.len())];
    println!("duration -> {duration}s       sample_rate -> {sample_rate}");

    //TODO: check if we can avoid using a temporary file
    let mut tmp = tempfile::NamedTempFile::with_suffix(".wav")
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
    {
        write_wav_file(tmp.path(), &audio_data, sample_rate)
            .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let mut out = vec![];
    tmp.read_to_end(&mut out)
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "audio/wav"
            .parse()
            .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?,
    );

    // Write wav file
    Ok((headers, out))
}
