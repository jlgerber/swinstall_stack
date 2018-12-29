use crate::traits::SwinstallCurrent;
use std::io::BufReader;
use std::fs::File;
use chrono::{NaiveDateTime};
use quick_xml::Reader;
use crate::errors::SwInstallError;

#[derive(Debug)]
pub struct Two;

impl SwinstallCurrent for Two {
    type SwBufReader = BufReader<File>;

    fn schema(&self) -> &'static str {
            "2"
    }

    fn current_at(&self, reader: &mut Reader<Self::SwBufReader>, datetime: &NaiveDateTime)
        -> Result<String, SwInstallError>
    {
        Ok("/foo/bar/bla.yaml_1".to_string())
    }

}