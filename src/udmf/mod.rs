//! `udmf` serialization framework.
//!
//! ## High Level
//! For higher level access with [`serde`] batteries included, see:
//! * **Deserialization**  
//!   [`de::Parser`]
//!
//! ## Low Level
//! For lower level access:
//! * **Deserialization**  
//!   [`de::Tokenizer`]

pub mod de;

/// `udmf` value type.
///
/// Since `udmf` is a self-describing format, we can represent every possible
/// value type below.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A boolean is stored.
    Boolean(bool),
    /// An integer is stored.
    Integer(i32),
    /// A float is stored.
    Float(f32),
    /// A string is stored.
    String(String),
    /// A nil.
    Nil,
}

impl From<bool> for Value {
    fn from(value: bool) -> Value {
        Value::Boolean(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Integer(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Value::Float(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl<'a> From<&'a str> for Value {
    fn from(value: &'a str) -> Self {
        Value::String(value.to_owned())
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(t) => t.into(),
            None => Value::Nil,
        }
    }
}
