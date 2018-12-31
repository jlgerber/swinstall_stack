//! parse the swinstall_stack xml file and invoke the appropriate SwinstallCurrent trait implementor.
//!

use chrono::{ NaiveDateTime, Local };
use crate::{
    SwInstallError,
    schemas::{ReturnElt, SchemaWrapper },
    traits::{ SwinstallCurrent,  SwInstallElementWrapper, },
    utils::versioned_from_swinstall_stack,
};
use log::{debug};
use std::{
    collections::HashMap,
    //io::BufReader,
    //fs::File,
    //path::{Path,},
};
use quick_xml::{
    events::{ BytesStart, Event },
    Reader,
};

//type SwReader = Reader<BufReader<File>>;
type SwinstallCurrentRegistry = HashMap<&'static str, SchemaWrapper > ;


#[derive(Debug)]
pub struct SwinstallParser {
    // Registry hashmap storing different implementations of elt parser based on
    // version of swisntall_stack
    registry: SwinstallCurrentRegistry,
    // optional default key in case the swinstall_stack does not have a schema
    // attribute
    default_schema: Option<String>
}

impl SwinstallParser {
    /// new up an Parser
    pub fn new() -> Self {
        SwinstallParser {
            registry: SwinstallCurrentRegistry::new(),
            default_schema: None
        }
    }

    /// Register a SchemaWrapper enum tagging an implementation of SwinstallCurrent
    /// with the schema registry, which affords for handling different generations
    /// of an swinstall_stack from the same code.
    pub fn register(&mut self, value: SchemaWrapper) {
        self.registry.insert(value.schema(), value);
    }

    /// Set the default schema. This is the schema associated with the
    /// SwinstallCurrent implementation which will run by default,
    /// when the outer swinstall tag has no version attribute.
    pub fn set_default_schema(&mut self, schema: String) -> bool  {

        if !self.registry.contains_key(&schema.as_str()) {
            return false;
        }

        self.default_schema = Some(schema);
        true
    }

    /// Retrieve the SwinstallComponent registered against a paritcular schema.
    pub fn get_component(&self, schema: &str)
    -> Option<&SchemaWrapper>
    {
        self.registry.get(schema)
    }

    // retrieve the schema
    fn schema<'a>(&self,  e: &'a BytesStart) -> Result<String, SwInstallError> {
         let mut schema = self.default_schema.clone().ok_or(SwInstallError::NoDefaultSchema)?;

        // get schema  from attributes
        for attr in e.attributes() {
            let attr = attr?;
            if attr.key == b"schema" {
                schema = std::str::from_utf8(&attr.value.into_owned())?.to_string();
            }
        }
        debug!("fetching elt_reader for schema: {}", schema.as_str());

        Ok(schema)
    }

    // Get the current version as a String
    fn current_version<'a, T>(&self, reader: &mut Reader<T>, schema: &str, datetime: &NaiveDateTime)
        -> Result<ReturnElt, failure::Error>
    where
        T: std::io::BufRead
    {

        let elt_reader = self.get_component(schema).ok_or(SwInstallError::RuntimeError(format!("Unable to get reader for schema: {}", schema)))?;
        debug!("calling elt_reader.current_at(reader, {})", datetime);

        // get back the version string of the current file
        let result = elt_reader.current_at(reader, datetime)?;
        Ok(result)
    }

    /// Retrieve the path to the file marked current in the supplied swinstall_stack.
    ///
    /// ```current``` takes a boxed closure that takes a str and returns a Result which is
    /// either a quick_xml::Reader<BufRead> or an SwInstallError. It also takes the
    /// full path to the swinstall_stack as a &str.
    ///
    /// The boxed closure input is provided to facilitate testing. However, there is a
    /// default closure which may be generated by calling ```utils::reader_from_file_fn()```
    pub fn current<T>(&self, readfn: Box<Fn(&str) -> Result<Reader<T>,SwInstallError>>, swinstall_stack: &str)
        -> Result<String, failure::Error>
    where
        T: std::io::BufRead
    {
        let dt = Local::now().naive_local();
        self.current_at(readfn, swinstall_stack, &dt)
    }

    /// Retrieve the path to the file marked current as close to but not later
    /// than the supplied datetime.
    ///
    /// ```current_at``` takes a boxed closure that takes a str and returns a Result which is
    /// either a quick_xml::Reader<BufRead> or an SwInstallError. It also takes the
    /// full path to the swinstall_stack as a &str and a NaiveDateTime reference.
    ///
    /// The boxed closure input is provided to facilitate testing. However, there is a
    /// default closure which may be generated by calling ```utils::reader_from_file_fn()```
    pub fn current_at<T>(&self,readfn: Box<Fn(&str) -> Result<Reader<T>,SwInstallError>>, swinstall_stack: &str, datetime: &NaiveDateTime)
    -> Result<String, failure::Error>
    where
        T: std::io::BufRead
    {
        //let mut reader = Reader::from_file(Path::new(swinstall_stack))?;
        let mut reader = readfn(swinstall_stack)?;
        let mut buf = Vec::new();

        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    if e.name() == b"stack_history" {
                        // get schema version
                        let schema = self.schema(&e)?;

                        debug!("current_at - calling self.current_version(...)");
                        // we find a current file or we error
                        let version_string = self.current_version(&mut reader, schema.as_str(), datetime)?.version();
                        // we construct the full path to the versioned file out of the full path to the swinstall_stack
                        // and the version_string
                        let versioned_file = versioned_from_swinstall_stack(swinstall_stack, version_string.as_str())?;
                        return Ok(versioned_file);
                    }
                },
                // we never found stack_history
                Ok(Event::Eof) => {
                    return Err(SwInstallError::NoCurrentFound)?
                }, // exits the loop when reaching end of file
                Err(e) => return Err(e)?,
                _ => {}, // There are several other `Event`s we do not consider here
            }

            // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
            buf.clear();
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::schemas::{
        one::One,
        two::Two
    };
    const SCHEMA1_XML: &'static str =
