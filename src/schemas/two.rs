use crate::traits::SwinstallCurrent;
use std::io::BufReader;
use std::fs::File;
use chrono::{NaiveDateTime};
use quick_xml::Reader;
use crate::errors::SwInstallError;
use quick_xml::events::attributes::Attributes;
use std::str::from_utf8;
use crate::constants::DATETIME_FMT;
use quick_xml::events::Event;

/*
Version 2 schema
<?xml version="1.0" encoding="UTF-8"?>
<stack_history path="/Users/jonathangerber/src/python/swinstall_proposal/examples/schema2/bak/packages.xml/packages.xml_swinstall_stack" schema="2">
   <elt action="install" datetime="20181221-142313" hash="c618755af9b63728411bc536d2c60cf2" version="5"/>
   <elt action="install" datetime="20181221-142248" hash="5c8fdabe2ae7fa9287c0672b88ef6593" version="4"/>
   <elt action="rollback" datetime="20181221-102242" hash="294fc86579b14b7d39" version="1"/>
   <elt action="rollback" datetime="20181221-102242" hash="c94f6266789a483a43" version="2"/>
   <elt action="install" datetime="20180702-144204" hash="194f835569a79ba433" version="3"/>
   <elt action="install" datetime="20180101-103813" hash="c94f6266789a483a43" version="2"/>
   <elt action="install" datetime="20171106-104603" hash="294fc86579b14b7d39" version="1"/>
</stack_history>
*/

#[derive(Debug)]
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

    pub fn from_attrs<'a>(attrs: Attributes<'a>) -> Result<Elt, SwInstallError> {
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
        Ok( Elt::new(
            from_utf8(&action.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
            from_utf8(&datetime.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
            from_utf8(&hash.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
            from_utf8(&version.ok_or(SwInstallError::MissingEltAttribute)?.into_owned())?.to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn elt_from_attrs() {

    }
}

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
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name() == b"elt" {
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

        Ok("/foo/bar/bla.yaml_1".to_string())
    }
}