mod error;

pub use error::SerdeError;
pub use schemou_macro::Schemou;

/// The type that will be used to store the length of the slice.
type LengthPrefix = u32;

/// Number of bytes used to store the length of the slice.
const LENGTH_BYTES: usize = std::mem::size_of::<LengthPrefix>();

pub trait Serde {
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError>
    where
        Self: Sized;
}

#[derive(Debug, Clone)]
pub struct ShortIdStr(String);

impl ShortIdStr {
    pub fn new(s: impl AsRef<str>) -> Result<Self, SerdeError> {
        s.as_ref()
            .chars()
            .all(|i| {
                (i.is_ascii_alphabetic() && i.is_lowercase())
                    || i.is_ascii_digit()
                    || i == '_'
                    || i == '.'
            })
            .then(|| ShortIdStr(s.as_ref().to_string()))
            .ok_or(SerdeError::InvalidChars)
    }
}

impl std::ops::Deref for ShortIdStr {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serde for ShortIdStr {
    fn serialize(&self) -> Vec<u8> {
        let bytes = self.as_bytes();
        if bytes.len() >= 255 {
            panic!()
        }

        let mut data = Vec::with_capacity(bytes.len() + 1);

        data.push(self.len() as u8);
        data.extend_from_slice(bytes);

        data
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
        let len = data[0] as usize;

        Ok((
            ShortIdStr::new(
                String::from_utf8(
                    data.get(1..len + 1)
                        .ok_or(SerdeError::NotEnoughData)?
                        .to_vec(),
                )
                .map_err(|_| SerdeError::InvalidUTF8)?,
            )?,
            len + 1,
        ))
    }
}

impl Serde for Vec<u8> {
    fn serialize(&self) -> Vec<u8> {
        if self.len() >= LengthPrefix::MAX as usize {
            panic!()
        }

        let mut data = Vec::with_capacity(self.len() + LENGTH_BYTES);

        data.extend_from_slice(&(self.len() as LengthPrefix).to_be_bytes());
        data.extend_from_slice(self);

        data
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
        let [a, b, c, d] = data.get(0..LENGTH_BYTES).ok_or(SerdeError::NotEnoughData)? else {
            unreachable!()
        };

        let len = u32::from_be_bytes([*a, *b, *c, *d]) as usize;

        Ok((
            data.get(LENGTH_BYTES..len + LENGTH_BYTES)
                .ok_or(SerdeError::NotEnoughData)?
                .to_vec(),
            len + LENGTH_BYTES,
        ))
    }
}

impl Serde for String {
    fn serialize(&self) -> Vec<u8> {
        let bytes = self.as_bytes().to_vec();
        bytes.serialize()
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError> {
        Vec::<u8>::deserialize(data).and_then(|(v, l)| {
            String::from_utf8(v)
                .map(|s| (s, l))
                .map_err(|_| SerdeError::InvalidUTF8)
        })
    }
}
