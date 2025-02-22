use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerdeError {
    #[error("ran out of data bytes while parsing, cannot deserialize the remaining fields")]
    NotEnoughData,

    #[error("raw bytes contain invalid UTF-8 data, cannot deserialize string")]
    InvalidUTF8,

    #[error("failed to parse data as `{ty_name}`: {error}")]
    ParsingError{
        ty_name: &'static str,
        error: String,
    },

    #[error("found invalid character")]
    InvalidChar,
}
