use axum::response::{IntoResponse, Response};

pub type RegistrieResult<T> = Result<T, RegistrieError>;

#[derive(thiserror::Error, Debug)]
pub enum RegistrieError {}

impl IntoResponse for RegistrieError {
    fn into_response(self) -> Response {
        // Log the error with its details (for debugging)
        tracing::error!("Internal server error: {:?}", self);

        // Return a generic 500 response without exposing internal details
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
            .into_response()
    }
}
