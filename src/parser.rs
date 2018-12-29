//! parse the swinstall_stack xml file and invoke the appropriate SwinstallCurrent trait implementor.
//!

use std::collections::HashMap;
use crate::traits::SwinstallCurrent;

type SwinstallCurrentRegistry = HashMap<&'static str, Box<dyn SwinstallCurrent>>;

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

    /// Register a struct implementing SwinstallCurrent with registry
    pub fn register(&mut self, value: Box<dyn SwinstallCurrent>) {
        self.registry.insert(value.schema(), value);
    }

    /// Set the default schema.
    pub fn set_default_schema(&mut self, schema: String) -> bool  {

        if !self.registry.contains_key(&schema.as_str()) {
            return false;
        }

        self.default_schema = Some(schema);
        true
    }

    pub fn get(&self, schema: &str) -> Option<&Box<dyn SwinstallCurrent>> {
        self.registry.get(schema)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{NaiveDateTime};
    use quick_xml::Reader;
    use std::io::BufReader;
    use std::fs::File;
    use std::cmp::PartialEq;

    #[derive(Debug)]
    struct MyCurrent;


    impl SwinstallCurrent for MyCurrent {
        //const SCHEMA: &'static str = "1";
        fn schema(&self) -> &'static str {
            "1"
        }

        fn current(&self, reader: &mut Reader<BufReader<File>>) -> Result<String,()> {
             Ok("/foo/bar/bla.yaml_20181123-090200".to_string())
        }


        fn current_at(&self, reader: &mut Reader<BufReader<File>>, datetime: &NaiveDateTime)
            -> Result<String, ()>
        {
            Ok("/foo/bar/bla.yaml_20181124-212211".to_string())
        }
    }


    #[derive(Debug)]
    struct MyCurrent2;

    impl SwinstallCurrent for MyCurrent2 {
        //const SCHEMA: &'static str = "2";
        fn schema(&self) -> &'static str {
            "2"
        }

        fn current(&self, reader: &mut Reader<BufReader<File>>) -> Result<String,()> {
             Ok("/foo/bar/bla.yaml_20181123-090200".to_string())
        }


        fn current_at(&self, reader: &mut Reader<BufReader<File>>, datetime: &NaiveDateTime)
            -> Result<String, ()>
        {
            Ok("/foo/bar/bla.yaml_20181124-212211".to_string())
        }
    }

    #[test]
    fn register_schema() {
        let mycur = MyCurrent {};
        let mycur2 = MyCurrent2 {};
        let mut parser = SwinstallParser::new();
        parser.register(Box::new(mycur));
        parser.register(Box::new(mycur2));
        parser.set_default_schema(String::from("2"));
        assert_eq!(parser.registry.len(), 2);
    }

    #[test]
    fn get_swinstall_parser() {
        let mycur = MyCurrent {};
        let mycur2 = MyCurrent2 {};
        let mut parser = SwinstallParser::new();
        parser.register(Box::new(mycur));
        parser.register(Box::new(mycur2));

        if let Some(result) = parser.get("2") {
            assert_eq!(result.schema(), "2");
        } else {
            panic!("unable to get schema 2");
        };

        if let Some(result) = parser.get("1") {
            assert_eq!(result.schema(), "1");
        } else {
            panic!("unable to get schema 1");
        };
    }
}