use super::{Error, Token, Tokenizer, Value};

use serde::de::{
    self, value::BorrowedStrDeserializer, DeserializeSeed, Error as _, MapAccess, Visitor,
};
use serde::forward_to_deserialize_any;

/// `udmf` block access.
pub struct BlockAccess<'a, 'de>(&'a mut Tokenizer<'de>);

impl<'a, 'de> MapAccess<'de> for BlockAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.0.next_token()? {
            Token::Ident(ident) => {
                let ident = seed
                    .deserialize(BorrowedStrDeserializer::new(ident))
                    .map(Some)?;

                // consume assignment
                if let Token::Assignment = self.0.next_token()? {
                    Ok(ident)
                } else {
                    // TODO: better error thing
                    Err(Error::custom("expected assignment token"))
                }
            }
            Token::EndBlock => Ok(None),
            // TODO: better error thing
            _ => Err(Error::custom("unexpected token")),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let deser = ValueDeserializer::new(&mut *self.0);
        seed.deserialize(deser)
    }
}

/// `udmf` top level deserializer.
pub struct TopLevelDeserializer<'a, 'de> {
    t: &'a mut Tokenizer<'de>,
}

impl<'a, 'de> TopLevelDeserializer<'a, 'de> {
    pub fn new(t: &'a mut Tokenizer<'de>) -> TopLevelDeserializer<'a, 'de> {
        TopLevelDeserializer { t }
    }
}

impl<'a, 'de> de::Deserializer<'de> for TopLevelDeserializer<'a, 'de> {
    type Error = Error;

    // implement main methods
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // check what type this is
        match self.t.next_token()? {
            Token::Assignment => {
                // this is a value
                ValueDeserializer::new(self.t).deserialize_any(visitor)
            }
            Token::StartBlock => {
                // this is a map
                let map_access = BlockAccess(self.t);

                visitor.visit_map(map_access)
            }
            _ => {
                // TODO error impl
                todo!()
            }
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct struct enum
        ignored_any identifier map bool str string
    }
}

/// `udmf` block deserializer.
pub struct ValueDeserializer<'a, 'de> {
    t: &'a mut Tokenizer<'de>,
}

impl<'a, 'de> ValueDeserializer<'a, 'de> {
    pub fn new(t: &'a mut Tokenizer<'de>) -> ValueDeserializer<'a, 'de> {
        ValueDeserializer { t }
    }
}

impl<'a, 'de> de::Deserializer<'de> for ValueDeserializer<'a, 'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let res = match self.t.next_value()? {
            Value::Boolean(b) => visitor.visit_bool(b),
            Value::Integer(int) => visitor.visit_i32(int),
            Value::Float(fl) => visitor.visit_f32(fl),
            Value::String(s) => visitor.visit_string(s),
            Value::Nil => visitor.visit_none(),
        };

        if let Token::Seperator = self.t.next_token()? {
            res
        } else {
            Err(Error::expected_seperator())
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char bytes byte_buf option unit
        unit_struct newtype_struct seq tuple tuple_struct struct enum
        ignored_any identifier map bool str string
    }
}
