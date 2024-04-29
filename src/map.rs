//! Map/course format readers.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use peg::error::ParseError;
use peg::str::LineCol;

// This is the actual grammar that this uses to read TEXTMAP lumps.
// Man I love Pegging
//
// This PEG definition is lifted from udmf 1.1
// https://github.com/rheit/zdoom/blob/master/specs/udmf.txt
peg::parser! {
    grammar textmap_parser() for str {
        rule keyword() -> String
            = k:$([^ '{' | '}' | '(' | ')' | ';' | '"' | '\'' | '\n' | '\t' | ' ']+) { k.to_owned() }

        rule boolean() -> Value
            = b:keyword()
        {?
            match b.as_str() {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err("expected boolean"),
            }
        }

        rule nil() -> Value
            = n:keyword()
        {?
            match n.as_str() {
                "nil" => Ok(Value::Nil),
                _ => Err("expected nil"),
            }
        }

        rule string_character() -> char
            = [^'"']

        /*
        rule string_contents() -> Value
            = s:$([^ '"' | '\\']*("\\" [_] [^ '"' | '\\']*)*)
            //= s:$(['A'..='Z' | 'a'..='z' | '0'..='9']*)
        {?
            let s = s.to_owned();

            // TODO: escape sequences

            Ok(Value::String(s))
        }*/

        rule quoted_string() -> Value
            = "\"" out:$(string_character()*) "\""
        { Value::String(out.to_owned()) }

        rule float() -> Value
            = sign:$(['+' | '-'])? float:$(['0'..='9']+ "." ['0'..='9']*)
        {?
            // TODO: exponential forms
            let float = float.parse::<f32>().or(Err("expected float"))?;

            let sign = match sign {
                Some("+") | None => 1.0,
                Some("-") => -1.0,
                _ => return Err("valid float"),
            };

            Ok(Value::Float(float * sign))
        }

        rule integer() -> Value
            = integer_natural() / integer_zero_start() / integer_hex()

        rule integer_natural() -> Value
            = sign:$(['+' | '-'])? number:$(['1'..='9']+['0'..='9']*)
        {?
            let number = number.parse::<i32>().or(Err("expected i32"))?;

            let sign = match sign {
                Some("+") | None => 1,
                Some("-") => -1,
                _ => return Err("valid float"),
            };

            Ok(Value::Integer(number * sign))
        }

        rule integer_zero_start() -> Value
            = number:$("0" ['0'..='9']*)
        {? number.parse::<i32>().or(Err("expected i32")).map(Value::Integer) }

        rule integer_hex() -> Value
            = number:$("0x" ['0'..='9' | 'A'..='F' | 'a'..='f']+)
        {?
            // skip 0x
            let number = &number[2..];

            i32::from_str_radix(number, 16).or(Err("expected hex i32")).map(Value::Integer)
        }

        rule value() -> Value
            = float() / integer() / quoted_string() / boolean()

        rule value_and_nil() -> Value
            = value() / nil()

        rule identifier() -> String
            = i:$(['A'..='Z' | 'a'..='z']+ ['A'..='Z' | 'a'..='z' | '0'..='9' | '_']*)
        { i.to_owned() }

        rule _() = quiet!{[' ' | '\n' | '\t']*}

        rule assignment_expr() -> (String, Value)
            = ident:identifier() _ "=" _ value:value_and_nil() _ ";"
        { (ident, value) }

        rule expr_list() -> HashMap<String, Value>
            = l:(assignment_expr() ** _)
        {
            l.into_iter().collect::<HashMap<String, Value>>()
        }

        rule block() -> Global
            = ident:identifier() _ "{" _  contents:expr_list() _ "}"
        {
            Global::Block(Block { ident, contents })
        }

        rule global_assignment() -> Global
            = a:assignment_expr()
        { Global::Assign(a.0, a.1) }

        rule global_expr() -> Global
            = global_assignment() / block()

        rule global_expr_list() -> Vec<Global>
            = global_expr() ** _

        pub rule udmf() -> Vec<Global>
            = _ out:global_expr_list() _
        { out }
    }
}

/// A single map from text format.
///
/// Stores all information about the map in continguous memory. This does not
/// include textures or any other fun things!
#[derive(Clone, Debug, Default)]
pub struct Map {
    things: Vec<Thing>,
    vertices: Vec<Vertex>,
    extras: Extras,
}

