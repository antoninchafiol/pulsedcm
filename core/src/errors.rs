use std::{error::Error, fmt::{self, Display}};
use crate::errors::fmt::Formatter;


// ======== START DICOM ========= 
#[derive(Debug)]
pub enum DicomError {
    Read(dicom_object::ReadError),
}

impl Display for DicomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(_) => f.write_str("Read error"),
        }
    }
}

impl Error for DicomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> { 
        match self { 
            Self::Read(source) => Some(source),
            // _ => None,
        }
    }
}
// ======== END DICOM ========= 



// ======== START MAIN ERROR KIND ========= 
#[derive(Debug)]
pub enum PulseErrorKind {
    IO(std::io::Error),
    Dicom(DicomError),
    Threading(rayon::ThreadPoolBuildError),
    CodecError(jp2k::err::Error),

}


impl Display for PulseErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "I/O Error: {}", e),
            Self::Dicom(e) => write!(f, "Dicom Error: {}", e),
            Self::Threading(e) => write!(f, "Threading Error: {}", e),
            Self::CodecError(e) => write!(f, "Codec Error: {}", e),
        }
    }
}

impl Error for PulseErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> { 
        match self { 
            Self::IO(source) => Some(source),
            Self::Dicom(source) => Some(source),
            Self::Threading(source) => Some(source),
            Self::CodecError(source) => Some(source),
        }
    }
}
// ======== END MAIN ERROR KIND ========= 


// ======== START ERROR STRUCT ==========
#[derive(Debug)]
pub struct PulseError {
    kind: PulseErrorKind,
    message: String,
}

impl PulseError {
    pub fn new(kind: PulseErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind, 
            message: message.into(),
        }
    }
}

impl Display for PulseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} : {}", self.kind, self.message )
    }
}


impl Error for PulseError {
    fn source(&self) ->  Option<&(dyn Error + 'static)>  {
        Some(&self.kind)
    }
}
// ======== END ERROR STRUCT ==========

