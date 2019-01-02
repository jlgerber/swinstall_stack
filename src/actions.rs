use crate::errors::SwInstallError;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Action {
    Install(String),
    Rollback(String),
    Rollforward(String),
}

impl Action {
    pub fn from_str(action: &str, version: &str) -> Result<Action, SwInstallError> {
        match action {
            "install" => Ok(Action::Install(version.to_string())),
            "rollback" => Ok(Action::Rollback(version.to_string())),
            "rollforward" => Ok(Action::Rollforward(version.to_string())),
            _ => Err(SwInstallError::InvalidAction(action.to_string()))
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Action::Install(_) => "install".to_string(),
            Action::Rollback(_) => "rollback".to_string(),
            Action::Rollforward(_) => "rollforward".to_string()
        }
    }

    pub fn version(&self) -> String {
        match self {
            Action::Install(ref s) => s.to_string(),
            Action::Rollback(ref s) => s.to_string(),
            Action::Rollforward(ref s) => s.to_string()
        }
    }
}

