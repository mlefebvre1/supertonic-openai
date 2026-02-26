use axum::extract::Path;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static MODELS: LazyLock<Vec<Model>> = LazyLock::new(|| {
    vec![Model {
        id: "supertonic2".to_string(),
        object: "model".to_string(),
        created: 0,
        owned_by: "unknown".to_string(),
    }]
});

#[derive(Serialize, Deserialize, Clone)]
struct Model {
    id: String,
    object: String,
    created: u64,
    owned_by: String,
}

async fn get_model(Path(model_id): Path<String>) -> Model {
    MODELS
        .iter()
        .find(|model| model.id == model_id)
        .cloned()
        .unwrap_or_else(|| Model {
            id: "unknown".to_string(),
            object: "model".to_string(),
            created: 0,
            owned_by: "unknown".to_string(),
        })
}

async fn list_models() -> Vec<Model> {
    (*MODELS).clone()
}
