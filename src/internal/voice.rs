use std::collections::HashMap;

use tokio::sync::OnceCell;

use super::error::Error;
use crate::third_party::{Style, load_voice_style};

pub struct Voice {
    file_path: String,
    data: OnceCell<Style>,
}

impl Voice {
    pub fn new(file_path: String) -> Self {
        Self {
            file_path,
            data: OnceCell::new(),
        }
    }

    pub async fn data(&self) -> Result<&Style, Error> {
        Ok(self
            .data
            .get_or_try_init(|| async { self._internal_data() })
            .await?)
    }

    fn _internal_data(&self) -> anyhow::Result<Style> {
        tracing::debug!(voice_path=%self.file_path, "loading voice style.");
        load_voice_style(std::slice::from_ref(&self.file_path), false)
    }
}

pub fn lazy_init_voices(assets_path: String) -> anyhow::Result<HashMap<String, Voice>> {
    let mut voices = HashMap::new();
    let voice_styles_path = format!("{}/voice_styles", assets_path);
    let entries = std::fs::read_dir(&voice_styles_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                voices.insert(
                    file_stem.to_string(),
                    Voice::new(path.to_string_lossy().to_string()),
                );
            }
        }
    }

    Ok(voices)
}
