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
    ver::Protocol,
    ArrayPrefix, Identifier, JsonOut, PrefixedArray, UnprefixedByteArray, Varint,
};

pub trait Encodeable {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()>;

    fn predict_size(&self) -> usize {
        0
    }
}

// Varints
impl Encodeable for Varint {
    fn encode(&self, out: &mut BytesMut, _ver: Protocol) -> Result<()> {
        let mut x = self.0 as u32;
        loop {
            let mut temp = (x & 0b0111_1111) as u8;
            x >>= 7;
            if x != 0 {
                temp |= 0b1000_0000;
            }

            out.extend_from_slice(&[temp]);
            if x == 0 {
                break;
            }
        }
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
                fn encode(&self, out: &mut BytesMut, _ver: Protocol) -> Result<()> {
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

impl Encodeable for bool {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        (if *self { 0x01u8 } else { 0x00u8 }).encode(out, ver)
    }

    fn predict_size(&self) -> usize {
        1
    }
}

// Strings
impl Encodeable for &str {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        let len = self.len() as i32;
        if len > 32767 {
            return Err(ProtoError::StringError(len, 32767));
        }

        Varint(len).encode(out, ver)?;

        out.extend_from_slice(self.as_bytes());

        Ok(())
    }

    fn predict_size(&self) -> usize {
        self.len()
    }
}

impl Encodeable for String {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        <&str>::encode(&self.as_str(), out, ver)
    }

    fn predict_size(&self) -> usize {
        self.len()
    }
}

// Chat
impl Encodeable for Component {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        let str = self.to_string();
        let len = str.len() as i32;
        if len > 262144 {
            return Err(ProtoError::StringError(len, 262144));
        }

        Varint(len).encode(out, ver)?;

        out.extend_from_slice(str.as_bytes());

        Ok(())
    }
}

// Id
impl Encodeable for Identifier {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        let mut str_out = String::with_capacity(self.0.len() + 1 + self.1.len());
        str_out.push_str(&self.0);
        str_out.push(':');
        str_out.push_str(&self.1);

        str_out.encode(out, ver)
    }

    fn predict_size(&self) -> usize {
        self.0.len() + 1 + self.1.len()
    }
}

// UUID
impl Encodeable for Uuid {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        if ver >= Protocol::V1_16 {
            // v1.16 changed the way UUIDs are serialized
            let (most, least) = self.as_u64_pair();
            most.encode(out, ver)?;
            least.encode(out, ver)
        } else if ver >= Protocol::V1_7_6 {
            // Since v1.7.6 UUIDs are serialized with hyphens
            let str_id = self.as_hyphenated().to_string();
            str_id.encode(out, ver)
        } else {
            // Prior to v1.7.6 UUIDs are serialized without hyphens
            let str_id = self.as_simple().to_string();
            str_id.encode(out, ver)
        }
    }

    fn predict_size(&self) -> usize {
        size_of::<u64>() * 2
    }
}

// Json
impl<'v, T: Serialize> Encodeable for JsonOut<'v, T> {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        let json_string = serde_json::to_string(&self.0)
            .map_err(|e| ProtoError::SerializationError(e.to_string()))?;
        json_string.encode(out, ver)
    }
}

// Options
impl<T: Encodeable> Encodeable for Option<T> {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        match self {
            Some(v) => {
                true.encode(out, ver)?;
                v.encode(out, ver)
            }
            None => false.encode(out, ver),
        }
    }

    fn predict_size(&self) -> usize {
        match self {
            Some(v) => 1 + v.predict_size(),
            None => 1,
        }
    }
}

// Vecs
impl<'b> Encodeable for UnprefixedByteArray<'b> {
    fn encode(&self, out: &mut BytesMut, _ver: Protocol) -> Result<()> {
        out.extend_from_slice(&self.0);
        Ok(())
    }

    fn predict_size(&self) -> usize {
        self.0.len()
    }
}

impl<'d, V: Encodeable + Clone, P: ArrayPrefix> Encodeable for PrefixedArray<'d, V, P> {
    fn encode(&self, out: &mut BytesMut, ver: Protocol) -> Result<()> {
        P::pfx_write(self.0.len(), out, ver)?;
        self.0
            .iter()
            .map(|each| each.encode(out, ver))
            .collect::<Result<Vec<()>>>()?;
        Ok(())
    }

    fn predict_size(&self) -> usize {
        P::pfx_size(self.0.len()) + self.0.iter().map(Encodeable::predict_size).sum::<usize>()
    }
}

// NBT
impl Encodeable for nbt::Blob {
    fn encode(&self, out: &mut BytesMut, _ver: Protocol) -> Result<()> {
        self.to_writer(&mut out.writer()).map_err(ProtoError::from)
    }
}

impl Encodeable for nbt::Value {
    fn encode(&self, out: &mut BytesMut, _ver: Protocol) -> Result<()> {
        self.to_writer(&mut out.writer()).map_err(ProtoError::from)
    }
}

#[cfg(test)]
mod tests {
    use crate::{ver::Protocol, Identifier};
    use bytes::BytesMut;
    use uuid::Uuid;

    use super::{Encodeable, Result, Varint};

    #[test]
    fn test_varint_write() -> Result<()> {
        let mut buf = BytesMut::new();
        Varint(123456).encode(&mut buf, Protocol::latest())?;
        assert_eq!(&[192, 196, 7], &buf[..]);
        Ok(())
    }

    #[test]
    fn test_string_write() -> Result<()> {
        let mut buf = BytesMut::new();
        "hello, world!".encode(&mut buf, Protocol::latest())?;
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
        id.encode(&mut buf, Protocol::latest())?;
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
        id.encode(&mut buf, Protocol::latest())?;
        assert_eq!(&[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], &buf[..]);
        Ok(())
    }

    #[test]
    fn test_opt_none_write() -> Result<()> {
        let mut buf = BytesMut::new();
        let opt: Option<String> = None;
        opt.encode(&mut buf, Protocol::latest())?;
        assert_eq!(&[0], &buf[..]);
        Ok(())
    }

    #[test]
    fn test_opt_some_write() -> Result<()> {
        let mut buf = BytesMut::new();
        let opt: Option<u8> = Some(123);
        opt.encode(&mut buf, Protocol::latest())?;
        assert_eq!(&[1, 123], &buf[..]);
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
                    $val.encode(&mut buf, Protocol::latest())?;
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