impl Map {
    /// Reads a map from a string.
    pub fn from_str(str: &str) -> Result<Map, Error> {
        // parse
        let input = preprocess(str);
        let parsed = textmap_parser::udmf(&input).map_err(Error::Parse)?;

        let mut map = Map::default();

        // load assignments
        for global in parsed.into_iter() {
            match global {
                Global::Assign(name, value) => {
                    map.extras.insert(name, value);
                }
                Global::Block(block) if block.ident == "thing" => {
                    let contents = Extras::new(block.contents);

                    map.things.push(Thing {
                        x: contents.get_float("x")?,
                        y: contents.get_float("y")?,
                        height: contents.get_optional_float("height")?,
                        angle: contents.get_integer("angle")?,
                        kind: contents.get_integer("type")?,
                        extras: contents.without(&["x", "y", "height", "angle", "type"]),
                    });
                }
                Global::Block(block) if block.ident == "vertex" => {
                    let contents = Extras::new(block.contents);

                    map.vertices.push(Vertex {
                        x: contents.get_float("x")?,
                        y: contents.get_float("y")?,
                        extras: contents.without(&["x", "y"]),
                    });
                }
                Global::Block(_block) => {
                    // unknown block
                    // TODO
                }
            }
        }

        Ok(map)
    }
}

fn preprocess(input: &str) -> String {
    // remove comments
    // TODO: Multilines
    let preprocessed = input.split("\n").map(|s| {
        if let Some(idx) = s.find("//") {
            &s[..idx]
        } else {
            s
        }
    });
    let mut output = String::with_capacity(input.len());

    for line in preprocessed {
        output.push_str(line);
        output.push_str("\n");
    }

    output
}

/// A thing.
///
/// I didn't name this.
#[derive(Clone, Debug)]
pub struct Thing {
    x: f32,
    y: f32,
    height: Option<f32>,
    angle: i32,
    kind: i32,
    extras: Extras,
}

/// A single vertex on the map.
#[derive(Clone, Debug)]
pub struct Vertex {
    x: f32,
    y: f32,
    extras: Extras,
}

/// A single block.
struct Block {
    ident: String,
    contents: HashMap<String, Value>,
}

enum Global {
    Block(Block),
    Assign(String, Value),
}

/// A type for extras.
#[derive(Clone, Debug, Default)]
pub struct Extras(HashMap<String, Value>);

impl Extras {
    /// Creates a new `Extras`.
    pub fn new(base: HashMap<String, Value>) -> Extras {
        Extras(base)
    }

    /// Gets a value as a float.
    pub fn get_float(&self, name: &str) -> Result<f32, ValueError> {
        self.0
            .get(name)
            .ok_or_else(|| ValueError::MissingField(name.to_owned()))
            .and_then(|value| match value {
                Value::Float(f) => Ok(*f),
                _ => Err(ValueError::InvalidType(value.type_name())),
            })
    }

    /// Gets a value as an optional float.
    pub fn get_optional_float(&self, name: &str) -> Result<Option<f32>, ValueError> {
        self.0
            .get(name)
            .map(|value| match value {
                Value::Float(f) => Ok(*f),
                _ => Err(ValueError::InvalidType(value.type_name())),
            })
            .transpose()
    }

    /// Gets a value as an integer.
    pub fn get_integer(&self, name: &str) -> Result<i32, ValueError> {
        self.0
            .get(name)
            .ok_or_else(|| ValueError::MissingField(name.to_owned()))
            .and_then(|value| match value {
                Value::Integer(f) => Ok(*f),
                _ => Err(ValueError::InvalidType(value.type_name())),
            })
    }

    /// Excludes all the names passed.
    pub fn without(mut self, names: &[&str]) -> Self {
        for &name in names {
            self.0.remove(name);
        }

        self
    }
}

impl Deref for Extras {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Extras {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// An error for maps.
#[derive(Debug)]
pub enum Error {
    Parse(ParseError<LineCol>),
    Value(ValueError),
}

impl From<ValueError> for Error {
    fn from(value: ValueError) -> Self {
        Error::Value(value)
    }
}

/// An error for accessing [`Extras`] with failable operations.
#[derive(Debug)]
pub enum ValueError {
    MissingField(String),
    InvalidType(&'static str),
}

/// A self-describing value.
#[derive(Clone, Debug)]
pub enum Value {
    /// Boolean (true or false).
    Bool(bool),
    /// A string.
    String(String),
    /// An integer number.
    Integer(i32),
    /// A float number.
    Float(f32),
    /// Nil
    Nil,
}

impl Value {
    /// The name of the type.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Bool(_) => "boolean",
            Value::String(_) => "string",
            Value::Integer(_) => "integer",
            Value::Float(_) => "float",
            Value::Nil => "nil",
        }
    }
}
