use schemou::*;

#[derive(Schemou, Debug)]
struct Info {
    a: legos::ShortIdStr,
    c: Vec<u8>,
    d: f32,
    e: i32,
}

#[derive(Schemou, Debug, PartialEq, Eq)]
struct TupleStruct(i32, Vec<u8>);

impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        *self.a == *other.a && self.c == other.c
    }
}

#[derive(Schemou, PartialEq, Debug)]
enum Foo {
    B,
    C { a: i8, b: u32 },
    D { info: Info },
}

#[test]
fn derive_tuple_struct() {
    let original = TupleStruct(42, vec![10, 20, 30]);

    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = TupleStruct::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
fn derive_struct() {
    let original = Info {
        a: legos::ShortIdStr::new("some_valid_username").unwrap(),
        c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        d: 5252345.0421,
        e: i32::MIN,
    };

    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = Info::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
fn mixed_test() {
    let original = Foo::D {
        info: Info {
            a: legos::ShortIdStr::new("some_valid_username").unwrap(),
            c: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            d: 5252345.0421,
            e: i32::MIN,
        },
    };

    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = Foo::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

#[test]
fn derive_enum() {
    let original = Foo::C { a: 10, b: 20 };

    let serialized = original.serialize_buffered();
    let (deserialized, bytes_read) = Foo::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}
