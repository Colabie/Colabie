use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bitcode::{DecodeOwned, Encode};

#[macro_export]
macro_rules! erout {
    ($err:expr) => {
        $err.map_err(|err| {
            ::tracing::error!("{err}");
            err
        })?
    };
}

pub struct BitCode<T>(pub T);

impl<T, S> FromRequest<S> for BitCode<T>
where
    T: DecodeOwned,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        Ok(Self(
            bitcode::decode(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?,
        ))
    }
}

impl<T> IntoResponse for BitCode<T>
where
    T: Encode,
{
    fn into_response(self) -> Response {
        bitcode::encode(&self.0).into_response()
    }
}
