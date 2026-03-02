use axum::{
    extract::FromRequest,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenAIError {
    #[error("voice not found")]
    VoiceNotFound(String),

    #[error("model not found")]
    ModelNotFound(),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// TODO:Should probably moved to http specific module
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(OpenAIError))]
pub struct AppJson<T>(T);
impl<T> AppJson<T> {
    pub fn new(inner: T) -> Self {
        Self(inner)
    }
}

impl<T> IntoResponse for AppJson<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

impl IntoResponse for OpenAIError {
    fn into_response(self) -> Response {
        // How we want errors responses to be serialized
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }
        let (status, message) = match &self {
            OpenAIError::VoiceNotFound(e) => (StatusCode::NOT_FOUND, e.to_string()),
            OpenAIError::ModelNotFound() => (StatusCode::NOT_FOUND, "model not found".to_string()),
            OpenAIError::Io(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            OpenAIError::Other(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };

        (status, AppJson(ErrorResponse { message })).into_response()
    }
}
// TODO:Should probably moved to http specific module