r#"<stack_history path="/dd/facility/etc/bak/packages.xml/packages.xml_swinstall_stack">
   <elt is_current="False" version="20181220-090624"/>
   <elt is_current="False" version="20181220-090616"/>
   <elt is_current="False" version="20181220-090608"/>
   <elt is_current="False" version="20181220-090333"/>
   <elt is_current="True" version="20161213-093146_r575055"/>
   <elt is_current="False" version="20181220-091955"/>
   <elt is_current="False" version="20181220-092031"/>
</stack_history>"#;

const SCHEMA2_XML: &'static str =
r#"<?xml version="1.0" encoding="UTF-8"?>
<stack_history path="/Users/jonathangerber/src/rust/swinstall_stack/examples/schema2/bak/packages.xml/packages.xml_swinstall_stack" schema="2">
   <elt action="install" datetime="20181221-142313" hash="c618755af9b63728411bc536d2c60cf2" version="5"/>
   <elt action="install" datetime="20181221-142248" hash="5c8fdabe2ae7fa9287c0672b88ef6593" version="4"/>
   <elt action="rollback" datetime="20181221-102242" hash="294fc86579b14b7d39" version="1"/>
   <elt action="rollback" datetime="20181221-102344" hash="c94f6266789a483a43" version="2"/>
   <elt action="install" datetime="20180702-144204" hash="194f835569a79ba433" version="3"/>
   <elt action="install" datetime="20180101-103813" hash="c94f6266789a483a43" version="2"/>
   <elt action="install" datetime="20171106-104603" hash="294fc86579b14b7d39" version="1"/>
</stack_history>"#;

    //use chrono::{NaiveDateTime};
    //use quick_xml::Reader;
    //use std::io::BufReader;
    //use std::fs::File;

    fn setup_parser() -> SwinstallParser {
        let mycur = One {};
        let mycur2 = Two {};
        let mut parser = SwinstallParser::new();
        parser.register(SchemaWrapper::One(mycur));
        parser.register(SchemaWrapper::Two(mycur2));
        parser.set_default_schema(String::from("1"));
        parser
    }

    #[test]
    fn register_schema() {
        let parser = setup_parser();
        assert_eq!(parser.registry.len(), 2);
    }

    #[test]
    fn get_swinstall_parser() {
        let parser = setup_parser();
        if let Some(result) = parser.get_component("2") {
            assert_eq!(result.schema(), "2");
        } else {
            panic!("unable to get schema 2");
        };

        if let Some(result) = parser.get_component("1") {
            assert_eq!(result.schema(), "1");
        } else {
            panic!("unable to get schema 1");
        };
    }

    #[test]
    fn get_parser_current_schema1() {
        let parser = setup_parser();
        let result = parser.current(Box::new(|swinstall: &str| {
          Ok(quick_xml::Reader::from_str(SCHEMA1_XML))
        }), "/dd/facility/etc/bak/packages.xml/packages.xml_swinstall_stack").unwrap();
        assert_eq!(result.as_str(), "/dd/facility/etc/bak/packages.xml/packages.xml_20161213-093146_r575055");
    }


    #[test]
    fn get_parser_current_schema2() {
        let parser = setup_parser();
        let result = parser.current(Box::new(|swinstall: &str| {
          Ok(quick_xml::Reader::from_str(SCHEMA2_XML))
        }), "/dd/facility/etc/bak/packages.xml/packages.xml_swinstall_stack").unwrap();
        assert_eq!(result.as_str(), "/dd/facility/etc/bak/packages.xml/packages.xml_5");
    }
}