use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn is_dev(&self) -> bool {
        match self {
            Environment::Development => true,
            _ => false,
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::Development
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "development" | "dev" | "local" => Ok(Self::Development),
            "production" | "prod" | "remote" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not supported environment. Use either `local` or `production`"
            )),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
        }
    }
}
