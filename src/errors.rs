use failure::Error;
use failure::Fail;
use std::convert::From;

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

}

impl From<quick_xml::Error> for SwInstallError {
    fn from(error: quick_xml::Error) -> Self {
        SwInstallError::QuckXmlError(error.to_string())
    }
}