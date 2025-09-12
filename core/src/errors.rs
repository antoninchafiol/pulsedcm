use std::{error::Error, fmt::{self, Display}};
use crate::errors::fmt::Formatter;


// ======== START DICOM ========= 
#[derive(Debug)]
pub enum DicomError {
    Read(dicom_object::ReadError),
    Write(dicom_object::WriteError),
    PixelData(dicom_pixeldata::Error),
}

impl Display for DicomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(e) => write!(f, "Read: {}", e ),
            Self::Write(e) => write!(f, "Write: {}", e ), 
            Self::PixelData(e) => write!(f, "Read: {}", e ),
        }
    }
}

impl Error for DicomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> { 
        match self { 
            Self::Read(s) => Some(s),
            Self::Write(s) => Some(s),
            Self::PixelData(s) => Some(s),
            // _ => None,
        }
    }
}
// ======== END DICOM ========= 



// ======== START MAIN ERROR KIND ========= 
#[derive(Debug)]
pub enum PulseErrorKind {
    IO(std::io::Error),
    SystemTime(std::time::SystemTimeError),
    Dicom(DicomError),
    Threading(rayon::ThreadPoolBuildError),
    ThreadPoison(String),
    ImageError(image::ImageError),
    CodecError(jp2k::err::Error),
    CSV(csv::Error),
    // Additional checks
    UnsupportedPixelData, 
    UnsupportedComponent,
}


impl Display for PulseErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(e) => write!(f, "I/O Error: {}", e),
            Self::Dicom(e) => write!(f, "Dicom Error: {}", e),
            Self::SystemTime(e) => write!(f, "System Time error: {}", e), 
            Self::Threading(e) => write!(f, "Threading Error: {}", e),
            Self::ThreadPoison(_) =>  write!(f, "Thread Poisoned"),
            Self::ImageError(e) => write!(f, "Image Error: {}", e),
            Self::CodecError(e) => write!(f, "Codec Error: {}", e),
            Self::CSV(e) => write!(f, "CSV Error: {}", e), 
            Self::UnsupportedPixelData => write!(f, "Unsupported pixel data"),
            Self::UnsupportedComponent => write!(f, "Unsupported number of components"),
        }
    }
}

impl Error for PulseErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> { 
        match self { 
            Self::IO(s) => Some(s),
            Self::Dicom(s) => Some(s),
            Self::SystemTime(s) => Some(s),
            Self::Threading(s) => Some(s),
            Self::ThreadPoison(_) => None,
            Self::ImageError(s) => Some(s),
            Self::CodecError(s) => Some(s),
            Self::CSV(s) => Some(s),


            Self::UnsupportedComponent => None,
            Self::UnsupportedPixelData => None,
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
impl From<std::io::Error> for PulseError {
    fn from(e: std::io::Error) -> Self {
        PulseError::new(PulseErrorKind::IO(e), "I/O error")
    }
}

impl From<rayon::ThreadPoolBuildError> for PulseError {
    fn from(e: rayon::ThreadPoolBuildError) -> Self {
        Self { 
            kind: PulseErrorKind::Threading(e), 
            message: "Thread pool error".to_string(), 
        }
    }
}

impl From<dicom_object::ReadError> for PulseError {
    fn from(e: dicom_object::ReadError) -> Self {
        PulseError::new(PulseErrorKind::Dicom(DicomError::Read(e)), "DICOM error")
    }
}

impl From<dicom_object::WriteError> for PulseError {
    fn from(e: dicom_object::WriteError) -> Self { Self { 
            kind: PulseErrorKind::Dicom(DicomError::Write(e)), 
            message: "DICOM Write to file error".to_string(), 
        }
    }
}



impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for PulseError {
    fn from(err: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        PulseError::new(PulseErrorKind::ThreadPoison(err.to_string()), "Mutex poisoned")
    }
}

impl From<std::time::SystemTimeError> for PulseError {
    fn from(e: std::time::SystemTimeError) -> Self { Self { 
            kind: PulseErrorKind::SystemTime(e), 
            message: "System time error".to_string(), 
        }
    }
}

impl From<csv::Error> for PulseError {
    fn from(e: csv::Error) -> Self { Self { 
            kind: PulseErrorKind::CSV(e), 
            message: "CSV error".to_string(), 
        }
    }
}

impl From<jp2k::err::Error> for PulseError {
    fn from(e: jp2k::err::Error) -> Self { Self { 
            kind: PulseErrorKind::CodecError(e), 
            message: "CSV error".to_string(), 
        }
    }
}

impl From<image::ImageError> for PulseError {
    fn from(e: image::ImageError) -> Self { Self { 
            kind: PulseErrorKind::ImageError(e), 
            message: "CSV error".to_string(), 
        }
    }
}

impl From<dicom_pixeldata::Error> for PulseError {
    fn from(e: dicom_pixeldata::Error) -> Self { Self { 
            kind: PulseErrorKind::Dicom(DicomError::PixelData(e)), 
            message: "CSV error".to_string(), 
        }
    }
}



