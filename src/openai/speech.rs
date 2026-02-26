use std::io::Read;

use axum::{Json, extract::Extension, extract::Query, http::StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

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
    voice: String,
    instructions: Option<String>,
    response_format: Option<String>,
    speed: Option<f32>,
    stream_format: Option<String>,
    total_step: Option<u8>,
    silence_duration: Option<f32>,
}

impl SpeechBodyParams {
    pub fn builder() -> Self {
        Self::default()
    }

    pub fn input(mut self, input: String) -> Self {
        self.input = input;
        self
    }

    pub fn voice(mut self, voice: String) -> Self {
        self.voice = voice;
        self
    }

    pub fn speed(mut self, speed: f32) -> Self {
        self.speed = Some(speed);
        self
    }

    pub fn total_step(mut self, total_step: u8) -> Self {
        self.total_step = Some(total_step);
        self
    }
    pub fn silence_duration(mut self, silence_duration: f32) -> Self {
        self.silence_duration = Some(silence_duration);
        self
    }
}

pub async fn create_speech(
    Query(params): Query<SpeechBodyParams>,
    Extension(state): Extension<Arc<Mutex<SharedState>>>,
) -> Result<Json<Vec<u8>>, StatusCode> {
    // TODO: should use a cache to load voices, and should return better errors
    println!("Creating speech with params: {:?}", params);
    let style =
        load_voice_style(&[params.voice], true).map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    println!("Loaded voice");
    // Inference
    let (audio_data, _duration) = {
        state
            .lock()
            .await
            .tts
            .call(
                &params.input,
                "en",
                &style,
                params.total_step.unwrap_or(10) as usize,
                params.speed.unwrap_or(1.0),
                params.silence_duration.unwrap_or(0.3),
            )
            .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?
    };

    let mut tmp = tempfile::NamedTempFile::new().map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
    {
        write_wav_file(tmp.path(), &audio_data, state.lock().await.tts.sample_rate)
            .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    let mut out = vec![];
    tmp.read_to_end(&mut out)
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Write wav file
    Ok(Json(out))
}
