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
        Identifier(namespace.into(), path.into())
    }
}
