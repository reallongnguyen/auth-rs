use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use uuid::Uuid;

#[derive(
  juniper::GraphQLScalarValue, Debug, Hash, Eq, PartialEq, Serialize, Clone, Deserialize,
)]
pub struct Id(String);

impl Id {
  pub fn new(value: String) -> Self {
    Id(value)
  }

  pub fn create_uuid_v4() -> Self {
    Self::new(Uuid::new_v4().to_string())
  }
}

impl FromStr for Id {
  type Err = anyhow::Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self::new(s.to_string()))
  }
}

impl fmt::Display for Id {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}
