use crate::traits::SwinstallCurrent;
use std::io::BufReader;
use std::fs::File;
use chrono::{NaiveDateTime};
use quick_xml::Reader;
use crate::errors::SwInstallError;

#[derive(Debug)]
pub struct One;

impl SwinstallCurrent for One {
    type SwBufReader = BufReader<File>;

    fn schema(&self) -> &'static str {
            "1"
    }

    fn current_at(&self, reader: &mut Reader<Self::SwBufReader>, datetime: &NaiveDateTime)
        -> Result<String, SwInstallError>
    {
        Ok("/foo/bar/bla.yaml_20181124-212211".to_string())
    }

}