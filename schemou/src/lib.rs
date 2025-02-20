pub mod legos;

mod error;
mod impls;

pub use error::SerdeError;
pub use schemou_macro::Schemou;

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
    pub pubkey: Box<[u8]>,
}

#[derive(Schemou)]
pub struct RegisterRes {
    pub commit_id: Box<[u8]>,
}

/*
 * Benchmarks:
 *
 * bitcode deserialization : 311,719.18 ns/iter (+/- 21,198.47)
 * schemou deserialization :  68,009.66 ns/iter (+/- 32,846.51)
 *
 * bitcode serialization   : 731,368.90 ns/iter (+/- 321,491.57)
 * schemou serialization   :  47,193.58 ns/iter (+/- 46,829.73)
 */
