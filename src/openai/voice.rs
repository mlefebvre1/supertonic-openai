use std::sync::Arc;

use axum::Extension;

use crate::{
    internal::AppState,
    openai::error::AppJson,
};

pub async fn list_voices(Extension(state): Extension<Arc<AppState>>) -> AppJson<Vec<String>> {
    AppJson::new(state.list_voice_names())
}
