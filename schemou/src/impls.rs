use crate::*;

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

fn serialize_with_length_prefix(slice: &[u8], output: &mut Vec<u8>) -> usize {
    if slice.len() >= LengthPrefix::MAX as usize {
        panic!("size exceeded length prefix");
    }

    output.extend_from_slice(&(slice.len() as LengthPrefix).to_be_bytes());
    output.extend_from_slice(slice);

    slice.len() + LENGTH_BYTES
}

fn deserialize_with_length_prefix<T, F: FnOnce(&[u8], usize) -> T>(
    data: &[u8],
    f: F,
) -> Result<(T, usize), SerdeError> {
    let len = u32::from_be_bytes(
        data.get(0..LENGTH_BYTES)
            .ok_or(SerdeError::NotEnoughData)?
            .try_into()
            .unwrap(),
    ) as usize;

    Ok((
        f(
            data.get(LENGTH_BYTES..len + LENGTH_BYTES)
                .ok_or(SerdeError::NotEnoughData)?,
            len + LENGTH_BYTES,
        ),
        len + LENGTH_BYTES,
    ))
}

impl Serde for char {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        output.extend((*self as u32).to_be_bytes());
        std::mem::size_of::<Self>()
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
        let raw = u32::from_be_bytes(
            data.get(..std::mem::size_of::<Self>())
                .ok_or(SerdeError::NotEnoughData)?
                .try_into()
                .unwrap(),
        );

        Ok((
            char::from_u32(raw).ok_or(SerdeError::ParsingError {
                ty_name: "char",
                error: format!("invalid character: {raw:X}"),
            })?,
            std::mem::size_of::<Self>(),
        ))
    }
}

macro_rules! impl_serde_for_numbers {
    [ $($t:ty),* ] => {
        $(
            impl Serde for $t {
                fn serialize(&self, output: &mut Vec<u8>) -> usize {
                    output.extend(self.to_be_bytes());
                    std::mem::size_of::<Self>()
                }

                fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
                    Ok((
                        Self::from_be_bytes(
                            data.get(..std::mem::size_of::<Self>())
                                .ok_or(SerdeError::NotEnoughData)?
                                .try_into()
                                .unwrap(),
                        ),
                        std::mem::size_of::<Self>(),
                    ))
                }
            }
        )*

        #[test]
        fn numeric_serde() {
            $(
                let n = <$t>::MAX;

                let v = serialize_buffered(&n);
                let (m, bytes_read) = <$t as Serde>::deserialize(&v).unwrap();

                assert_eq!(n, m);
                assert_eq!(bytes_read, v.len());
            )*
        }
    };
}

impl_serde_for_numbers! {
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
    f32, f64
}

#[cfg(test)]
fn serialize_buffered(d: &impl Serde) -> Vec<u8> {
    let mut data = vec![];
    _ = Serde::serialize(d, &mut data);
    data
}

#[test]
fn char_serde() {
    let original = '💯';
    let serialized = serialize_buffered(&original);
    let (deserialized, bytes_read) = char::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
#[should_panic]
fn char_serde_check() {
    let data = 0x110000_u32.to_be_bytes();
    _ = char::deserialize(&data).unwrap();
}

#[test]
fn vec_serde() {
    let original = b"The quick brown fox jumps over the lazy dog.".to_vec();
    let serialized = serialize_buffered(&original);

    let (deserialized, bytes_read) = Vec::<u8>::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}
