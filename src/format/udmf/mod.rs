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

use serde::de::{Deserialize, Visitor};
use serde::ser::Serialize;

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

impl Value {
    /// Gets the name of the type contained.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Boolean(_) => "boolean",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Nil => "nil",
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Boolean(v) => serializer.serialize_bool(*v),
            Value::Integer(v) => serializer.serialize_i32(*v),
            Value::Float(v) => serializer.serialize_f32(*v),
            Value::String(v) => serializer.serialize_str(v),
            Value::Nil => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'d> Visitor<'d> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("udmf value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Boolean(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Integer(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::Float(v))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value::String(v.to_owned()))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
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
