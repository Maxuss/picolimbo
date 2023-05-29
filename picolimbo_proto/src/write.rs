// This code takes some snippets from the Valence project:
// https://github.com/valence-rs/valence/blob/e3c0aec9670523cab6517ceb8a16de6d200dea62/crates/valence_core/src/packet/var_int.rs
// Valence is licensed under the MIT license.

use std::mem::size_of;

use bytes::{BufMut, BytesMut};
use lobsterchat::component::Component;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    error::{ProtoError, Result},
    Identifier, JsonOut, Varint,
};

pub trait Encodeable {
    fn encode(&self, out: &mut BytesMut) -> Result<()>;

    fn predict_size(&self) -> usize {
        0
    }
}

// Varints
impl Encodeable for Varint {
    fn encode(&self, out: &mut BytesMut) -> Result<()> {
        let x = self.0 as u64;
        let stage1 = (x & 0x000000000000007f)
            | ((x & 0x0000000000003f80) << 1)
            | ((x & 0x00000000001fc000) << 2)
            | ((x & 0x000000000fe00000) << 3)
            | ((x & 0x00000000f0000000) << 4);

        let leading = stage1.leading_zeros();

        let unused_bytes = (leading - 1) >> 3;
        let bytes_needed = 8 - unused_bytes;

        // set all but the last MSBs
        let msbs = 0x8080808080808080;
        let msbmask = 0xffffffffffffffff >> (((8 - bytes_needed + 1) << 3) - 1);

        let merged = stage1 | (msbs & msbmask);
        let bytes = merged.to_le_bytes();

        out.extend_from_slice(&bytes[..bytes_needed as usize]);
        Ok(())
    }

    fn predict_size(&self) -> usize {
        Varint::size_of(self.0)
    }
}

// Primitives
macro_rules! impl_encodeable_for_primitive  {
    ($(
        $ty:ident ($mtd:ident)
    ),+) => {
        $(
            impl Encodeable for $ty {
                fn encode(&self, out: &mut BytesMut) -> Result<()> {
                    out.$mtd(*self);
                    Ok(())
                }
            }
        )+
    };
}

impl_encodeable_for_primitive!(
    u8(put_u8),
    i8(put_i8),
    u16(put_u16),
    i16(put_i16),
    u32(put_u32),
    i32(put_i32),
    u64(put_u64),
    i64(put_i64),
    f32(put_f32),
    f64(put_f64)
);

// Strings
impl Encodeable for &str {
    fn encode(&self, out: &mut BytesMut) -> Result<()> {
        let len = self.len() as i32;
        if len > 32767 {
            return Err(ProtoError::StringError(len, 32767));
        }

        Varint(len).encode(out)?;

        out.extend_from_slice(self.as_bytes());

        Ok(())
    }

    fn predict_size(&self) -> usize {
        self.len()
    }
}

impl Encodeable for String {
    fn encode(&self, out: &mut BytesMut) -> Result<()> {
        <&str>::encode(&self.as_str(), out)
    }

    fn predict_size(&self) -> usize {
        self.len()
    }
}

// Chat
impl Encodeable for Component {
    fn encode(&self, out: &mut BytesMut) -> Result<()> {
        let str = self.to_string();
        let len = str.len() as i32;
        if len > 262144 {
            return Err(ProtoError::StringError(len, 262144));
        }

        Varint(len).encode(out)?;

        out.extend_from_slice(str.as_bytes());

        Ok(())
    }
}

// Id
impl Encodeable for Identifier {
    fn encode(&self, out: &mut BytesMut) -> Result<()> {
        let mut str_out = String::with_capacity(self.0.len() + 1 + self.1.len());
        str_out.push_str(&self.0);
        str_out.push(':');
        str_out.push_str(&self.1);

        str_out.encode(out)
    }

    fn predict_size(&self) -> usize {
        self.0.len() + 1 + self.1.len()
    }
}

// UUID
impl Encodeable for Uuid {
    fn encode(&self, out: &mut BytesMut) -> Result<()> {
        let (most, least) = self.as_u64_pair();
        most.encode(out)?;
        least.encode(out)
    }

    fn predict_size(&self) -> usize {
        size_of::<u64>() * 2
    }
}

// Json
impl<'v, T: Serialize> Encodeable for JsonOut<'v, T> {
    fn encode(&self, out: &mut BytesMut) -> Result<()> {
        let json_string = serde_json::to_string(&self.0)
            .map_err(|e| ProtoError::SerializationError(e.to_string()))?;
        json_string.encode(out)
    }
}

#[cfg(test)]
mod tests {
    use crate::Identifier;
    use bytes::BytesMut;
    use uuid::Uuid;

    use super::{Encodeable, Result, Varint};

    #[test]
    fn test_varint_write() -> Result<()> {
        let mut buf = BytesMut::new();
        Varint(123456).encode(&mut buf)?;
        assert_eq!(&[192, 196, 7], &buf[..]);
        Ok(())
    }

    #[test]
    fn test_string_write() -> Result<()> {
        let mut buf = BytesMut::new();
        "hello, world!".encode(&mut buf)?;
        assert_eq!(
            &[13, 104, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33],
            &buf[..]
        );
        Ok(())
    }

    #[test]
    fn test_id_write() -> Result<()> {
        let mut buf = BytesMut::new();
        let id: Identifier = "minecraft:hello".into();
        id.encode(&mut buf)?;
        assert_eq!(
            &[15, 109, 105, 110, 101, 99, 114, 97, 102, 116, 58, 104, 101, 108, 108, 111],
            &buf[..]
        );
        Ok(())
    }

    #[test]
    fn test_uuid_write() -> Result<()> {
        let mut buf = BytesMut::new();
        let id: Uuid = Uuid::nil();
        id.encode(&mut buf)?;
        assert_eq!(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], &buf[..]);
        Ok(())
    }

    macro_rules! build_primitive_write_test {
        ($(
            $name:ident($val:literal) ? [$($expected:literal),+]
        );+ $(;)?) => {
            $(
                #[test]
                fn $name() -> Result<()> {
                    let mut buf = BytesMut::new();
                    $val.encode(&mut buf)?;
                    assert_eq!((&[$($expected),+]), &buf[..]);
                    Ok(())
                }
            )+
        };
    }

    build_primitive_write_test! {
        test_u8_write(123u8) ? [123];
        test_i8_write(123i8) ? [123];
        test_u16_write(12345u16) ? [48, 57];
        test_i16_write(12345i16) ? [48, 57];
        test_u32_write(123456u32) ? [0, 1, 226, 64];
        test_i32_write(-123456i32) ? [255, 254, 29, 192];
        test_u64_write(123456789u64) ? [0, 0, 0, 0, 7, 91, 205, 21];
        test_i64_write(-123456789i64) ? [255, 255, 255, 255, 248, 164, 50, 235];
        test_f32_write(12345.6789f32) ? [70, 64, 230, 183];
        test_f64_write(-1.23456789f64) ? [191, 243, 192, 202, 66, 131, 222, 27];
    }
}
