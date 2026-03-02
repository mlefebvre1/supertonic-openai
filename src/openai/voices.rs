use std::{path::PathBuf, sync::Arc};

use axum::Extension;

use crate::{
    app::SharedState,
    openai::{OpenAIError, error::AppJson},
};

// async fn combine_voices() -> &'static str {
//     "TODO"
// }

pub async fn list_voices(
    Extension(state): Extension<Arc<SharedState>>,
) -> Result<AppJson<Vec<String>>, OpenAIError> {
    let voices = list_voice_names(&state.asset_path)?;

    Ok(AppJson::new(voices))
}

fn list_voice_files(asset_path: &str) -> anyhow::Result<Vec<PathBuf>> {
    std::fs::read_dir(format!("{}/voice_styles", asset_path))?
        .map(|entry| {
            let entry = entry?;
            Ok(entry.path())
        })
        .collect::<anyhow::Result<Vec<PathBuf>>>()
}

fn list_voice_names(asset_path: &str) -> Result<Vec<String>, OpenAIError> {
    let files = list_voice_files(asset_path)?;
    Ok(files
        .iter()
        .map(|entry| entry.file_stem().unwrap().to_string_lossy().into_owned())
        .collect::<Vec<String>>())
}
