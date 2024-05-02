//! Map/course format readers.

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::format::udmf::{self, Value};

/// Extra fields.
pub type Extras = HashMap<String, Value>;

/// A single map.
///
/// Stores all information about the map in continguous memory. This does not
/// include textures or any other fun things!
#[derive(Clone, Debug, Default)]
pub struct Map {
    namespace: String,
    version: i32,
    things: Vec<Thing>,
    linedefs: Vec<LineDef>,
    sidedefs: Vec<SideDef>,
    sectors: Vec<Sector>,
    vertices: Vec<Vertex>,
    extras: Extras,
}

impl Map {
    /// Reads a map from a string.
    pub fn from_str(str: &str) -> Result<Map, udmf::de::Error> {
        #[derive(Default)]
        struct PartialMap {
            namespace: Option<String>,
            version: Option<i32>,
            things: Vec<Thing>,
            linedefs: Vec<LineDef>,
            sidedefs: Vec<SideDef>,
            sectors: Vec<Sector>,
            vertices: Vec<Vertex>,
            extras: Extras,
        }

        let mut map = PartialMap::default();

        // parse
        let input = preprocess(str);
        let mut parser = udmf::de::Parser::new(&input);

        while let Some(ident) = parser.next_key()? {
            match ident {
                "namespace" => {
                    map.namespace = Some(parser.next_value()?);
                }
                "version" => {
                    map.version = Some(parser.next_value()?);
                }
                "thing" => {
                    map.things.push(parser.next_value()?);
                }
                "vertex" => {
                    map.vertices.push(parser.next_value()?);
                }
                "linedef" => {
                    map.linedefs.push(parser.next_value()?);
                }
                "sidedef" => {
                    map.sidedefs.push(parser.next_value()?);
                }
                "sector" => {
                    map.sectors.push(parser.next_value()?);
                }
                extra => {
                    map.extras.insert(extra.to_string(), parser.next_value()?);
                }
            }
        }

        // create from partial
        Ok(Map {
            namespace: map
                .namespace
                .ok_or_else(|| udmf::de::Error::missing_field("namespace"))?,
            version: map
                .version
                .ok_or_else(|| udmf::de::Error::missing_field("version"))?,
            linedefs: map.linedefs,
            sidedefs: map.sidedefs,
            vertices: map.vertices,
            things: map.things,
            sectors: map.sectors,
            extras: map.extras,
        })
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
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Thing {
    pub x: f32,
    pub y: f32,
    #[serde(default)]
    pub height: Option<f32>,
    pub angle: i32,
    #[serde(rename = "type")]
    pub kind: i32,
    #[serde(flatten)]
    pub extras: Extras,
}

/// A single vertex on the map.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    #[serde(flatten)]
    pub extras: Extras,
}

/// A line definition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LineDef {
    pub v1: i32,
    pub v2: i32,
    #[serde(rename = "sidefront")]
    pub side_front: i32,
    #[serde(rename = "sideback", default)]
    pub side_back: Option<i32>,
    #[serde(rename = "twosided", default)]
    pub two_sided: bool,
    #[serde(flatten)]
    pub extras: Extras,
}

/// A side definition.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SideDef {
    #[serde(rename = "offsetx", default)]
    pub offset_x: i32,
    #[serde(rename = "offsety", default)]
    pub offset_y: i32,
    pub sector: i32,
    #[serde(flatten)]
    pub extras: Extras,
}

/// A sector.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Sector {
    #[serde(rename = "heightfloor", default)]
    pub height_floor: i32,
    #[serde(rename = "heightceiling", default)]
    pub height_ceiling: i32,
    #[serde(rename = "texturefloor")]
    pub texture_floor: String,
    #[serde(rename = "textureceiling")]
    pub texture_ceiling: String,
    #[serde(flatten)]
    pub extras: Extras,
}
