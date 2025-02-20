mod error;

pub use error::SerdeError;

pub trait Serde {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError>
    where
        Self: Sized;
}
