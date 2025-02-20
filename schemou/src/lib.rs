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
