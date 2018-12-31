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
    path::{Path,},
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
    pub fn current(&self, swinstall_stack: &str) -> Result<String, failure::Error> {
        let dt = Local::now().naive_local();
        self.current_at(swinstall_stack, &dt)
    }

    /// Retrieve the path to the file marked current as close to but not later
    /// than the supplied datetime.
    pub fn current_at(&self, swinstall_stack: &str, datetime: &NaiveDateTime) -> Result<String, failure::Error> {
        let mut reader = Reader::from_file(Path::new(swinstall_stack))?;
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


// #[cfg(test)]
// mod tests {
//     use super::*;

//     use crate::schemas::one::One;

//     use chrono::{NaiveDateTime};
//     use quick_xml::Reader;
//     use std::io::BufReader;
//     use std::fs::File;

//     #[derive(Debug)]
//     struct MyCurrent;

//     impl SwinstallCurrent for MyCurrent {
//         type SwBufReader = BufReader<File>;
//         type SwElem = ReturnElt;
//         //const SCHEMA: &'static str = "1";
//         fn schema(&self) -> &'static str {
//             "1"
//         }

//         fn current(&self, reader: &mut Reader<Self::SwBufReader>) -> Result<ReturnElt, SwInstallError> {
//              Err(SwInstallError::RuntimeError("/foo/bar/bla.yaml_20181123-090200".to_string()))
//         }


//         fn current_at(&self, reader: &mut Reader<Self::SwBufReader>, datetime: &NaiveDateTime)
//             -> Result<ReturnElt, SwInstallError>
//         {
//             Err(SwInstallError::RuntimeError("/foo/bar/bla.yaml_20181124-212211".to_string()))
//         }
//     }


//     #[derive(Debug)]
//     struct MyCurrent2;

//     impl SwinstallCurrent for MyCurrent2 {
//         type SwBufReader = BufReader<File>;
//         type SwElem = ReturnElt;

//         //const SCHEMA: &'static str = "2";
//         fn schema(&self) -> &'static str {
//             "2"
//         }

//         fn current(&self, reader: &mut Reader<Self::SwBufReader>) -> Result<ReturnElt, SwInstallError> {
//              Err(SwInstallError::RuntimeError("/foo/bar/bla.yaml_20181123-090200".to_string()))
//         }


//         fn current_at(&self, reader: &mut Reader<Self::SwBufReader>, datetime: &NaiveDateTime)
//             -> Result<ReturnElt, SwInstallError>
//         {
//             Err(SwInstallError::RuntimeError("/foo/bar/bla.yaml_20181124-212211".to_string()))
//         }
//     }

//     #[test]
//     fn register_schema() {
//         let mycur = One {};
//         let mycur2 = MyCurrent2 {};
//         let mut parser = SwinstallParser::new();
//         parser.register(Box::new(mycur));
//         parser.register(Box::new(mycur2));
//         parser.set_default_schema(String::from("2"));
//         assert_eq!(parser.registry.len(), 2);
//     }

//     #[test]
//     fn get_swinstall_parser() {
//         let mycur = MyCurrent {};
//         let mycur2 = MyCurrent2 {};
//         let mut parser = SwinstallParser::new();
//         parser.register(Box::new(mycur));
//         parser.register(Box::new(mycur2));

//         if let Some(result) = parser.get_component("2") {
//             assert_eq!(result.schema(), "2");
//         } else {
//             panic!("unable to get schema 2");
//         };

//         if let Some(result) = parser.get_component("1") {
//             assert_eq!(result.schema(), "1");
//         } else {
//             panic!("unable to get schema 1");
//         };
//     }
// }