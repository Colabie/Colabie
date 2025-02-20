use super::*;

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
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        let bytes = self.as_bytes();
        if bytes.len() >= 255 {
            panic!()
        }

        output.push(self.len() as u8);
        output.extend_from_slice(bytes);

        bytes.len() + 1
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
