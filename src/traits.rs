//! Part of swinstall_stack
//!
//! traits.rs
//!
//! Define traits used in the project. Currently, there is one: `SwinstallCurrent`.
//!
//! `SwinstallCurrent` defines the interface for introspecting swinstall_stack xml
//! files; speficically, for looping over a number of elt tags, parsing them via
//! quick-xml, and identifying the contents.
//!
//! There are three major responsibilities of this trait:
//!
//! - identifying the schema version of the swinstall_stack xml file
//! - retrieving the current swinstalled file tracked in the swinstall_stack
//! - retrieving the file swinstalled on the date and time closest to but not
//!   exceeding that provided by the user
//!
//! Because swinstall_stack maintains a registry of SwinstallCurrent trait objects,
//! allowing us to parse multiple different schema versions from the same runtime,
//! identified at runtime via the outer *stack_history's schema_version* attribute,
//! there are a number of constraints imposed by Rusts notion of object safety. These
//! include disallowing trait objects from using Generic parameters. Thus, we are forced
//! to define generic parameters in terms of an associated type, SwBufReader.
//!
//! Unfortunately, this has a side effect of being unable to test the crate with
//! xml strings. We have to create actual xml files and feed them to the tests. Not a
//! big deal, but a bit of a pain.
//!
//! Another approach might have been to define the different schema structs as an enum,
//! but I didn't want to pattern match against each enum branch for each elt tag,
//! as the each xml file should have a uniform elt tag structure based on its schema.

use chrono::{NaiveDateTime, Local};
use crate::errors::SwInstallError;
use quick_xml::Reader;
use std::fmt::Debug;
use quick_xml::events::attributes::Attributes;

/// This trait targets the enum which wraps each of the schema return Elements and is
/// used to help circumvent issues with Object Safety.
pub trait SwInstallElementWrapper: Debug + PartialEq + Eq + Sized {
    fn from_attrs<'a>(version: &str, attrs: Attributes<'a>) -> Result<Self, SwInstallError>;
    fn version(&self) -> String;
}

/// This trait defines common interface for the Elt element which represents
/// an entry in the swinstall_stack for a specific schema.
pub trait SwInstallElement: Debug + PartialEq + Eq + Sized {
    fn from_attrs<'a>(attrs: Attributes<'a>) -> Result<Self, SwInstallError>;
    fn version(&self) -> String;
}

pub trait SwinstallCurrent: std::fmt::Debug + std::cmp::PartialEq + Eq {
    type SwBufReader;
    type SwElem: SwInstallElementWrapper;

    // this sucks. associated const are not object safe so....
    //const SCHEMA: &'static str;
    fn schema(&self) -> &'static str;

    /// retrieve the version string of the current resource, given a reader that points at one or more elt tags
    /// within the swinstall_stack xml document.
    fn current(&self, reader: &mut Reader<Self::SwBufReader>) -> Result<Self::SwElem, SwInstallError>
    where <Self as SwinstallCurrent>::SwBufReader: std::io::BufRead {
        let now =  Local::now().naive_local();
        self.current_at(reader, &now)
    }

    /// Retrieve the version string of the current resource at the provided datetime, given a reader
    /// that points at one or more elt tags within the swinstall_stack xml document.
    ///
    /// The version string may differ per implementation. For instance, in schema 2, the version string
    /// is a number. In schema 1, it is a str made up of datetime components: YYYYMMDD-HHMMSS
    ///
    /// It is the job of the surrounding code to turn the version string into a full path to
    /// the versioned file.
    fn current_at(&self, reader: &mut Reader<Self::SwBufReader>, datetime: &NaiveDateTime)
        -> Result<Self::SwElem, SwInstallError>
    where
        <Self as SwinstallCurrent>::SwBufReader: std::io::BufRead;
}
