mod error;
mod impls;

pub use error::SerdeError;
pub use schemou_macro::Schemou;

/// The type that will be used to store the length of the slice.
type LengthPrefix = u32;

/// Number of bytes used to store the length of the slice.
const LENGTH_BYTES: usize = std::mem::size_of::<LengthPrefix>();

pub trait Serde {
    /// Write the serialized data to output and return the bytes written
    fn serialize(&self, output: &mut Vec<u8>) -> usize;
    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct ShortIdStr(String);

fn serialize_with_length_prefix(slice: &[u8], output: &mut Vec<u8>) -> usize {
    if slice.len() >= LengthPrefix::MAX as usize {
        panic!()
    }

    output.extend_from_slice(&(slice.len() as LengthPrefix).to_be_bytes());
    output.extend_from_slice(slice);

    slice.len() + LENGTH_BYTES
}

fn deserialize_with_length_prefix<T, F: FnOnce(&[u8], usize) -> T>(
    data: &[u8],
    f: F,
) -> Result<(T, usize), SerdeError> {
    let [a, b, c, d] = data.get(0..LENGTH_BYTES).ok_or(SerdeError::NotEnoughData)? else {
        unreachable!()
    };

    let len = u32::from_be_bytes([*a, *b, *c, *d]) as usize;

    Ok((
        f(
            data.get(LENGTH_BYTES..len + LENGTH_BYTES)
                .ok_or(SerdeError::NotEnoughData)?,
            len + LENGTH_BYTES,
        ),
        len + LENGTH_BYTES,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Schemou, Debug)]
    struct Info {
        a: ShortIdStr,
        b: String,
        c: Vec<u8>,
    }

    impl PartialEq for Info {
        fn eq(&self, other: &Self) -> bool {
            *self.a == *other.a && self.b == other.b && self.c == other.c
        }
    }

    #[test]
    fn short_id_str_check() {
        assert!(matches!(
            ShortIdStr::new("_some123_valid_username..."),
            Ok(..)
        ));

        assert!(matches!(
            ShortIdStr::new("_some-invalid*username"),
            Err(SerdeError::InvalidChars)
        ));
    }

    #[test]
    fn vec_serde() {
        let original = b"The quick brown fox jumps over the lazy dog.".to_vec();
        let mut serialized = vec![];

        _ = original.serialize(&mut serialized);
        let (deserialized, bytes_read) = Vec::<u8>::deserialize(&serialized).unwrap();

        assert_eq!(deserialized, original);
        assert_eq!(bytes_read, serialized.len());
    }

    #[test]
    fn derive_macro() {
        let original = Info {
            a: ShortIdStr::new("some_valid_username").unwrap(),
            b: "The quick brown fox jumps over the lazy dog.".to_string(),
            c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        };

        let serialized = original.serialize_buffered();
        let (deserialized, bytes_read) = Info::deserialize(&serialized).unwrap();

        assert_eq!(deserialized, original);
        assert_eq!(bytes_read, serialized.len());
    }
}
