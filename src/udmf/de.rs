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

        let out = self.peek_char().and_then(|ch| Token::try_from(ch));

        match out {
            Ok(token) => {
                self.next_char().expect("valid read");
                Ok(token)
            }
            Err(_err) => {
                // try reading it as an ident
                Ok(Token::Ident(self.next_ident()?))
            }
        }
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

            Ok(Value::String(unescape_string(output)))
        } else if ch.is_ascii_digit() || matches!(ch, '+' | '-') {
            // this is the start of an unsigned/hex integer
            self.read_number()
        } else {
            // TODO: this cannot read hex numbers, but that's fine for now
            // since it's a readability thing and I don't think anyone is
            // writing udmfs by hand

            // this is a keyword
            let end = self
                .input
                .find(&['^', '{', '}', '(', ')', ';', '"', '\'', '\n', '\t', ' '])
                .unwrap_or_else(|| self.input.len());

            let keyword = &self.input[..end];
            self.input = &self.input[end..];

            match keyword {
                "true" => Ok(Value::Boolean(true)),
                "false" => Ok(Value::Boolean(false)),
                _ => Err(Error {
                    kind: ErrorKind::InvalidKeyword(keyword.to_owned()),
                }),
            }
        }
    }

    /// Returns the next identifier.
    fn next_ident(&mut self) -> Result<&'de str, Error> {
        // skip any whitespace
        //self.skip_whitespace();

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

    fn read_number(&mut self) -> Result<Value, Error> {
        // get sign
        let sign = self.peek_char()?;

        if matches!(sign, '+' | '-') {
            self.next_char().expect("remaining data");
        }

        // read until nondigit character
        let end = self
            .input
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or_else(|| self.input.len());

        let next_char = self.input[end..].chars().next();
        if next_char == Some('.') {
            // this is a float! read to end
            let end = self.input[(end + '.'.len_utf8())..]
                .find(|c: char| !c.is_ascii_digit())
                .map(|e| e + end + '.'.len_utf8())
                .unwrap_or_else(|| self.input.len());

            // got float
            let output = self.input[..end]
                .parse::<f32>()
                // the only error that can happen is if self.input == ""
                .map_err(|_| Error {
                    kind: self
                        .input
                        .chars()
                        .next()
                        .map(|c| ErrorKind::UnexpectedChar(c))
                        .unwrap_or_else(|| ErrorKind::Eof),
                })?;

            // add sign
            let output = match sign {
                '-' => output * -1.,
                _ => output,
            };

            // reset cursor
            self.input = &self.input[end..];

            match self.peek_char() {
                Ok('e') | Ok('E') => {
                    self.next_char().expect("remaining data");

                    // this is an exponential
                    let sign = self.peek_char()?;

                    if matches!(sign, '+' | '-') {
                        self.next_char().expect("remaining data");
                    }

                    // get digits
                    let end = self
                        .input
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or_else(|| self.input.len());

                    let exp = self.input[..end]
                        .parse::<i32>()
                        // the only error that can happen is if self.input == ""
                        .map_err(|_| Error {
                            kind: self
                                .input
                                .chars()
                                .next()
                                .map(|c| ErrorKind::UnexpectedChar(c))
                                .unwrap_or_else(|| ErrorKind::Eof),
                        })?;

                    let exp = match sign {
                        '-' => exp * -1,
                        _ => exp,
                    };

                    // use this as pow
                    let output = 10f32.powi(exp) * output;

                    self.input = &self.input[end..];
                    Ok(Value::Float(output))
                }
                _ => {
                    // return value as is
                    Ok(Value::Float(output))
                }
            }
        } else {
            // vomit int
            let output = self.input[..end]
                .parse::<i32>()
                // the only error that can happen is if self.input == ""
                .map_err(|_| Error {
                    kind: self
                        .input
                        .chars()
                        .next()
                        .map(|c| ErrorKind::UnexpectedChar(c))
                        .unwrap_or_else(|| ErrorKind::Eof),
                })?;

            // add sign
            let output = match sign {
                '-' => output * -1,
                _ => output,
            };

            self.input = &self.input[end..];
            Ok(Value::Integer(output))
        }
    }

    fn read_number_zero_prefix(&mut self) -> Result<Value, Error> {
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

/// Unescapes a string.
pub fn unescape_string(mut s: &str) -> String {
    let mut out = String::with_capacity(s.len());

    loop {
        let next = s.find('\\');

        if let Some(next) = next {
            // add everything up to backslash
            out.push_str(&s[..next]);

            s = &s[(next + '\\'.len_utf8())..];

            // lookup table
            let next = s.chars().next();

            match next {
                Some('"') => {
                    // for escaping quotes
                    out.push('"');
                }
                Some(ch) => {
                    // push unedited chars
                    out.push('\\');
                    out.push(ch);
                }
                None => (),
            }

            if let Some(ch) = next {
                s = &s[ch.len_utf8()..];
            }
        } else {
            out.push_str(s);
            break;
        }
    }

    out
}

/// Tokens that can be produced by [`Tokenizer`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Token<'a> {
    /// An identifier.
    Ident(&'a str),
    /// Acts as the division between an identifier and its assignment.
    Assignment,
    /// Seperator between assignments.
    Seperator,
    /// Marks the start of a block.
    StartBlock,
    /// Marks the end of a block.
    EndBlock,
}

impl<'a> TryFrom<char> for Token<'a> {
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
    InvalidKeyword(String),
    Eof,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::UnexpectedChar(ch) => write!(f, "unexpected: '{}'", ch),
            ErrorKind::UnquotedString => write!(f, "unquoted string"),
            ErrorKind::InvalidKeyword(st) => write!(f, "invalid keyword: \"{}\"", st),
            ErrorKind::Eof => write!(f, "got eof"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_float() {
        let input = r#"
        4.2
        9.99999
        +8.1
        -4.0
        2.0E-1
        4.0E9
        -2.0E-2
        "#;
        let mut input = Tokenizer::new(input);

        assert_eq!(input.next_value().unwrap(), Value::Float(4.2));
        assert_eq!(input.next_value().unwrap(), Value::Float(9.99999));
        assert_eq!(input.next_value().unwrap(), Value::Float(8.1));
        assert_eq!(input.next_value().unwrap(), Value::Float(-4.0));
        assert_eq!(input.next_value().unwrap(), Value::Float(0.2));
        assert_eq!(input.next_value().unwrap(), Value::Float(4_000_000_000.0));
        assert_eq!(input.next_value().unwrap(), Value::Float(-0.02));
    }

    #[test]
    fn read_int() {
        let input = r#"
        -10
        17
        38
        "#;
        let mut input = Tokenizer::new(input);

        assert_eq!(input.next_value().unwrap(), Value::Integer(-10));
        assert_eq!(input.next_value().unwrap(), Value::Integer(17));
        assert_eq!(input.next_value().unwrap(), Value::Integer(38));
    }

    #[test]
    fn read_string() {
        let input = r#"
        "Hey Paisanos!"
        "Welcome to the \"Super Mario Bros. Super Show\"!"
        "Do Do Do Do"
        "#;
        let mut input = Tokenizer::new(input);

        assert_eq!(
            input.next_value().unwrap(),
            Value::String("Hey Paisanos!".into())
        );
        assert_eq!(
            input.next_value().unwrap(),
            Value::String("Welcome to the \"Super Mario Bros. Super Show\"!".into())
        );
        assert_eq!(
            input.next_value().unwrap(),
            Value::String("Do Do Do Do".into())
        );
    }

    #[test]
    fn read_top_level_variables() {
        let input = r#"
        namespace = "ringracers";
        version = 1;
        "#;
        let mut input = Tokenizer::new(input);

        assert_eq!(input.next_token().unwrap(), Token::Ident("namespace"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(
            input.next_value().unwrap(),
            Value::String("ringracers".into())
        );
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
    }

    #[test]
    fn read_all() {
        // This is an example config
        let input = r#"
        namespace = "ringracers";
        version = 1;

        thing {
            x = 43.0;
            y = 459.0;
            height = 20.0;
            angle = 30;
            arg0 = "WADSUP";
            arg1 = true;
        }

        vertex {
            x = 17.0;
            y = 38.0;
        }
        "#;
        let mut input = Tokenizer::new(input);

        assert_eq!(input.next_token().unwrap(), Token::Ident("namespace"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(
            input.next_value().unwrap(),
            Value::String("ringracers".into())
        );
        assert_eq!(input.next_token().unwrap(), Token::Seperator);

        assert_eq!(input.next_token().unwrap(), Token::Ident("version"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Integer(1));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);

        assert_eq!(input.next_token().unwrap(), Token::Ident("thing"));
        assert_eq!(input.next_token().unwrap(), Token::StartBlock);
        assert_eq!(input.next_token().unwrap(), Token::Ident("x"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Float(43.0));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::Ident("y"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Float(459.0));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::Ident("height"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Float(20.0));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::Ident("angle"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Integer(30));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::Ident("arg0"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::String("WADSUP".into()));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::Ident("arg1"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Boolean(true));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::EndBlock);

        assert_eq!(input.next_token().unwrap(), Token::Ident("vertex"));
        assert_eq!(input.next_token().unwrap(), Token::StartBlock);
        assert_eq!(input.next_token().unwrap(), Token::Ident("x"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Float(17.0));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::Ident("y"));
        assert_eq!(input.next_token().unwrap(), Token::Assignment);
        assert_eq!(input.next_value().unwrap(), Value::Float(38.0));
        assert_eq!(input.next_token().unwrap(), Token::Seperator);
        assert_eq!(input.next_token().unwrap(), Token::EndBlock);
    }
}
