use crate::{Serde, SerdeError};

#[derive(Debug, Clone)]
pub struct ShortIdStr(String);

impl ShortIdStr {
    pub fn new(s: impl AsRef<str>) -> Result<Self, SerdeError> {
        if s.as_ref().len() > u8::MAX as _ {
            return Err(Self::error(
                "string length exceeded 255 characters".to_string(),
            ));
        }

        s.as_ref()
            .chars()
            .find(|&i| {
                // find invalid char
                !((i.is_ascii_alphabetic() && i.is_lowercase())
                    || i.is_ascii_digit()
                    || i == '_'
                    || i == '.')
            })
            .map_or_else(
                || Ok(ShortIdStr(s.as_ref().to_string())),
                |i| Err(Self::error(format!(
                    "invalid character: '{i}', note: only lowercase alphabetic, digits, '.' and '_' are valid characters for `ShortIdStr`"
                ))),
            )
    }

    fn error(error: String) -> SerdeError {
        SerdeError::ParsingError {
            ty_name: "ShortIdStr",
            error,
        }
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

        // SAFETY: length is already checked in `ShortIdStr::new(..)` function
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

#[test]
fn new_check() {
    assert!(matches!(
        dbg!(ShortIdStr::new("_some123_valid_username...")),
        Ok(..)
    ));

    assert!(matches!(
        ShortIdStr::new("_some-invalid*username"),
        Err(SerdeError::ParsingError { .. })
    ));
}
