use axum::response::{IntoResponse, Response};

pub type RegistrieResult<T> = Result<T, RegistrieError>;

#[derive(thiserror::Error, Debug)]
pub enum RegistrieError {}

impl IntoResponse for RegistrieError {
    fn into_response(self) -> Response {
        // TODO: log error and respond with 500, don't leak internals
        todo!()
    }
}
