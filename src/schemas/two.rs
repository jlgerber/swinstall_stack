//! two.rs
//!
//! Implementation of traits::SwinstallCurrent for revised schema (version 2) of
//! swinstall_stack xml file. The redesign's goals are to:
//!
//! - make current lookups *practically* O(1) on average (by reordering)
//! - preserve installation history for rollbacks / rollforwards
//! - distinguish version number from installation timestamp to avoid the
//!   need to reinstall files for rollback / rollforward
//! - store file hash to help identify post-install mutations
//!
//! # Details
//!
//! The original swinstall_stack design (schema 1) has a number of flaws:
//! - new installations are appended to the end of a list of installations, requiring
//!   the traversal of the list for average / normal lookups. Normal running time
//!   is O(n).
//! - rollbacks / rollforwards are lossy. One cannot acurately piece together a
//!   timeline of their application. In the event of a rollback/forward, an is_current flag
//!   is simply updated in the elements on the stack.
//! - timestamp and revision id are conflated in a single field (denormalization)
//!
//!
//! # Example version 2 schema
//!
//! ```xml
//! <stack_history path="/Users/jonathangerber/src/python/swinstall_proposal/examples/schema2/bak/packages.xml/packages.xml_swinstall_stack" schema="2">
//!   <elt action="install" datetime="20181221-142313" hash="c618755af9b63728411bc536d2c60cf2" version="5"/>
//!   <elt action="install" datetime="20181221-142248" hash="5c8fdabe2ae7fa9287c0672b88ef6593" version="4"/>
//!   <elt action="rollback" datetime="20181221-102242" hash="294fc86579b14b7d39" version="1"/>
//!   <elt action="rollback" datetime="20181221-102242" hash="c94f6266789a483a43" version="2"/>
//!   <elt action="install" datetime="20180702-144204" hash="194f835569a79ba433" version="3"/>
//!   <elt action="install" datetime="20180101-103813" hash="c94f6266789a483a43" version="2"/>
//!   <elt action="install" datetime="20171106-104603" hash="294fc86579b14b7d39" version="1"/>
//! </stack_history>
//! ```

use chrono::{NaiveDateTime};
use crate::{
    constants::DATETIME_FMT,
    errors::SwInstallError,
    traits::{ SwinstallCurrent, SwInstallElement },
};
#[allow(unused_imports)]
use log::{ debug, info, warn };
use std::{
    fs::File,
    io::BufReader,
    str::from_utf8,
};
use quick_xml::{
    Reader,
    events::{ attributes::Attributes, Event, },
};

#[derive(Debug, PartialEq, Eq)]
struct Elt {
    pub action: String,
    pub datetime: String,
    pub hash: String,
    pub version: String
}

impl Elt {
    pub fn new(action: String, datetime:String, hash: String, version: String) -> Self {
        Elt {
            action, datetime, hash, version
        }
    }
}

impl SwInstallElement for Elt {

    fn from_attrs<'a>(attrs: Attributes<'a>) -> Result<Elt, SwInstallError> {
        let mut action = None;
        let mut datetime = None;
        let mut hash = None;
        let mut version = None;

        for attr in attrs {
            let attr = attr?;
            match attr.key {
                b"action"   => action = Some(attr.value),
                b"datetime" => datetime = Some(attr.value),
                b"hash"     => hash = Some(attr.value),
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
            from_utf8(&action.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
            from_utf8(&datetime.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
            from_utf8(&hash.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
            from_utf8(&version.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
        );
        debug!("elt: {:?}", elt);
        Ok(elt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
        fn elt_from_attrs() {
       let str_from = r#"<elt action="install" datetime="20180702-144204" hash="194f835569a79ba433" version="3"/>"#;
       let mut reader = Reader::from_str(str_from);
       let mut buf = Vec::new();
       loop {
            match reader.read_event(&mut buf) {
                        Ok(Event::Empty(ref e)) => {
                            let elt = Elt::from_attrs(e.attributes()).expect("could not create elt");
                            let expected = Elt {
                                action: String::from("install"),
                                datetime: "20180702-144204".to_string(),
                                hash: String::from("194f835569a79ba433"),
                                version: "3".to_string(),
                            };

                            assert_eq!(elt, expected);
                            break;
                        }
                        _ => {}
            }
        }
    }
}

/// Model the elt tag contents from swinstall_log
#[derive(Debug)]
pub struct Two;

impl Two {
    pub fn new() -> Self {
        Two {}
    }
}

impl SwinstallCurrent for Two {
    type SwBufReader = BufReader<File>;

    fn schema(&self) -> &'static str {
            "2"
    }

    fn current_at(&self, reader: &mut Reader<Self::SwBufReader>, datetime: &NaiveDateTime)
        -> Result<String, SwInstallError>
    {
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Empty(ref e)) => {
                    if e.name() == b"elt" {
                        debug!("Event::Empty - elt tag matched");
                        let elt = Elt::from_attrs(e.attributes())?;
                        let dt = NaiveDateTime::parse_from_str(elt.datetime.as_str(), DATETIME_FMT)?;
                        if dt <= *datetime {
                            return Ok(elt.version.clone());
                        }
                    }
                },
                // we never found stack_history
                Ok(Event::Eof) => {
                    return Err(SwInstallError::NoCurrentFound)?
                }, // exits the loop when reaching end of file
                Err(e) => { return Err(e)? },
                _ => {}, // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }
    }
}