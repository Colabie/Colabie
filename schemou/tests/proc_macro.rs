use schemou::*;

#[derive(Schemou, Debug)]
struct Info {
    a: legos::ShortIdStr,
    b: String,
    c: Vec<u8>,
}

impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        *self.a == *other.a && self.b == other.b && self.c == other.c
    }
}

#[test]
fn derive_macro() {
    let original = Info {
        a: legos::ShortIdStr::new("some_valid_username").unwrap(),
        b: "The quick brown fox jumps over the lazy dog.".to_string(),
        c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };

    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = Info::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}
