use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Eq, PartialEq, Debug)]
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
pub enum Interleave {
    Bip,
    Bil,
    Bsq,
}

impl FromStr for Interleave {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bip" => Ok(Self::Bip),
            "bil" => Ok(Self::Bil),
            "bsq" => Ok(Self::Bsq),
            _ => Err(())
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
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
pub enum FileByteOrder {
    Intel = 0,
    Network = 1,
}

impl FromStr for FileByteOrder {
    type Err = ();

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
    NoKey(usize),
    NoValue(usize),
    DuplicateKey(String),
    RequiredFieldNotFound(&'static str),
    InvalidFirstLine
}

impl Display for ParseHeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseHeaderError::BadValue(e) => {
                writeln!(f, "Failed to parse field '{}'.", e)
            }
            ParseHeaderError::NoKey(line) => {
                writeln!(f, "Key not found for line {}!", line)
            }
            ParseHeaderError::NoValue(line) => {
                writeln!(f, "Value not found for line {}!", line)
            }
            ParseHeaderError::DuplicateKey(k) => {
                writeln!(f, "Duplicate key {}!", k)
            }
            ParseHeaderError::RequiredFieldNotFound(k) => {
                writeln!(f, "Required field not found {}!", k)
            }
            ParseHeaderError::InvalidFirstLine => {
                writeln!(f, "First line should be 'ENVI'!")
            }
        }
    }
}

impl Error for ParseHeaderError {}

impl FromStr for Headers {
    type Err = ParseHeaderError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut fields_map = HashMap::with_capacity(32);

        for (number, text) in s.lines().enumerate() {
            if number == 0 {
                if text != "ENVI" {
                    return Err(ParseHeaderError::InvalidFirstLine)
                }
            } else {
                let mut split = text.split('=');

                let key = split.next()
                    .map(|x| Ok(x))
                    .unwrap_or(Err(ParseHeaderError::NoKey(number)))?
                    .trim();

                let value = split.next()
                    .map(|x| Ok(x))
                    .unwrap_or(Err(ParseHeaderError::NoValue(number)))?
                    .trim();

                fields_map.insert(key.to_lowercase(), value.to_owned());
            }
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
            other: fields_map
        })
    }
}

fn parse_scalar_field<T>(
    fields_map: &mut HashMap<String, String>,
    field: &'static str,
) -> Result<T, ParseHeaderError>
    where T: FromStr
{
    fields_map.remove(field)
        .map(|x| Ok(x))
        .unwrap_or(Err(ParseHeaderError::RequiredFieldNotFound(field)))?
        .parse::<T>()
        .map_err(|_| ParseHeaderError::BadValue("bands"))
}