//!
//! one.rs
//!
//! Implementation of traits::SwinstallCurrent
//! This module provides the code to operate on schema v1
//! swinstall_stack xml stores.
//!
//! # Details
//!
//! swinstall_store xml files are  stored, along with versioned files,
//! in a special directory structure which lives along side swinstalled
//! files on disk.
//!
//! For a given file, <filname>.<ext>, witin a parent directory, swinstall
//! creates a ```bak``` directory. Within the ```bak``` directory, swinstall
//! creates a ```<filename>.<ext>``` directory. Within this directory, swinstall
//! places both versioned file installs and the swinstall_stack xml file to
//! journal installations.
//!
//! swinstall_store files obey the following naming convention:
//! ```ignore
//! <filename>.<ext>_swinstall_store
//! ```
//!
//! versioned files are named thusly:
//!
//! ```ignore
//! <filename>.<ext>_<version>
//! ```
//!
//! # V1 swinstall_stack xml example
//!
//! ```xml
//! <stack_history path="/dd/facility/etc/bak/packages.xml/packages.xml_swinstall_stack">
//!   <elt is_current="False" version="20181220-090624"/>
//!   <elt is_current="False" version="20181220-090616"/>
//!   <elt is_current="False" version="20181220-090608"/>
//!   <elt is_current="False" version="20181220-090333"/>
//!   <elt is_current="True" version="20161213-093146_r575055"/>
//!   <elt is_current="False" version="20181220-091955"/>
//!   <elt is_current="False" version="20181220-092031"/>
//! </stack_history
//! ```
//!
//! # Problems with this design
//!
//! There are a number of issues with this original schema design:
//!
//! - Rollbacks / Rollforwards alter is_current settings in the stack without
//!   recording change dates resulting in lossy history. One cannot reconstruct the
//!   sequence of events which resulted in the current state if rollbacks have occured.
//! - new versions are appended to the end of stack_history, making non-pathological
//!   use cases take O(n) time for lookups (bad design)
//! - version stores both a date-time stamp and an optional VCS revision id
//!

use chrono::{ NaiveDateTime };
use crate::constants::DATETIME_FMT;
use crate::errors::SwInstallError;
use crate::traits::{ SwinstallCurrent, SwinstallElement  };
use crate::schemas;
use std::{
    cmp::PartialEq,
   // fs::File,
   // io::BufReader,
    str::{ FromStr, from_utf8, }
};
#[allow(unused_imports)]
use log::{debug, info, warn};
use quick_xml::{
    events::{attributes::Attributes, Event, },
    Reader,
};

/// Model the elt tag contents from swinstall_log
#[derive(Debug, PartialEq, Eq)]
pub struct Elt {
    pub is_current: bool,
    pub version: String,
    pub revision: Option<String>,
}

impl Elt {
    pub fn new(is_current: bool, version: String) -> Self {
        let mut pieces: Vec<String> = version.split("_").map(|x| x.to_string()).collect();
        let revision = if pieces.len() == 2 { pieces.pop() } else { None };
        let version = pieces.pop().unwrap_or("10000101-010101".to_string());
        Elt {
            is_current, version, revision
        }
    }
}
impl SwinstallElement  for Elt {

