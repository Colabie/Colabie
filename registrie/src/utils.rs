use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};

use schemou::Serde;

#[macro_export]
macro_rules! erout {
    ($err:expr) => {
        $err.map_err(|err| {
            ::tracing::error!("{err}");
            err
        })?
    };
}

pub struct Schemou<T: Serde>(pub T);

impl<T, S> FromRequest<S> for Schemou<T>
where
    T: Serde,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        Ok(Self(
            T::deserialize(&bytes)
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .0,
        ))
    }
}

impl<T> IntoResponse for Schemou<T>
where
    T: Serde,
{
    fn into_response(self) -> Response {
        let mut v = vec![];
        self.0.serialize(&mut v);
        v.into_response()
    }
}
