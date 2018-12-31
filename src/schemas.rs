pub mod one;
pub mod two;
use crate::traits::SwInstallElement;
use crate::errors::SwInstallError;
use quick_xml::events::attributes::Attributes;

/// Work around for Object Safety issues with associated types.
/// I introduced this enum to allow us to return a full structure
/// as opposed to a string.
#[derive(Debug, PartialEq, Eq)]
pub enum ReturnElt {
    One(one::Elt),
    Two(two::Elt),
}

impl SwInstallElement for ReturnElt {

    fn from_attrs<'a>(version: &str, attrs: Attributes<'a>) -> Result<Self, SwInstallError> {
        match version {
            "1" => Ok(ReturnElt::One(one::Elt::from_attrs("1", attrs)?)),
            "2" => Ok(ReturnElt::Two(two::Elt::from_attrs("2", attrs)?)),
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

