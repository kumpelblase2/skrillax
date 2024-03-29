pub mod error;
mod time;

use byteorder::ReadBytesExt;
use bytes::{BufMut, BytesMut};
pub use error::SerializationError;
#[cfg(feature = "derive")]
pub use silkroad_serde_derive::*;
use std::io::Read;
pub use time::SilkroadTime;

macro_rules! implement_primitive {
    ($tt:ty, $read:ident) => {
        impl Serialize for $tt {
            fn write_to(&self, writer: &mut BytesMut) {
                writer.put_slice(&self.to_le_bytes());
            }
        }

        impl ByteSize for $tt {
            fn byte_size(&self) -> usize {
                std::mem::size_of::<$tt>()
            }
        }

        impl Deserialize for $tt {
            fn read_from<T: Read + ReadBytesExt>(reader: &mut T) -> Result<Self, SerializationError> {
                Ok(reader.$read::<byteorder::LittleEndian>()?)
            }
        }
    };
}

/// The `Serialize` trait allows an item to be serialized into a binary
/// representation of itself, which may then be used send it off over
/// the network. This trait requires the [ByteSize] trait to also be
/// present in order to pre-allocate the necessary amount of space for
/// the serialized data.
///
/// `Serialize` only provides one method: [Serialize::write_to]. This
/// method is used to serialize the data and write it into the given
/// buffer. This buffer may already contain data unrelated to this item
/// and may have more space available for more items to follow. However,
/// it is always at least the size provided by [ByteSize].
pub trait Serialize: ByteSize {
    fn write_to(&self, writer: &mut BytesMut);
}

/// `Deserialize` allows an item to be created from a binary representation.
/// Given that there are many different ways such a conversion may fail, this
/// operation will always yield a [Result]. It is not even sure that there
/// are enough bytes available to be read for the deserialization of this
/// item to completed successfully.
pub trait Deserialize {
    fn read_from<T: Read + ReadBytesExt>(reader: &mut T) -> Result<Self, SerializationError>
    where
        Self: Sized; // Technically, we don't care about being `Sized`, but unfortunately, Result does.
}

/// An item having a [ByteSize] implementation specifies it has a known
/// size, independent of if it's [Sized] or not. The size reported by
/// [ByteSize] may sometimes not be the same as [std::mem::size_of], as
/// alignment should not be taken into account for [ByteSize].
pub trait ByteSize {
    fn byte_size(&self) -> usize;
}

impl Serialize for u8 {
    fn write_to(&self, writer: &mut BytesMut) {
        writer.put_u8(*self);
    }
}

impl ByteSize for u8 {
    fn byte_size(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

impl Deserialize for u8 {
    fn read_from<T: Read + ReadBytesExt>(reader: &mut T) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        Ok(reader.read_u8()?)
    }
}

impl Serialize for bool {
    fn write_to(&self, writer: &mut BytesMut) {
        let value = u8::from(*self);
        value.write_to(writer);
    }
}

impl ByteSize for bool {
    fn byte_size(&self) -> usize {
        1
    }
}

impl Deserialize for bool {
    fn read_from<T: Read + ReadBytesExt>(reader: &mut T) -> Result<Self, SerializationError>
    where
        Self: Sized,
    {
        Ok(reader.read_u8()? == 1)
    }
}

implement_primitive!(u16, read_u16);
implement_primitive!(i16, read_i16);
implement_primitive!(u32, read_u32);
implement_primitive!(i32, read_i32);
implement_primitive!(u64, read_u64);
implement_primitive!(i64, read_i64);
implement_primitive!(f32, read_f32);
implement_primitive!(f64, read_f64);

#[cfg(test)]
mod test {
    extern crate self as silkroad_serde;

    use super::*;
    use bytes::{Bytes, BytesMut};
    use silkroad_serde::SerializationError;
    use silkroad_serde_derive::{ByteSize, Deserialize, Serialize};

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    struct Test {
        one: u8,
        two: u16,
        three: u32,
        four: u64,
    }

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    enum TestEnum {
        #[silkroad(value = 1)]
        A,
        #[silkroad(value = 2)]
        B(u8),
    }

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    struct Wrapped {
        inner: Test,
        inner2: TestEnum,
    }

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    struct NormalString(String);

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    struct DoubleString(#[silkroad(size = 2)] String);

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    struct WithVec(Vec<u32>);

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    struct TestCond {
        cond: u8,
        unrelated: String,
        #[silkroad(when = "cond == 1")]
        value: Option<u8>,
    }

    #[derive(Serialize, ByteSize, Deserialize, Eq, PartialEq, Debug)]
    #[silkroad(size = 2)]
    enum LargerEnum {
        #[silkroad(value = 1)]
        A,
        #[silkroad(value = 0xFFF)]
        B,
    }

    macro_rules! test_serialize_deserialize {
        ($ty:ty, $init:expr, $size:literal) => {
            let start = $init;
            assert_eq!($size, start.byte_size());
            let mut out_buff = BytesMut::with_capacity($size);
            start.write_to(&mut out_buff);
            let output = out_buff.freeze();
            let result = <$ty>::try_from(output);
            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(start, result);
        };
    }

    #[test]
    pub fn test_simple() {
        test_serialize_deserialize!(
            Test,
            Test {
                one: 1,
                two: 2,
                three: 3,
                four: 4,
            },
            15
        );
    }

    #[test]
    pub fn test_enum() {
        test_serialize_deserialize!(TestEnum, TestEnum::A, 1);
        test_serialize_deserialize!(TestEnum, TestEnum::B(1), 2);
    }

    #[test]
    pub fn test_wrapped() {
        test_serialize_deserialize!(
            Wrapped,
            Wrapped {
                inner: Test {
                    one: 1,
                    two: 2,
                    three: 3,
                    four: 4,
                },
                inner2: TestEnum::B(3),
            },
            17
        );
    }

    #[test]
    pub fn test_strings() {
        test_serialize_deserialize!(NormalString, NormalString("123".to_string()), 5);
        test_serialize_deserialize!(DoubleString, DoubleString("456".to_string()), 8);
    }

    #[test]
    pub fn test_vec() {
        test_serialize_deserialize!(WithVec, WithVec(vec![123, 456, 789]), 13);
    }

    #[test]
    pub fn test_cond() {
        test_serialize_deserialize!(
            TestCond,
            TestCond {
                cond: 1,
                unrelated: String::from("abc"),
                value: Some(1),
            },
            7
        );

        test_serialize_deserialize!(
            TestCond,
            TestCond {
                cond: 0,
                unrelated: String::from("abc"),
                value: None,
            },
            6
        );
    }

    #[test]
    pub fn test_large_enum() {
        test_serialize_deserialize!(LargerEnum, LargerEnum::B, 2);
    }

    #[test]
    pub fn test_unknown_variant() {
        let bytes = Bytes::from_static(&[3u8]);
        assert!(matches!(
            TestEnum::try_from(bytes),
            Err(SerializationError::UnknownVariation(3, "TestEnum"))
        ));
    }
}
