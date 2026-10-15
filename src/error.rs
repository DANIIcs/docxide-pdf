use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidDocx(String),
    Zip(zip::result::ZipError),
    Xml(roxmltree::Error),
    Pdf(String),
    Io(std::io::Error),
    RenamedEnvVar(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidDocx(reason) => write!(f, "not a valid DOCX file: {reason}"),
            Error::Zip(e) => write!(f, "ZIP error: {e}"),
            Error::Xml(e) => write!(f, "XML error: {e}"),
            Error::Pdf(e) => write!(f, "PDF error: {e}"),
            Error::Io(e) => write!(f, "IO error: {e}"),
            Error::RenamedEnvVar(old) => write!(
                f,
                "{old} is no longer read; rename it to {}",
                old.replacen("DOCXSIDE_", "DOCXIDE_", 1)
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<zip::result::ZipError> for Error {
    fn from(e: zip::result::ZipError) -> Self {
        Self::Zip(e)
    }
}

impl From<roxmltree::Error> for Error {
    fn from(e: roxmltree::Error) -> Self {
        Self::Xml(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
