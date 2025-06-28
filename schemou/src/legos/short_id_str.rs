use sirius::{Sirius, SiriusError};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ShortIdStr(String);

impl ShortIdStr {
    pub fn new(s: impl Into<String>) -> Result<Self, SiriusError> {
        Self::from_bytes(s.into().into_bytes())
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, SiriusError> {
        if bytes.len() > u8::MAX as _ {
            return Err(Self::error(
                "string length exceeded 255 characters".to_string(),
            ));
        } else if bytes.len() < 3 {
            return Err(Self::error("string length below 3 characters".to_string()));
        }

        let invalid_char = bytes.iter().find(|&i| {
            !((i.is_ascii_alphabetic() && i.is_ascii_lowercase())
                || i.is_ascii_digit()
                || *i == b'_'
                || *i == b'.')
        });

        match invalid_char {
            None => Ok(ShortIdStr(String::from_utf8(bytes).expect("Invariant checked above"))),
            Some(i) => Err(Self::error(format!(
                    "invalid character: '{i}', note: only lowercase alphabetic, digits, '.' and '_' are valid characters for `ShortIdStr`"
                ))),
        }
    }

    fn error(error: String) -> SiriusError {
        SiriusError::ParsingError {
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

impl Sirius for ShortIdStr {
    fn serialize(&self, output: &mut impl std::io::Write) -> Result<usize, SiriusError> {
        let bytes = self.as_bytes();

        // SAFETY: length is already checked in `ShortIdStr::new(..)` function
        output.write_all(&[bytes.len() as u8])?;
        output.write_all(bytes)?;

        Ok(bytes.len() + 1)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SiriusError> {
        let len = *data.first().ok_or(SiriusError::NotEnoughData)? as usize;
        let short_id_str = ShortIdStr::from_bytes(
            data.get(1..len + 1)
                .ok_or(SiriusError::NotEnoughData)?
                .to_owned(),
        )?;

        Ok((short_id_str, len + 1))
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
        Err(SiriusError::ParsingError { .. })
    ));
}
