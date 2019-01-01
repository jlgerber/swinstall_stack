//! Utilities
//!
//! Standalone helper functions
//!

use crate::errors::SwInstallError;
use std::path::{ PathBuf };

/// Given the path to a versionless swinstalled file, get the path to
/// the swinstall_stack.
pub fn swinstall_stack_from_versionless(filepath: &str) -> Result<String,SwInstallError> {
    let  pb = PathBuf::from(filepath);

    let file_name = pb.file_name()
                      .ok_or(SwInstallError::NoFileNameFromPath)?
                      .to_str()
                      .ok_or(SwInstallError::ConvertOsStrFail)?;

    let mut pb = pb.to_path_buf();

    pb.pop(); // pop off the file name since we dont need it in the path
    pb.push("bak");
    pb.push(file_name);
    pb.push(format!("{}_swinstall_stack", file_name));

    let result = pb.to_str()
      .ok_or(SwInstallError::Utf8Error(filepath.to_string()))?.to_string();

    Ok(result)
}

/// Given a filepath to a versionless swinstalled file, and a str representing a specific version
/// whose makeup is determined by the swinstall_stack schema, construct a full path to a
/// versioned file
pub fn versioned_from_versionless(filepath: &str, version: &str) -> Result<String,SwInstallError> {
    let  pb = PathBuf::from(filepath);

    let file_name = pb.file_name()
                      .ok_or(SwInstallError::NoFileNameFromPath)?
                      .to_str()
                      .ok_or(SwInstallError::ConvertOsStrFail)?;

    let mut pb = pb.to_path_buf();

    pb.pop(); // pop off the file name since we dont need it in the path
    pb.push("bak");
    pb.push(file_name);
    pb.push(format!("{}_{}", file_name, version));

    let result = pb.to_str()
      .ok_or(SwInstallError::Utf8Error(filepath.to_string()))?.to_string();

    Ok(result)
}

/// Given the full path to the swinstall_stack and a version string, construct the full path to
/// the versioned swinstalled file.
pub fn versioned_from_swinstall_stack(filepath: &str, version: &str) -> Result<String,SwInstallError> {
    let  mut pb = PathBuf::from(filepath);
    pb.pop(); // remove swinstall_stack
    // get versionless directory name
    let file_name = pb.file_name()
                      .ok_or(SwInstallError::NoFileNameFromPath)?
                      .to_str()
                      .ok_or(SwInstallError::ConvertOsStrFail)?;

    let mut pb = pb.to_path_buf();

    pb.pop(); // pop off the file name since we dont need it in the path
    pb.push(file_name);
    pb.push(format!("{}_{}", file_name, version));

    let result = pb.to_str()
      .ok_or(SwInstallError::Utf8Error(filepath.to_string()))?.to_string();

    Ok(result)
}

/// Generate the default closure for reading an xml file
pub fn reader_from_file_fn() -> Box<Fn(&str)
    -> Result<quick_xml::Reader<std::io::BufReader<std::fs::File>>, SwInstallError>>

{
    Box::new(
        |swinstall_stack: &str| {
            Ok(quick_xml::Reader::from_file(std::path::Path::new(swinstall_stack))?)
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swinstall_stack_from_versionless_file() {
        let path_str = "/dd/facility/etc/packages.xml";
        let expected = "/dd/facility/etc/bak/packages.xml/packages.xml_swinstall_stack";
        let path = swinstall_stack_from_versionless(path_str);
        assert_eq!(path.unwrap(), expected);
    }
    #[test]
    fn versioned_file_from_versionless_file() {
        let path_str = "/dd/facility/etc/packages.xml";
        let expected = "/dd/facility/etc/bak/packages.xml/packages.xml_0002";
        let path = versioned_from_versionless(path_str, "0002");
        assert_eq!(path.unwrap(), expected);
    }

    #[test]
    fn versioned_file_from_swinstall_stack() {
        let path_str = "/dd/facility/etc/bak/packages.xml/packages.xml_swinstall_stack";
        let expected = "/dd/facility/etc/bak/packages.xml/packages.xml_0002";
        let path = versioned_from_swinstall_stack(path_str, "0002");
        assert_eq!(path.unwrap(), expected);
    }
}