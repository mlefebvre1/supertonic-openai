use crate::third_party::TextToSpeech;
use tokio::sync::{Mutex, OnceCell};

pub struct SharedState {
    pub asset_path: String,
    pub tts: Mutex<TextToSpeech>,
    _voices: OnceCell<Vec<String>>,
}

impl SharedState {
    pub fn new(asset_path: &str, tts: TextToSpeech) -> Self {
        Self {
            asset_path: asset_path.to_string(),
            tts: Mutex::new(tts),
            _voices: OnceCell::new(),
        }
    }

    pub async fn voices(&self) -> Result<Vec<String>, String> {
        let voices = self
            ._voices
            .get_or_init(|| async {
                std::fs::read_dir(format!("{}/voice_styles", self.asset_path))
                    .unwrap()
                    .map(|entry| {
                        let entry = entry.unwrap();
                        entry.file_name().into_string().unwrap()
                    })
                    .collect()
            })
            .await
            .clone();
        Ok(voices)
    }
}
