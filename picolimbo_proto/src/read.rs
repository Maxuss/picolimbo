// This code takes some snippets from the Valence project:
// https://github.com/valence-rs/valence/blob/e3c0aec9670523cab6517ceb8a16de6d200dea62/crates/valence_core/src/packet/var_int.rs
// Valence is licensed under the MIT license.

use std::{
    borrow::Cow,
    io::{Cursor, Read},
    iter::repeat_with,
};

use byteorder::ReadBytesExt;

use bytes::Buf;
use uuid::Uuid;

use crate::{
    ver::Protocol, ArrayPrefix, Identifier, PrefixedArray, ProtoError, Result, UnprefixedByteArray,
    Varint,
};

pub trait Decodeable {
    fn decode(read: &mut Cursor<&[u8]>, ver: Protocol) -> Result<Self>
    where
        Self: Sized;
}

// Varint
impl Decodeable for Varint {
    fn decode(read: &mut Cursor<&[u8]>, _ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        let mut val: i32 = 0;
        for i in 0..5 {
            let byte = read.read_u8()?;
            val |= (byte as i32 & 0b01111111) << (i * 7);
            if byte & 0b10000000 == 0 {
                return Ok(Varint(val));
            }
        }
        Err(ProtoError::VarintError(
            "Varint is too large! Expected maximum 5 bytes long.",
        ))
    }
}

// Primitives
macro_rules! impl_decodeable_for_primitive  {
    ($(
        $ty:ident ($mtd:ident)
    ),+) => {
        $(
            impl Decodeable for $ty {
                fn decode(read: &mut Cursor<&[u8]>, _ver: Protocol) -> Result<Self>
                where
                    Self: Sized
                {
                    read.$mtd::<byteorder::BigEndian>().map_err(ProtoError::from)
                }
            }
        )+
    };
}

impl_decodeable_for_primitive!(
    u16(read_u16),
    i16(read_i16),
    u32(read_u32),
    i32(read_i32),
    u64(read_u64),
    i64(read_i64),
    f32(read_f32),
    f64(read_f64)
);

impl Decodeable for u8 {
    fn decode(read: &mut Cursor<&[u8]>, _ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        read.read_u8().map_err(ProtoError::from)
    }
}

impl Decodeable for i8 {
    fn decode(read: &mut Cursor<&[u8]>, _ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        read.read_i8().map_err(ProtoError::from)
    }
}

impl Decodeable for bool {
    fn decode(read: &mut Cursor<&[u8]>, ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        u8::decode(read, ver).map(|b| b == 0x01)
    }
}

// Strings
impl Decodeable for String {
    fn decode(read: &mut Cursor<&[u8]>, ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        let size = Varint::decode(read, ver)?.0;
        if size > 32767 {
            return Err(ProtoError::StringError(size, 32767));
        }

        let mut buf = vec![0; size as usize];
        read.read_exact(&mut buf)?;
        String::from_utf8(buf).map_err(ProtoError::from)
    }
}

// IDs
impl Decodeable for Identifier {
    fn decode(read: &mut Cursor<&[u8]>, ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        String::decode(read, ver).map(Identifier::from)
    }
}

// UUID
impl Decodeable for Uuid {
    fn decode(read: &mut Cursor<&[u8]>, _ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        let mut buf = [0; 16];
        read.read_exact(&mut buf)?;
        Ok(Uuid::from_bytes(buf))
    }
}

// Options
impl<T: Decodeable> Decodeable for Option<T> {
    fn decode(read: &mut Cursor<&[u8]>, ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        bool::decode(read, ver).map(|b| if b { T::decode(read, ver).ok() } else { None })
    }
}

// Vecs
impl<'b> Decodeable for UnprefixedByteArray<'b> {
    fn decode(read: &mut Cursor<&[u8]>, _ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        let mut remaining = vec![0; read.remaining()];
        read.read_exact(&mut remaining)?;
        Ok(Self(Cow::Owned(remaining)))
    }
}

impl<'d, V: Decodeable + Clone, P: ArrayPrefix> Decodeable for PrefixedArray<'d, V, P> {
    fn decode(read: &mut Cursor<&[u8]>, ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        let prefix = P::pfx_read(read, ver)?;
        let vec = repeat_with(|| V::decode(read, ver))
            .take(prefix)
            .collect::<Result<Vec<V>>>()?;
        Ok(P::array(Cow::Owned(vec)))
    }
}

// NBT
impl Decodeable for nbt::Blob {
    fn decode(read: &mut Cursor<&[u8]>, _ver: Protocol) -> Result<Self>
    where
        Self: Sized,
    {
        nbt::Blob::from_reader(read).map_err(ProtoError::from)
    }
}

#[cfg(test)]
mod tests {
    use std::{borrow::Cow, io::Cursor};

    use bytes::BytesMut;
    use uuid::Uuid;

    use crate::{
        ver::Protocol, ArrayPrefix, Decodeable, Encodeable, Identifier, Result,
        UnprefixedByteArray, Varint,
    };

    fn encode_decode<T: Decodeable + Encodeable>(original: T) -> Result<T> {
        let mut buf = BytesMut::new();
        original.encode(&mut buf, Protocol::latest())?;
        let mut out = Cursor::new(&buf[..]);
        T::decode(&mut out, Protocol::latest())
    }

    macro_rules! test_preserves {
        (
            $(
                    $test_name:ident($origin:expr)
            );+ $(;)?
        ) => {
            $(
                #[test]
                fn $test_name() -> Result<()> {
                    let original = $origin;
                    let decoded = encode_decode(original.clone())?;
                    assert_eq!(original, decoded);
                    Ok(())
                }
            )+
        };
    }

    test_preserves! {
        test_preserves_varint(Varint(0x12345678));
        test_preserves_string(String::from("Hello, World!"));
        test_preserves_identifier(Identifier::from("minecraft:stone"));
        test_preserves_uuid(Uuid::new_v4());

        // Primitives
        test_preserves_u8(0x12u8);
        test_preserves_i8(-0x12i8);
        test_preserves_u16(0x1234u16);
        test_preserves_i16(-0x1234i16);
        test_preserves_u32(0x12345678u32);
        test_preserves_i32(-0x12345678i32);
        test_preserves_u64(0x123456789abcdef0u64);
        test_preserves_i64(-0x123456789abcdef0i64);
        test_preserves_unprefixed(UnprefixedByteArray(Cow::Owned(vec![0, 1, 2, 3, 4, 5, 6, 7, 8])));
        test_preserves_varint_prefixed(Varint::array(Cow::Owned(vec!["Hello".to_owned(), "world!".to_owned()])));
        test_preserves_long_prefixed(u64::array(Cow::Owned(vec!["Hello".to_owned(), "world!".to_owned()])));
        test_preserves_opt_none(Option::<String>::None);
        test_preserves_opt_some(Some("Hello, world!".to_owned()));
    }
}
