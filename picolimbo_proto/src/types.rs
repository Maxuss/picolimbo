use std::{borrow::Cow, io::Cursor, marker::PhantomData, mem::size_of};

use bytes::BytesMut;
use serde::Serialize;

use crate::{ver::Protocol, Decodeable, Encodeable};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Varint(pub i32);

impl Varint {
    pub const fn size_of(value: i32) -> usize {
        match value {
            0 => 1,
            n => (31 - n.leading_zeros() as usize) / 7 + 1,
        }
    }
}

macro_rules! impl_varint_from_primitive {
    ($($ty:ident),+) => {
        $(
            impl From<$ty> for Varint {
                fn from(value: $ty) -> Self {
                    Self(value as i32)
                }
            }
        )+
    };
}

impl_varint_from_primitive!(i8, i16, i32, i64, i128, u8, u16, u32);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Identifier(pub String, pub String);

impl Identifier {
    pub fn new<N: Into<String>, P: Into<String>>(namespace: N, path: N) -> Self {
        Self(namespace.into(), path.into())
    }
}

impl<S> From<S> for Identifier
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        let str_value = value.into();
        let (namespace, path) = str_value.split_at(str_value.find(':').unwrap());
        Identifier(namespace.into(), path.trim_start_matches(':').into())
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct JsonOut<'v, T>(pub &'v T);

impl<'v, T> From<&'v T> for JsonOut<'v, T> {
    fn from(value: &'v T) -> Self {
        JsonOut(value)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct UnprefixedByteArray<'b>(pub Cow<'b, [u8]>);

pub trait ArrayPrefix {
    fn pfx_write(len: usize, out: &mut BytesMut, ver: Protocol) -> crate::Result<()>;
    fn pfx_size(len: usize) -> usize;
    fn pfx_read(read: &mut Cursor<&[u8]>, ver: Protocol) -> crate::Result<usize>;

    fn array<V: Clone>(cow: Cow<[V]>) -> PrefixedArray<V, Self>
    where
        Self: Sized;

    fn decoding<'d, V: Decodeable + Clone>(
        read: &mut Cursor<&[u8]>,
        ver: Protocol,
    ) -> crate::Result<PrefixedArray<'d, V, Self>>
    where
        Self: Sized,
    {
        PrefixedArray::<V, Self>::decode(read, ver)
    }
}

impl ArrayPrefix for Varint {
    fn pfx_write(len: usize, out: &mut BytesMut, ver: Protocol) -> crate::Result<()> {
        Varint(len as i32).encode(out, ver)
    }

    fn array<V: Clone>(base: Cow<[V]>) -> PrefixedArray<V, Self> {
        PrefixedArray(base, PhantomData)
    }

    fn pfx_size(len: usize) -> usize {
        Varint::size_of(len as i32)
    }

    fn pfx_read(read: &mut Cursor<&[u8]>, ver: Protocol) -> crate::Result<usize> {
        Varint::decode(read, ver).map(|v| v.0 as usize)
    }
}

impl ArrayPrefix for u64 {
    fn pfx_write(len: usize, out: &mut BytesMut, ver: Protocol) -> crate::Result<()> {
        (len as u64).encode(out, ver)
    }

    fn array<V: Clone>(base: Cow<[V]>) -> PrefixedArray<V, Self> {
        PrefixedArray(base, PhantomData)
    }

    fn pfx_size(_: usize) -> usize {
        size_of::<u64>()
    }

    fn pfx_read(read: &mut Cursor<&[u8]>, ver: Protocol) -> crate::Result<usize> {
        u64::decode(read, ver).map(|v| v as usize)
    }
}

impl ArrayPrefix for u16 {
    fn pfx_write(len: usize, out: &mut BytesMut, ver: Protocol) -> crate::Result<()> {
        (len as u16).encode(out, ver)
    }

    fn pfx_size(_: usize) -> usize {
        size_of::<u16>()
    }

    fn pfx_read(read: &mut Cursor<&[u8]>, ver: Protocol) -> crate::Result<usize> {
        u16::decode(read, ver).map(|v| v as usize)
    }

    fn array<V: Clone>(cow: Cow<[V]>) -> PrefixedArray<V, Self>
    where
        Self: Sized,
    {
        PrefixedArray(cow, PhantomData)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct PrefixedArray<'d, V: Clone, P: ArrayPrefix>(pub Cow<'d, [V]>, PhantomData<P>);
