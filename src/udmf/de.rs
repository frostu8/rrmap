//! `udmf` deserialization functions and structs.

use std::fmt::{self, Display, Formatter};

use super::Value;

/// `udmf` tokenizer.
pub struct Tokenizer<'de> {
    input: &'de str,
}

impl<'de> Tokenizer<'de> {
    /// Creates a new `Tokenizer`.
    pub fn new(input: &'de str) -> Tokenizer<'de> {
        Tokenizer { input }
    }

    /// Returns the next token.
    pub fn next_token(&mut self) -> Result<Token, Error> {
        // skip any whitespace
        self.skip_whitespace();

        self.next_char().and_then(|ch| Token::try_from(ch))
    }

    /// Returns the next value.
    pub fn next_value(&mut self) -> Result<Value, Error> {
        // skip any whitespace
        self.skip_whitespace();

        let ch = self.peek_char()?;

        if ch == '"' {
            // start of string, read as string
            // eat char
            self.next_char().expect("remaining input");

            // we read until end quote
            let mut end = 0;

            while end < self.input.len() {
                let next_quote = self.input[end..].find('"');

                if let Some(idx) = next_quote {
                    // if this quote isn't escaped, we're fine
                    let char_before = self.input[end..(end + idx)].chars().last();

                    if char_before == Some('\\') {
                        // keep scanning
                        end += idx + '"'.len_utf8();
                    } else {
                        // we found an unescaped quote!
                        end += idx;
                        break;
                    }
                } else {
                    // found an unquoted string!
                    return Err(Error::unquoted_string());
                }
            }

            let output = &self.input[..end];
            // skip over quote
            self.input = &self.input[(end + '"'.len_utf8())..];

            Ok(Value::String(output.to_owned()))
        } else if ch.is_ascii_digit() {
            // this is the start of an unsigned/hex integer
            self.read_int()
        } else {
            todo!()
        }
    }

    /// Returns the next identifier.
    pub fn next_ident(&mut self) -> Result<&'de str, Error> {
        // skip any whitespace
        self.skip_whitespace();

        // get next char
        let ch = self.peek_char()?;

        if matches!(ch, 'A'..='Z' | 'a'..='z' | '_') {
            // find char where it stops
            let end = self
                .input
                .find(|c: char| !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '_'));

            if let Some(idx) = end {
                let out = &self.input[..idx];
                self.input = &self.input[idx..];

                Ok(out)
            } else {
                let out = self.input;
                self.input = "";

                Ok(out)
            }
        } else {
            Err(Error::unexpected_char(ch))
        }
    }

    fn read_int(&mut self) -> Result<Value, Error> {
        // check if this is a hex digit
        let char_after = self.input.chars().nth(1);

        if char_after == Some('x') {
            let input = &self.input[2..];

            // this is a hex digit, read until not hex digit
            let end = input
                .find(|c: char| !c.is_ascii_hexdigit())
                .unwrap_or_else(|| input.len());

            // parse hex digit
            let output = i32::from_str_radix(&input[..end], 16).expect("valid hex digits found");

            self.input = &input[end..];
            Ok(Value::Integer(output))
        } else {
            todo!()
        }
    }

    fn next_char(&mut self) -> Result<char, Error> {
        // get char
        let ch = self.peek_char()?;
        self.input = &self.input[ch.len_utf8()..];
        Ok(ch)
    }

    fn peek_char(&self) -> Result<char, Error> {
        self.input.chars().next().ok_or_else(Error::eof)
    }

    fn skip_whitespace(&mut self) {
        // get next non_whitespace character
        let next_char = self.input.find(|c: char| !c.is_ascii_whitespace());

        if let Some(idx) = next_char {
            self.input = &self.input[idx..];
        } else {
            // we have reached the end of the stream
            self.input = "";
        }
    }
}

/// Tokens that can be produced by [`Tokenizer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    /// Acts as the division between an identifier and its assignment.
    Assignment,
    /// Seperator between assignments.
    Seperator,
    /// Marks the start of a block.
    StartBlock,
    /// Marks the end of a block.
    EndBlock,
}

impl TryFrom<char> for Token {
    type Error = Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '=' => Ok(Token::Assignment),
            '{' => Ok(Token::StartBlock),
            '}' => Ok(Token::EndBlock),
            ';' => Ok(Token::Seperator),
            _ => Err(Error::unexpected_char(value)),
        }
    }
}

/// An error that occurs during deserialization.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    fn eof() -> Error {
        Error {
            kind: ErrorKind::Eof,
        }
    }

    fn unquoted_string() -> Error {
        Error {
            kind: ErrorKind::UnquotedString,
        }
    }

    fn unexpected_char(ch: char) -> Error {
        Error {
            kind: ErrorKind::UnexpectedChar(ch),
        }
    }
}

/// Inner details about the error.
#[derive(Debug)]
pub enum ErrorKind {
    UnexpectedChar(char),
    UnquotedString,
    Eof,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedChar(ch) => write!(f, "unexpected: '{}'", ch),
            ErrorKind::UnquotedString => write!(f, "unquoted string"),
            ErrorKind::Eof => write!(f, "got eof"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_hex() {
        let input = "0x005A";
        let mut input = Tokenizer::new(input);

        assert_eq!(input.next_value().unwrap(), Value::Integer(0x005A));
    }

    #[test]
    fn read_top_level_variables() {
        let input = r#"
        namespace = "ringracers";
        version = 1;
        "#;

        let mut input = Tokenizer::new(input);

        assert_eq!(input.next_ident().unwrap(), "namespace");
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(
            input.next_value().unwrap(),
            Value::String("ringracers".into())
        );
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
    }
}
