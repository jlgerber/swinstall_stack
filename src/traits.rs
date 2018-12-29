
use chrono::{NaiveDateTime};
use quick_xml::Reader;
use std::io::BufReader;
use std::fs::File;

pub trait SwinstallCurrent: std::fmt::Debug  {
    // this sucks. associated const are not object safe so....
    //const SCHEMA: &'static str;
    fn schema(&self) -> &'static str;

    /// retrieve the path to the current resource, given a reader that points at one or more elt tags
    /// within the swinstall_stack xml document.
    fn current(&self, reader: &mut Reader<BufReader<File>>) -> Result<String,()>;

    /// Retrieve the path to the current resource at the provided datetime, given a reader
    /// that points at one or more elt tags within the swinstall_stack xml document
    fn current_at(&self, reader: &mut Reader<BufReader<File>>, datetime: &NaiveDateTime) -> Result<String, ()>;
}