use chrono::format::ParseError;
use failure::Fail;
use std::{
    convert::From,
    num::ParseIntError,
    str::{ Utf8Error, ParseBoolError },
};

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
    #[fail(display = "ParseBoolError - failed to parse bool: {}", _0)]
    ParseBoolError(String),
    #[fail(display = "InvalidAction - supplied unsupported action str in new: {}", _0)]
    InvalidAction(String),
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
        SwInstallError::ChronoParseError(error.to_string())
    }
}

impl From<std::num::ParseIntError> for SwInstallError {
    fn from(error: ParseIntError) -> Self {
        SwInstallError::ParseIntError(error.to_string())
    }
}

impl From<ParseBoolError> for SwInstallError {
    fn from(error: ParseBoolError) -> Self {
        SwInstallError::ParseBoolError(error.to_string())
    }
}