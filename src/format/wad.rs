//! Lower level WAD stuff.

use std::fmt::{self, Debug, Formatter};
use std::io::{self, Read, Seek, SeekFrom};

/// Allows a type to be read as bytes.
///
/// Useful for a strictly defined structure like WADs.
pub trait ByteRead: Sized {
    /// Reads the type from an IO stream.
    fn read<R>(r: R) -> Result<Self, Error>
    where
        R: Read + Seek;
}

/// Represents an in-memory WAD file.
///
/// WAD files are typically small enough so this isn't insane.
#[derive(Clone, Debug)]
pub struct Wad {
    header: Header,
    lump_infos: Vec<LumpInfo>,
    lump_data: Vec<LumpData>,
}

impl Wad {
    /// Reads a WAD file from a reader.
    pub fn from_reader<R>(mut r: R) -> Result<Wad, Error>
    where
        R: Read + Seek,
    {
        let header = Header::read(&mut r)?;

        let lump_infos = LumpInfo::read_of(&mut r, &header)?;
        let lump_data = LumpData::read_of(&mut r, &lump_infos)?;

        Ok(Wad {
            header,
            lump_infos,
            lump_data,
        })
    }

    /// The header of the WAD.
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Gets all the lumps in the WAD as an iterator.
    pub fn lumps(&self) -> impl Iterator<Item = Lump<'_>> + '_ {
        self.lump_infos
            .iter()
            .zip(self.lump_data.iter())
            .map(|(lump_info, lump_data)| Lump {
                lump_info,
                lump_data,
            })
    }

    /// Gets a specific lump by name.
    pub fn lump(&self, name: impl AsRef<str>) -> Option<Lump> {
        let name = name.as_ref();

        self.lumps().find(|l| l.name() == name)
    }
}

/// A single immutable reference to a lump in a WAD.
pub struct Lump<'a> {
    lump_info: &'a LumpInfo,
    lump_data: &'a LumpData,
}

impl<'a> Lump<'a> {
    /// The name of the lump.
    pub fn name(&self) -> &str {
        &self.lump_info.name
    }

    /// The lump data.
    pub fn data(&self) -> &[u8] {
        self.lump_data.as_ref()
    }
}

/// The header of a WAD file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    /// The wad type.
    pub ident: WadType,
    /// How many lumps are in the wad.
    pub num_lumps: usize,
    /// Where the directory is located.
    pub info_table_offset: usize,
}

impl ByteRead for Header {
    /// Reads a header from a WAD file.
    fn read<R>(mut r: R) -> Result<Header, Error>
    where
        R: Read + Seek,
    {
        // read ident
        let mut ident = [0u8; 4];

        if r.read(&mut ident)? < 4 {
            return Err(Error::UnexpectedEof);
        }

        let ident = match &ident {
            b"IWAD" => WadType::Iwad,
            b"PWAD" => WadType::Pwad,
            _ => {
                return Err(Error::InvalidWadType(
                    String::from_utf8_lossy(&ident).into_owned(),
                ))
            }
        };

        Ok(Header {
            ident,
            num_lumps: i32::read(&mut r)? as usize,
            info_table_offset: i32::read(&mut r)? as usize,
        })
    }
}

/// The type of WAD, `"IWAD"` or `"PWAD"`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WadType {
    Iwad,
    Pwad,
}

/// Struct describing information about a lump.
#[derive(Clone, Debug, PartialEq, Eq)]
struct LumpInfo {
    file_pos: usize,
    size: usize,
    name: String,
}

impl LumpInfo {
    /// Reads many lump infos.
    ///
    /// This seeks with the reader, but the reader's state is reset to what it
    /// originally was when passed into this function.
    pub fn read_of<R>(mut r: R, header: &Header) -> Result<Vec<LumpInfo>, Error>
    where
        R: Read + Seek,
    {
        // seek to directory
        let old_cursor = r.seek(SeekFrom::Current(0))?;
        r.seek(SeekFrom::Start(header.info_table_offset as u64))?;

        // start reading from here
        let result = (0..header.num_lumps)
            .map(|_| LumpInfo::read(&mut r))
            .collect::<Result<Vec<LumpInfo>, Error>>();

        // reset cursor
        r.seek(SeekFrom::Start(old_cursor))?;

        result
    }
}

impl ByteRead for LumpInfo {
    fn read<R>(mut r: R) -> Result<Self, Error>
    where
        R: Read + Seek,
    {
        Ok(LumpInfo {
            file_pos: i32::read(&mut r)? as usize,
            size: i32::read(&mut r)? as usize,
            name: read_string::<8, _>(&mut r)?,
        })
    }
}

/// Lump data.
///
/// Meant to be a newtype just in case we need to add more methods.
#[derive(Clone, Default)]
struct LumpData(Vec<u8>);

impl LumpData {
    /*
    /// Creates a new `LumpData`.
    pub fn new(data: impl Into<Vec<u8>>) -> LumpData {
        LumpData(data.into())
    }*/

    /// Creates an empty `LumpData`.
    pub fn empty() -> LumpData {
        LumpData::default()
    }

    /// Reads all the different lump data.
    ///
    /// The vec returned is layed out so that the index of `lump_infos` matches
    /// with their respective data.
    ///
    /// This seeks with the reader, but the reader's state is reset to what it
    /// originally was when passed into this function.
    pub fn read_of<R>(mut r: R, lump_infos: &[LumpInfo]) -> Result<Vec<LumpData>, Error>
    where
        R: Read + Seek,
    {
        // remember old location
        let old_cursor = r.seek(SeekFrom::Current(0))?;

        let result = lump_infos
            .iter()
            .map(|lump_info| {
                if lump_info.size > 0 {
                    // seek to data
                    r.seek(SeekFrom::Start(lump_info.file_pos as u64))?;

                    // read all data
                    let mut buf = (0..lump_info.size).map(|_| 0).collect::<Vec<u8>>();

                    if r.read(&mut buf)? == buf.len() {
                        Ok(LumpData(buf))
                    } else {
                        Err(Error::UnexpectedEof)
                    }
                } else {
                    // this is a virtual lump, do nothing
                    Ok(LumpData::empty())
                }
            })
            .collect::<Result<Vec<LumpData>, Error>>();

        // reset cursor
        r.seek(SeekFrom::Start(old_cursor))?;

        result
    }
}

impl Debug for LumpData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("LumpData(_)")
    }
}

impl AsRef<[u8]> for LumpData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// An error type when reading or writing WADs.
#[derive(Debug)]
pub enum Error {
    Utf8(std::str::Utf8Error),
    InvalidWadType(String),
    Io(io::Error),
    UnexpectedEof,
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

fn read_string<const N: usize, R>(mut r: R) -> Result<String, Error>
where
    R: Read,
{
    let mut bytes = [0u8; N];

    if r.read(&mut bytes)? == N {
        // remove null bytes
        let bytes = match bytes.iter().position(|&ch| ch == 0x0) {
            Some(null_idx) => &bytes[..null_idx],
            None => &bytes[..],
        };

        std::str::from_utf8(&bytes)
            .map(|s| s.to_owned())
            .map_err(Error::Utf8)
    } else {
        Err(Error::UnexpectedEof)
    }
}

// INFO: primitive ByteRead impls
impl ByteRead for i32 {
    fn read<R>(mut r: R) -> Result<i32, Error>
    where
        R: Read + Seek,
    {
        let mut bytes = [0u8; 4];

        if r.read(&mut bytes)? == 4 {
            Ok(i32::from_le_bytes(bytes))
        } else {
            Err(Error::UnexpectedEof)
        }
    }
}
