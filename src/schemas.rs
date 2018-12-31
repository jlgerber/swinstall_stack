pub mod one;
pub mod two;
use crate::traits::{
    SwinstallCurrent,
    SwInstallElementWrapper,
    SwInstallElement,
};
use chrono::NaiveDateTime;
use crate::errors::SwInstallError;
use std::{
    cmp::PartialEq,
    fs::File,
    io::BufReader,
    str::from_utf8,
};
use quick_xml::{
    Reader,
    events::{ attributes::Attributes, Event, },
};
/// Work around for Object Safety issues with associated types.
/// I introduced this enum to allow us to return a full structure
/// as opposed to a string.
#[derive(Debug, PartialEq, Eq)]
pub enum ReturnElt {
    One(one::Elt),
    Two(two::Elt),
}

impl SwInstallElementWrapper for ReturnElt {

    fn from_attrs<'a>(version: &str, attrs: Attributes<'a>) -> Result<Self, SwInstallError> {
        match version {
            "1" => Ok(ReturnElt::One(one::Elt::from_attrs( attrs)?)),
            "2" => Ok(ReturnElt::Two(two::Elt::from_attrs(attrs)?)),
            _ => Err(SwInstallError::RuntimeError(String::from("unable to instantiate Elt")))
        }
    }

    fn version(&self) -> String {
        match *self {
            ReturnElt::One(ref e) => e.version(),
            ReturnElt::Two(ref e) => e.version(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SchemaWrapper {
    One(one::One),
    Two(two::Two),
}


impl SwinstallCurrent for SchemaWrapper {
    type SwBufReader = BufReader<File>;
    type SwElem = ReturnElt;

    fn schema(&self) -> &'static str {
        match *self {
            SchemaWrapper::One(ref one) => one.schema(),
            SchemaWrapper::Two(ref two) => two.schema(),
        }
    }

    fn current_at(&self, reader: &mut Reader<Self::SwBufReader>, datetime: &NaiveDateTime)
        -> Result<Self::SwElem, SwInstallError>
    where
        <Self as SwinstallCurrent>::SwBufReader: std::io::BufRead
    {
        match *self {
            SchemaWrapper::One(ref one) => one.current_at(reader, datetime),
            SchemaWrapper::Two(ref two) => two.current_at(reader, datetime),
        }
    }
}