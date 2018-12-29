use failure::Error;
use failure::Fail;
use std::convert::From;
use std::str::Utf8Error;
use chrono::format::ParseError;

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
    #[fail(display = "failed to convert to utf8")]
    Utf8Error,
    #[fail(display = "chrono parse error: {}", _0)]
    ChronoParseError(String),
}

impl From<quick_xml::Error> for SwInstallError {
    fn from(error: quick_xml::Error) -> Self {
        SwInstallError::QuckXmlError(error.to_string())
    }
}

impl From<Utf8Error> for SwInstallError {
    fn from(error: Utf8Error) -> Self {
        SwInstallError::Utf8Error
    }
}

impl From<ParseError> for SwInstallError {
    fn from(error: ParseError) -> Self {
        SwInstallError::ChronoParseError(error.cause().unwrap().to_string())
    }
}