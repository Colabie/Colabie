use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerdeError {
    #[error("ran out of data bytes while parsing, cannot deserialize the remaining fields")]
    NotEnoughData,

    #[error("failed to parse data as `{ty_name}`: {error}")]
    ParsingError {
        ty_name: &'static str,
        error: String,
    },
}
