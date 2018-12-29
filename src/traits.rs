
use chrono::{NaiveDateTime, Local};
use quick_xml::Reader;
use std::io::BufReader;
use std::fs::File;
use crate::errors::SwInstallError;

pub trait SwinstallCurrent: std::fmt::Debug  {
    type SwBufReader;

    // this sucks. associated const are not object safe so....
    //const SCHEMA: &'static str;
    fn schema(&self) -> &'static str;

    /// retrieve the version string of the current resource, given a reader that points at one or more elt tags
    /// within the swinstall_stack xml document.
    fn current(&self, reader: &mut Reader<Self::SwBufReader>) -> Result<String, SwInstallError>
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
        -> Result<String, SwInstallError>
    where
        <Self as SwinstallCurrent>::SwBufReader: std::io::BufRead;
}