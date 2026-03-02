use std::collections::HashMap;

use tokio::sync::Mutex;

use super::tts::load_model;
use crate::third_party::TextToSpeech;

use super::error::Error;
use super::voice::{Voice, lazy_init_voices};

pub struct AppState {
    voices: HashMap<String, Voice>,
    pub tts: Mutex<TextToSpeech>,
}

impl AppState {
    pub fn new(assets_path: String) -> Result<Self, Error> {
        let voices = lazy_init_voices(assets_path.clone())?;
        let tts = load_model(assets_path.clone(), false)?;

        Ok(Self {
            voices,
            tts: Mutex::new(tts),
        })
    }

    pub fn get_voice(&self, name: &str) -> Option<&Voice> {
        self.voices.get(name)
    }

    pub fn list_voice_names(&self) -> Vec<String> {
        self.voices.keys().cloned().collect()
    }
}
