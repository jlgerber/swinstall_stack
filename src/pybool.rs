use crate::errors::SwInstallError;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Pybool {
    True,
    False,
}

impl Pybool {
    /// Generate a Pybool from a bool
    pub fn new(value: bool) -> Self {
        match value {
            true => Pybool::True,
            false => Pybool::False,
        }
    }

    /// Generate a pybool from a str
    pub fn from_str(value: &str) -> Result<Self, SwInstallError> {
        match value {
            "True" | "true" => Ok(Pybool::True),
            "False" | "false" => Ok(Pybool::False),
            _ => Err(SwInstallError::RuntimeError(format!("invalid input: {}", value)))
        }
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            Pybool::True => true,
            Pybool::False => false,
        }
    }
}

impl ToString for Pybool {
    fn to_string(&self) -> String {
        match *self {
            Pybool::True => String::from("True"),
            Pybool::False => String::from("False"),
        }
    }
}

impl From<bool> for Pybool {
    fn from(t_or_f: bool) -> Self {
        Pybool::new(t_or_f)
    }
}