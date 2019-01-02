use crate::errors::SwInstallError;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum Action {
    Install,
    Rollback,
    Rollforward,
}

impl Action {
    pub fn from_str(action: &str) -> Result<Action, SwInstallError> {
        match action {
            "install" => Ok(Action::Install),
            "rollback" => Ok(Action::Rollback),
            "rollforward" => Ok(Action::Rollforward),
            _ => Err(SwInstallError::InvalidAction(action.to_string()))
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Action::Install => "install".to_string(),
            Action::Rollback => "rollback".to_string(),
            Action::Rollforward => "rollforward".to_string()
        }
    }

}

