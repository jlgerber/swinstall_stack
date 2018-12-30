use failure::Error;
use failure::Fail;
use std::convert::From;
use std::str::Utf8Error;
use chrono::format::ParseError;
use std::num::ParseIntError;

#[derive(Debug, Fail)]
pub enum SwInstallError {
    #[fail(display = "quick xml error: {}", _0)]
    QuckXmlError(String),
    #[fail(display = "No default schema")]
    NoDefaultSchema,
    #[fail(display = "No current file found in swinstall_stack")]
    NoCurrentFound,
    #[fail(display = "No path in swinstall_stack")]
    NoPathInXml,
    #[fail(display = "No parent from path")]
    NoParentFromPath,
    #[fail(display = "Unable to extract file_name from path")]
    NoFileNameFromPath,
    #[fail(display = "Failed to convert OsStr to Str")]
    ConvertOsStrFail,
    #[fail(display = "Missing attribute on elt tag")]
    MissingEltAttribute,
    #[fail(display = "failed to convert to utf8: {}", _0)]
    Utf8Error(String),
    #[fail(display = "chrono parse error: {}", _0)]
    ChronoParseError(String),
    #[fail(display = "runtime error: {}", _0)]
    RuntimeError(String),
    #[fail(display = "Invalid Date: {}", _0)]
    InvalidDate(String),
    #[fail(display = "Invalid Time: {}", _0)]
    InvalidTime(String),
    #[fail(display = "ParseIntError - failed to parse int: {}", _0)]
    ParseIntError(String),
}

impl From<quick_xml::Error> for SwInstallError {
    fn from(error: quick_xml::Error) -> Self {
        SwInstallError::QuckXmlError(error.to_string())
    }
}

impl From<Utf8Error> for SwInstallError {
    fn from(error: Utf8Error) -> Self {
        SwInstallError::Utf8Error(error.to_string())
    }
}

impl From<ParseError> for SwInstallError {
    fn from(error: ParseError) -> Self {
        SwInstallError::ChronoParseError(error.cause().unwrap().to_string())
    }
}

impl From<std::num::ParseIntError> for SwInstallError {
    fn from(error: ParseIntError) -> Self {
        SwInstallError::ParseIntError(error.to_string())
    }
}