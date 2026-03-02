use std::sync::Arc;

use axum::Extension;

use crate::{internal::AppState, openai::error::AppJson};

#[tracing::instrument(skip(state))]
pub async fn list_voices(Extension(state): Extension<Arc<AppState>>) -> AppJson<Vec<String>> {
    tracing::info!("Listing available voices.");
    let voices = state.list_voice_names();
    AppJson::new(voices)
}
