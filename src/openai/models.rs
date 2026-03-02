use axum::extract::Path;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

use crate::openai::{OpenAIError, error::AppJson};

static MODELS: LazyLock<Vec<Model>> = LazyLock::new(|| {
    vec![Model {
        id: "supertonic".to_string(),
        object: "model".to_string(),
        created: 0, //TODO: set time
        owned_by: "unknown".to_string(),
    }]
});

#[derive(Serialize, Deserialize, Clone)]
pub struct Model {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

pub async fn get_model(Path(model_id): Path<String>) -> Result<AppJson<Model>, OpenAIError> {
    let model = MODELS
        .iter()
        .find(|model| model.id == model_id)
        .ok_or(OpenAIError::ModelNotFound)?
        .clone();
    Ok(AppJson::new(model))
}

pub async fn list_models() -> AppJson<Vec<Model>> {
    AppJson::new(MODELS.clone())
}
