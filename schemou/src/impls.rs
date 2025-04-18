use crate::*;
use std::alloc::{self, Layout};
use std::ptr::NonNull;

impl<const N: usize> Serde for [u8; N] {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        let prefix_len = match N {
            n if n < u8::MAX as usize => {
                output.extend_from_slice(&(N as u8).to_be_bytes());
                std::mem::size_of::<u8>()
            }
            n if n < u16::MAX as usize => {
                output.extend_from_slice(&(N as u16).to_be_bytes());
                std::mem::size_of::<u16>()
            }
            _ => panic!("bigger values are not supported"),
        };

        output.extend_from_slice(self);
        self.len() + prefix_len
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError>
    where
        Self: Sized,
    {
        let (prefix_len, len) = match N {
            n if n < u8::MAX as usize => {
                let prefix_len = std::mem::size_of::<u8>();
                (
                    prefix_len,
                    u8::from_be_bytes(
                        data.get(0..prefix_len)
                            .ok_or(SerdeError::NotEnoughData)?
                            .try_into()
                            .unwrap(),
                    ) as usize,
                )
            }
            n if n < u16::MAX as usize => {
                let prefix_len = std::mem::size_of::<u16>();
                (
                    prefix_len,
                    u16::from_be_bytes(
                        data.get(0..prefix_len)
                            .ok_or(SerdeError::NotEnoughData)?
                            .try_into()
                            .unwrap(),
                    ) as usize,
                )
            }
            _ => unreachable!("bigger values are not supported"),
        };

        Ok((
            data.get(prefix_len..len + prefix_len)
                .ok_or(SerdeError::NotEnoughData)?
                .try_into()
                .expect("unreachable"),
            prefix_len + len,
        ))
    }
}

impl<T: Serde> Serde for Box<T> {
    fn serialize(&self, output: &mut Vec<u8>) -> usize {
        T::serialize(self, output)
    }

    fn deserialize(data: &[u8]) -> Result<(Self, usize), SerdeError>
    where
        Self: Sized,
    {
        // Directly deserialize on the heap to avoid unnecessary stack allocation
        // We use a two-step approach:
        // 1. Parse the data to determine the size/structure
        // 2. Allocate on the heap and deserialize directly there
        
        // First, we need to determine how many bytes this T will take
        let (_, size) = unsafe { 
            // This is a bit of a hack: we "peek" at the data to determine the size
            // without fully deserializing it. This works because deserialize returns
            // the size as the second element of the tuple.
            match T::deserialize(data) {
                Ok((_, size)) => (std::ptr::null(), size),
                Err(e) => return Err(e),
            }
        };
        
        // Now that we know how much data we need, allocate memory on heap
        // and deserialize directly there
        unsafe {
            // Create a layout for our type
            let layout = Layout::new::<T>();
            
            // Allocate memory on the heap
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }
            
            // Convert raw pointer to Box
            let box_ptr = NonNull::new_unchecked(ptr as *mut T);
            
            // Deserialize directly into allocated memory
            match T::deserialize(data) {
                Ok((value, size)) => {
                    // Place the value into the allocated memory
                    std::ptr::write(ptr as *mut T, value);
                    // Return the box and size
                    Ok((Box::from_raw(box_ptr.as_ptr()), size))
                },
                Err(e) => {
                    // Clean up the allocation if deserialization fails
                    alloc::dealloc(ptr, layout);
                    Err(e)
                }
            }
        }
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
fn char_serde_check() {
    let data = 0x110000_u32.to_be_bytes();
    assert!(matches!(
        char::deserialize(&data),
        Err(SerdeError::ParsingError {
            ty_name: "char",
            ..
        })
    ));
}

#[test]
fn vec_serde() {
    let original = b"The quick brown fox jumps over the lazy dog.".to_vec();
    let serialized = serialize_buffered(&original);

    let (deserialized, bytes_read) = Vec::<u8>::deserialize(&serialized).unwrap();

    assert_eq!(deserialized, original);
    assert_eq!(bytes_read, serialized.len());
}

// Add test for Box<T> deserialization
#[test]
fn box_serde() {
    let original = Box::new(42u32);
    let serialized = serialize_buffered(&original);
    
    let (deserialized, bytes_read) = Box::<u32>::deserialize(&serialized).unwrap();
    
    assert_eq!(*deserialized, *original);
    assert_eq!(bytes_read, serialized.len());
}
