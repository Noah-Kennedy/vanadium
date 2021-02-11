use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::option::Option::Some;
use std::str::FromStr;

use regex::Regex;

#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Headers {
    /// The number of bands per image file.
    pub bands: usize,

    /// The order of the bytes in integer, long integer, 64-bit integer, unsigned 64-bit integer,
    /// floating point, double precision, and complex data types.
    ///
    /// Use one of the following:
    /// * Byte order=0 (Host (Intel) in the Header Info dialog) is least significant byte first
    /// (LSF) data (DEC and MS-DOS systems).
    /// * Byte order=1 (Network (IEEE) in the Header Info dialog) is most significant byte first
    /// (MSF) data (all other platforms).
    pub byte_order: FileByteOrder,

    /// The type of data representation:
    /// * 1 = Byte: 8-bit unsigned integer
    /// * 2 = Integer: 16-bit signed integer
    /// * 3 = Long: 32-bit signed integer
    /// * 4 = Floating-point: 32-bit single-precision
    /// * 5 = Double-precision: 64-bit double-precision floating-point
    /// * 6 = Complex: Real-imaginary pair of single-precision floating-point
    /// * 9 = Double-precision complex: Real-imaginary pair of double precision floating-point
    /// * 12 = Unsigned integer: 16-bit
    /// * 13 = Unsigned long integer: 32-bit
    /// * 14 = 64-bit long integer (signed)
    /// * 15 = 64-bit unsigned long integer (unsigned)
    pub data_type: DataType,

    /// The ENVI-defined file type, such as a certain data format and processing result.
    /// The available file types are listed in the filetype.txt file (see File Type File).
    /// The file type ASCII string must exacly match an entry in the filetype.txt file, including
    /// the proper case.
    pub file_type: String,

    /// The number of bytes of embedded header information present in the file.
    /// ENVI skips these bytes when reading the file. The default value is 0 bytes.
    pub header_offset: usize,

    /// Refers to whether the data
    /// (interleave)[https://www.harrisgeospatial.com/docs/enviimagefiles.html] is BSQ, BIL, or BIP.
    pub interleave: Interleave,

    /// The number of lines per image for each band.
    pub lines: usize,

    /// The number of samples (pixels) per image line for each band.
    pub samples: usize,

    /// Any nonmandatory fields
    pub other: HashMap<String, String>,
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Fields
///////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct ParseFieldError;

impl Display for ParseFieldError {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Invalid")
    }
}

impl Error for ParseFieldError {}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Interleave {
    Bip,
    Bil,
    Bsq,
}


impl FromStr for Interleave {
    type Err = ParseFieldError;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bip" => Ok(Self::Bip),
            "bil" => Ok(Self::Bil),
            "bsq" => Ok(Self::Bsq),
            _ => Err(ParseFieldError)
        }
    }
}

impl ToString for Interleave {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn to_string(&self) -> String {
        match self {
            Interleave::Bip => "bip".to_owned(),
            Interleave::Bil => "bil".to_owned(),
            Interleave::Bsq => "bsq".to_owned(),
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    U8 = 1,
    I16 = 2,
    I32 = 3,
    F32 = 4,
    F64 = 5,
    Complex32 = 6,
    Complex64 = 9,
    U16 = 12,
    U32 = 13,
    I64 = 14,
    U64 = 15,
}

impl FromStr for DataType {
    type Err = ();

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Self::U8),
            "2" => Ok(Self::I16),
            "3" => Ok(Self::I32),
            "4" => Ok(Self::F32),
            "5" => Ok(Self::F64),
            "6" => Ok(Self::Complex32),
            "9" => Ok(Self::Complex64),
            "12" => Ok(Self::U16),
            "13" => Ok(Self::U32),
            "14" => Ok(Self::I64),
            "15" => Ok(Self::U64),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FileByteOrder {
    Intel = 0,
    Network = 1,
}

impl FromStr for FileByteOrder {
    type Err = ();

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Self::Intel),
            "1" => Ok(Self::Network),
            _ => Err(())
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Errors
///////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub enum ParseHeaderError {
    BadValue(&'static str),
    /// Not used
    _NoKey(usize),
    NoValue(usize),
    /// Not used
    _DuplicateKey(String),
    RequiredFieldNotFound(&'static str),
    /// Not used
    _InvalidFirstLine,
}

impl Display for ParseHeaderError {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseHeaderError::BadValue(e) => {
                writeln!(f, "Failed to parse field '{}'.", e)
            }
            ParseHeaderError::_NoKey(line) => {
                writeln!(f, "Key not found for line {}!", line)
            }
            ParseHeaderError::NoValue(line) => {
                writeln!(f, "Value not found for line {}!", line)
            }
            ParseHeaderError::_DuplicateKey(k) => {
                writeln!(f, "Duplicate key {}!", k)
            }
            ParseHeaderError::RequiredFieldNotFound(k) => {
                writeln!(f, "Required field not found {}!", k)
            }
            ParseHeaderError::_InvalidFirstLine => {
                writeln!(f, "First line should be 'ENVI'!")
            }
        }
    }
}

impl Error for ParseHeaderError {}

const REGEX_STR: &str = "(?m:^(?P<key>.+) = (?:(?:(?P<value>.+)\n)|(?:\\{(?P<value2>.+)\\}\n)))";


impl FromStr for Headers {
    type Err = ParseHeaderError;

    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline(never))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields_map = HashMap::with_capacity(32);

        let regex = Regex::new(REGEX_STR).unwrap();

        for (i, cap) in regex.captures_iter(s).enumerate() {
            let key = &cap["key"];

            let value = if let Some(v) = cap.name("value") {
                v.as_str()
            } else if let Some(v) = cap.name("value2") {
                v.as_str()
            } else {
                return Err(ParseHeaderError::NoValue(i));
            };

            fields_map.insert(key.to_lowercase(), value.to_owned());
        }

        Ok(Self {
            bands: parse_scalar_field(&mut fields_map, "bands")?,
            byte_order: parse_scalar_field(&mut fields_map, "byte order")?,
            data_type: parse_scalar_field(&mut fields_map, "data type")?,
            file_type: parse_scalar_field(&mut fields_map, "file type")?,
            header_offset: parse_scalar_field(&mut fields_map, "header offset")?,
            interleave: parse_scalar_field(&mut fields_map, "interleave")?,
            lines: parse_scalar_field(&mut fields_map, "lines")?,
            samples: parse_scalar_field(&mut fields_map, "samples")?,
            other: fields_map,
        })
    }
}

#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline(never))]
fn parse_scalar_field<T>(
    fields_map: &mut HashMap<String, String>,
    field: &'static str,
) -> Result<T, ParseHeaderError>
    where T: FromStr
{
    fields_map.remove(field)
        .map(Ok)
        .unwrap_or(Err(ParseHeaderError::RequiredFieldNotFound(field)))?
        .parse()
        .map_err(|_| ParseHeaderError::BadValue("bands"))
}