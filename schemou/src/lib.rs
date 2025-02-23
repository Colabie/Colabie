pub mod legos;

mod error;
mod impls;

pub use error::SerdeError;
pub use schemou_macro::Schemou;

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

#[derive(Schemou)]
pub struct RegisterReq {
    pub username: legos::ShortIdStr,
    // TODO: All schemou types should be Hardened and have explicit invarients, no generic de-serialization
    pub pubkey: Box<[u8]>,
}

#[derive(Schemou)]
pub struct RegisterRes {
    pub commit_id: Box<[u8]>,
}
