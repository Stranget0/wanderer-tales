use std::error::Error;

#[derive(Debug)]
pub enum BindGroupBuilderError<K: std::fmt::Display> {
    NoBufferFound(K),
    NoBindTypeFound(K),
}
impl<K: std::fmt::Display> std::fmt::Display for BindGroupBuilderError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoBufferFound(name) => write!(f, "No buffer found with name {name}"),
            Self::NoBindTypeFound(name) => write!(f, "No bind type found with name {name}"),
        }
    }
}
impl<K: std::fmt::Display> BindGroupBuilderError<K> {
    pub fn no_buffer_found(name: K) -> Self {
        Self::NoBufferFound(name)
    }
    pub fn no_bind_type_found(name: K) -> Self {
        Self::NoBindTypeFound(name)
    }
}

impl<K: std::fmt::Display + std::fmt::Debug> Error for BindGroupBuilderError<K> {}
