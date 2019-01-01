use crate::errors::SwInstallError;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Action {
    Install,
    Rollback,
    Rollforward,
}

impl Action {
    fn from_str(action: &str) -> Result<Action, SwInstallError> {
        match action {
            "install" => Ok(Action::Install),
            "rollback" => Ok(Action::Rollback),
            "rollforward" => Ok(Action::Rollforward),
            _ => Err(SwInstallError::InvalidAction(action.to_string()))
        }
    }
}