    fn from_attrs<'a>( attrs: Attributes<'a>) -> Result<Elt, SwInstallError> {
        let mut is_current = None;
        let mut version = None;

        for attr in attrs {
            let attr = attr?;
            match attr.key {
                b"is_current"   => is_current = Some(attr.value),
                b"version"  => version = Some(attr.value),
                _ => {},
            }
        }

        // breaking this down, each component (action, datetime, etc) is
        // approximately a Option<Cow<[u8]>>. For each component, we need to
        //    extract from the Option (ok_or(...))
        //    convert to a vec<u8> ( into_owned())
        //    convert to a str (from_utf8)
        //    convert to a String (to_string)
        let elt = Elt::new(
            bool::from_str(
                from_utf8(
                    &is_current
                    .ok_or(SwInstallError::MissingEltAttribute)?
                    .into_owned()
                )?
                .to_string()
                .to_lowercase()
                .as_str()
            )?,
            from_utf8(
                &version
                .ok_or(SwInstallError::MissingEltAttribute)?
                .into_owned()
            )?
            .to_string(),
        );
        debug!("Elt::from_attrs(...) -> {:?}", elt);
        Ok(elt)
    }

    fn version(&self) -> String {
        let revision = match self.revision {
            Some(ref r) => format!("_{}",r),
            None => String::from(""),
        };
        format!("{}{}", self.version, revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elt_from_attrs() {
       let str_from = r#"<elt is_current="True" version="20161213-093146"/>"#;
       let mut reader = Reader::from_str(str_from);
       let mut buf = Vec::new();
       loop {
            match reader.read_event(&mut buf) {
                        Ok(Event::Empty(ref e)) => {
                            let elt = Elt::from_attrs( e.attributes()).expect("could not create elt");
                            let expected = Elt {
                                is_current: true,
                                version: "20161213-093146".to_string(),
                                revision: None
                            };
                            assert_eq!(elt, expected);
                            break;
                        }
                        _ => {}
            }
        }
    }

    #[test]
    fn elt_from_attrs_with_revision() {
       let str_from = r#"<elt is_current="True" version="20161213-093146_r575055"/>"#;
       let mut reader = Reader::from_str(str_from);
       let mut buf = Vec::new();
       loop {
            match reader.read_event(&mut buf) {
                        Ok(Event::Empty(ref e)) => {
                            let elt = Elt::from_attrs( e.attributes()).expect("could not create elt");
                            let expected = Elt {
                                is_current: true,
                                version: "20161213-093146".to_string(),
                                revision: Some("r575055".to_string())
                            };
                            assert_eq!(elt, expected);
                            break;
                        }
                        _ => {}
            }
        }
    }
}

#[derive(Debug, Eq)]
pub struct One;

impl One {
    pub fn new() -> Self {
        One {}
    }
}

impl PartialEq for One {
    fn eq(&self, other: &One) -> bool {
        self.schema() == other.schema()
    }
}

impl SwinstallCurrent for One {
    type SwElem = schemas::ReturnElt;

    fn schema(&self) -> &'static str {
            "1"
    }

    fn current_at<T>(&self, reader: &mut Reader<T>, datetime: &NaiveDateTime)
        -> Result<Self::SwElem, SwInstallError>
    where
        T: std::io::BufRead
    {
        debug!("one::One.current_at called");
        let mut buf = Vec::new();
        let mut current=false;
        let mut in_datetime = false;
        let mut last_elt = None;
        // for some reason, this complains that in_empty is never read
        // even though it is used in the inner scope and must be in
        // this outer scope for lifetime reasons.
        #[allow(unused_assignments)]
        let mut in_empty = false;
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    in_empty = true;
                    debug!("current_at - Event::Empty");
                    if e.name() == b"elt" {
                        debug!("current_at - Event::Empty - elt tag matched");
                        let elt = Elt::from_attrs( e.attributes())?;
                        debug!("current_at - Event::Empty - Elt::from_attrs returned");
                        let version_str = elt.version.as_str();
                        debug!("current_at - Event::Empty - passing {} to NaiveDateTime::parse_from_str", version_str);
                        let dt = NaiveDateTime::parse_from_str(version_str, DATETIME_FMT)?;
                        // update loop state variables
                        in_datetime =  dt <= *datetime;
                        current = elt.is_current ;
                        debug!("current_at - Event::Empty - state vars: <in_datetime: {} current: {}>", in_datetime, current);
                        // we only update the last_elt if we are in the valid datetime range
                        // as specified by the user.
                        if in_datetime {
                            last_elt = Some(elt);
                        }
                    }
                },
                // we never found stack_history
                Ok(Event::Eof) => {
                    debug!("current_at - Event::Eof");
                    return Err(SwInstallError::NoCurrentFound)?
                }, // exits the loop when reaching end of file
                Err(e) => { return Err(e)? },
                _ => {
                    in_empty = false;
                    debug!("current_at - other tag found");
                }, // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
            // two cases for leaving early
            // 1 - we are current this iteration, and we are within the datetime range
            // 2 - we are not in the datetime range. (presumably we were the prior loop)
            if in_empty && ((current && in_datetime) || !in_datetime) {
                match last_elt {
                    Some( elt) => {
                        return Ok(schemas::ReturnElt::One(elt));
                        // return match elt.revision {
                        //     Some(ref r) => Ok(format!("{}_{}", elt.version, r)),
                        //     None => return Ok(elt.version.clone()),
                        // };

                    }
                    None => {
                        return Err(SwInstallError::NoCurrentFound)?
                    }
                }
            }
        }
        // Err(SwInstallError::NoCurrentFound)?
    }

}