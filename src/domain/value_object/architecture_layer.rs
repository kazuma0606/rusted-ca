use serde::{Deserialize, Serialize};
use std::fmt;

use crate::shared::error::domain_error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchitectureLayer {
    Presentation,
    Application,
    Domain,
    Infrastructure,
}

impl ArchitectureLayer {
    pub fn from_string(s: &str) -> Result<Self, DomainError> {
        match s.to_lowercase().as_str() {
            "presentation" | "controller" | "api" => Ok(Self::Presentation),
            "application" | "usecase" | "service" => Ok(Self::Application),
            "domain" | "entity" | "value_object" => Ok(Self::Domain),
            "infrastructure" | "repository" | "database" => Ok(Self::Infrastructure),
            _ => Err(DomainError::InvalidValue(format!(
                "Invalid architecture layer: {}",
                s
            ))),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Presentation => "PRESENTATION".to_string(),
            Self::Application => "APPLICATION".to_string(),
            Self::Domain => "DOMAIN".to_string(),
            Self::Infrastructure => "INFRASTRUCTURE".to_string(),
        }
    }
}

impl fmt::Display for ArchitectureLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architecture_layer_from_string() {
        assert_eq!(
            ArchitectureLayer::from_string("presentation").unwrap(),
            ArchitectureLayer::Presentation
        );
        assert_eq!(
            ArchitectureLayer::from_string("APPLICATION").unwrap(),
            ArchitectureLayer::Application
        );
        assert_eq!(
            ArchitectureLayer::from_string("Domain").unwrap(),
            ArchitectureLayer::Domain
        );
        assert_eq!(
            ArchitectureLayer::from_string("infrastructure").unwrap(),
            ArchitectureLayer::Infrastructure
        );
    }

    #[test]
    fn test_architecture_layer_aliases() {
        assert_eq!(
            ArchitectureLayer::from_string("controller").unwrap(),
            ArchitectureLayer::Presentation
        );
        assert_eq!(
            ArchitectureLayer::from_string("usecase").unwrap(),
            ArchitectureLayer::Application
        );
        assert_eq!(
            ArchitectureLayer::from_string("repository").unwrap(),
            ArchitectureLayer::Infrastructure
        );
    }

    #[test]
    fn test_architecture_layer_invalid() {
        assert!(ArchitectureLayer::from_string("invalid").is_err());
    }
}