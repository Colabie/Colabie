// #![feature(test)]
// extern crate test;
// use bitcode::{Decode, Encode};

// use schemou::*;

// #[derive(Schemou, Debug, Encode, Decode)]
// struct Foo {
//     a: String,
//     b: Vec<u8>,
//     c: String,
//     d: Vec<u8>,
// }

// impl Foo {
//     pub fn new() -> Self {
//         Self {
//             a: "The quick brown fox jumps over the lazy dog.".repeat(10000),
//             b: (1..100000).map(|i| (i % 255) as u8).collect(),
//             c: ".dog lazy the over jumps fox brown quick The".repeat(10000),
//             d: (1..100000).map(|i| (i % 255) as u8).collect(),
//         }
//     }
// }

// fn main() {
//     let data = Foo::new().serialize_buffered();
//     println!("{}", data.len());
// }


// #[bench]
// fn bench_serialize_schemou(b: &mut test::Bencher) {
//     let foo = Foo::new();
//     let mut v = Vec::with_capacity(foo.serialize_buffered().len());

//     b.iter(|| {
//         _ = foo.serialize(&mut v);
//         v.clear();
//     });
// }

// #[bench]
// fn bench_serialize_bitcode(b: &mut test::Bencher) {
//     let foo = Foo::new();
//     b.iter(|| {
//         _ = bitcode::encode(&foo);
//     });
// }

// #[bench]
// fn bench_deserialize_schemou(b: &mut test::Bencher) {
//     let foo = Foo::new();
//     let d = foo.serialize_buffered();

//     b.iter(|| {
//         _ = Foo::deserialize(&d);
//     });
// }

// #[bench]
// fn bench_deserialize_bitcode(b: &mut test::Bencher) {
//     let foo = Foo::new();
//     let d = bitcode::encode(&foo);

//     b.iter(|| {
//         let _: Foo = bitcode::decode(&d).unwrap();
//     });
// }
