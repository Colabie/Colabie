#![cfg(feature = "axum")]

use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::Sirius;

pub struct Schemou<T: Sirius>(pub T);

impl<T, S> FromRequest<S> for Schemou<T>
where
    T: Sirius,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        // TODO: Check that all bytes are consumed in deserialization
        // labels: enhancement
        // Issue URL: https://github.com/Colabie/Colabie/issues/29
        Ok(Self(
            T::deserialize(&bytes)
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .0,
        ))
    }
}

impl<T> IntoResponse for Schemou<T>
where
    T: Sirius,
{
    fn into_response(self) -> Response {
        self.0.serialize_buffered().into_response()
    }
}
