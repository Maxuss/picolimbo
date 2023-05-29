#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Varint(pub i32);

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