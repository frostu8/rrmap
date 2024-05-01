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
