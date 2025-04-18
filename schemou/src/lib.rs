pub mod legos;

mod axum;
mod error;
mod impls;

#[cfg(feature = "axum")]
pub use axum::Schemou;
pub use error::SerdeError;
pub use schemou_macro::Serde;

/// The type that will be used to store the length of the slice.
pub type LengthPrefix = u32;

/// Number of bytes used to store the length of the slice.
const LENGTH_BYTES: usize = std::mem::size_of::<LengthPrefix>();

pub trait Serde {
    /// Write the serialized data to output and return the bytes written
    fn serialize(&self, output: &mut Vec<u8>) -> usize;
    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError>
    where
        Self: Sized;
}

const AUTH_SIZE: usize = 2048;

#[derive(Serde)]
pub struct C2RRegister {
    pub username: legos::ShortIdStr,
    // TODO: All schemou types should be Hardened and have explicit invarients, no generic de-serialization
    // labels: enhancement, help wanted
    // Issue URL: https://github.com/Colabie/Colabie/issues/21
    pub pubkey: Box<[u8]>,
}

#[derive(Serde)]
pub struct R2CRegister {
    pub commit_id: Box<[u8]>,
}

#[derive(Serde)]
pub struct C2SAck {
    pub username: legos::ShortIdStr,
}

#[derive(Serde)]
pub struct S2CAuthReq {
    pub random: [u8; AUTH_SIZE],
}

#[derive(Serde)]
pub struct C2SAuthRes {
    pub signed_random: Box<[u8]>,
}

#[derive(Serde)]
pub enum S2CAuthResult {
    Success,
    Failure,
}
