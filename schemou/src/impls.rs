use crate::{Serde, SerdeError};

/// The type that will be used to store the length of the slice.
type LengthPrefix = u32;

/// Number of bytes used to store the length of the slice.
const LENGTH_BYTES: usize = std::mem::size_of::<LengthPrefix>();

impl Serde for Box<[u8]> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        serialize_with_length_prefix(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
        deserialize_with_length_prefix(data, |i, _| i.into())
    }
}

impl Serde for Vec<u8> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        serialize_with_length_prefix(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
        deserialize_with_length_prefix(data, |i, _| i.into())
    }
}

impl Serde for String {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        serialize_with_length_prefix(self.as_bytes(), output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
        deserialize_with_length_prefix(data, |i, l| {
            String::from_utf8(i.to_vec())
                .map(|i| (i, l))
                .map_err(|_| SerdeError::InvalidUTF8)
        })?
        .0
    }
}

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

#[test]
fn vec_serde() {
    let original = b"The quick brown fox jumps over the lazy dog.".to_vec();
    let mut serialized = vec![];

    _ = original.serialize(&mut serialized);
    let (deserialized, bytes_read) = Vec::<u8>::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}
