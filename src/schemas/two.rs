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

use chrono::{ NaiveDateTime };
use crate::{
    actions::Action,
    constants::{ DATETIME_FMT, TAG_ELT },
    errors::SwInstallError,
    schemas,
    traits::{ SwinstallCurrent, SwinstallElement  },
};
#[allow(unused_imports)]
use log::{ debug, info, warn };
use quick_xml::{
    events::{
        attributes::Attributes,
        attributes::Attribute,
        Event,
        BytesStart,
        BytesEnd,
    },
    Reader,
    Writer,
};
use std::{
    cmp::PartialEq,
    io::Cursor,
    str::from_utf8,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Elt {
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

impl SwinstallElement  for Elt {

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

    fn version(&self) -> String {
        self.version.clone()
    }
}

/// Model the elt tag contents from swinstall_log
#[derive(Debug, Eq)]
pub struct Two;

impl Two {
    pub fn new() -> Self {
        Two {}
    }
}

impl PartialEq for Two {
    fn eq(&self, other: &Two) -> bool {
        self.schema() == other.schema()
    }
}

impl SwinstallCurrent for Two {
    type SwElem = schemas::ReturnElt;

    fn schema(&self) -> &'static str {
            "2"
    }

    fn current_at<T>(&self, reader: &mut Reader<T>, datetime: &NaiveDateTime)
        -> Result<Self::SwElem, SwInstallError>
    where
        T: std::io::BufRead
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

                            return Ok(schemas::ReturnElt::Two(elt));
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

    /// Update the swinstall_stack with a new element. We assume that the outer
    /// block has already been written and we are only responsible for writing
    /// the Elements (Elt tags)
    fn update<R, W>(&self, action: Action, reader: &mut Reader<R>, writer: &mut Writer<W>, elem: Self::SwElem)
            -> Result<(), SwInstallError>
        where
        R: std::io::BufRead,
        W: std::io::Write
    {
        match action {
            Action::Install => {
                let mut cnt = 0;
                let mut buf = Vec::new();

                let elem = match elem {
                    schemas::ReturnElt::Two(e) => e,
                    _ => panic!("wrong type of ReturnELt"),
                };

                loop {
                     match reader.read_event(&mut buf) {
                        Ok(Event::Start(ref e)) => {
                            writer.write_event(Event::Start(e.to_owned())).is_ok();
                        }
                        Ok(Event::Empty(ref e)) if e.name() == TAG_ELT => {
                            if cnt == 0 {
                                // we will insert here
                                let tag_vec = TAG_ELT.to_vec();
                                let tag_len = tag_vec.len();
                                let mut bselem = BytesStart::owned(tag_vec, tag_len);
                                bselem.push_attribute(Attribute::from(("action", elem.action.as_str())));
                                bselem.push_attribute(Attribute::from( ("datetime", elem.datetime.as_str()) ));
                                bselem.push_attribute(Attribute::from(("hash", elem.hash.as_str()) ));
                                bselem.push_attribute(Attribute::from(("version", elem.version.as_str())));
                                writer.write_event(Event::Empty(bselem)).is_ok();
                                cnt +=1;
                            }

                            writer.write_event(Event::Empty(e.to_owned())).is_ok();

                            // // crates a new element ... alternatively we could reuse `e` by calling
                            // // `e.into_owned()`
                            // let mut elem = BytesStart::owned(b"my_elem".to_vec(), "my_elem".len());

                            // // collect existing attributes
                            // elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));

                            // // copy existing attributes, adds a new my-key="some value" attribute
                            // elem.push_attribute(("my-key", "some value"));

                            // // writes the event to the writer
                            // assert!(writer.write_event(Event::Start(elem)).is_ok());
                        },
                        Ok(Event::End(ref e))  => {
                            writer.write_event(Event::End(e.to_owned())).is_ok();
                        },
                        Ok(Event::Eof) => break,
                        Ok(e) => assert!(writer.write_event(e).is_ok()),
                        // or using the buffer
                        // Ok(e) => assert!(writer.write(&buf).is_ok()),
                        Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    }
                    buf.clear();
                }
                Ok(())
            }
            _ => unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn elt_from_attrs() {
       let swinstall_stack_elt_tags = r#"<elt action="install" datetime="20180702-144204" hash="194f835569a79ba433" version="3"/>"#;
       let mut reader = Reader::from_str(swinstall_stack_elt_tags);
       let mut buf = Vec::new();
       loop {
            match reader.read_event(&mut buf) {
                        Ok(Event::Empty(ref e)) => {
                            let elt = Elt::from_attrs( e.attributes()).expect("could not create elt");
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

    #[test]
    fn update_two() {
        let two = Two::new();
        let swinstall_stack_elt_tags = r#"<elt action="install" datetime="20180702-144204" hash="194f835569a79ba433" version="3"/>"#;
        // we ultimately want a call like:
        // let file = "/dd/facility/etc/packages.xml";
        // install_file(file)
        // where fn install_file()
        let mut writer = Writer::new(Cursor::new(Vec::new()));
        let mut reader = Reader::from_str(swinstall_stack_elt_tags);
        let action = Action::Install;
        let elem = Elt::new(action.to_string(), "20190101-113000".to_string(), "124a835569a79ba433".to_string(), "4".to_string());
        //
        let result = two.update( action,
            &mut reader,
            &mut writer,
            schemas::ReturnElt::Two(elem)
        );
        assert_eq!(result.unwrap(), ());
        let result = writer.into_inner().into_inner();
        let result = String::from_utf8(result).unwrap();
        let expected = r#"<elt action="install" datetime="20190101-113000" hash="124a835569a79ba433" version="4"/><elt action="install" datetime="20180702-144204" hash="194f835569a79ba433" version="3"/>"#;
        assert_eq!(result.as_str(), expected);
    }
}
