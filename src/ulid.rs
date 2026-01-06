use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;

use schemars::{json_schema, JsonSchema, Schema};
pub use ulid::DecodeError;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ulid(ulid::Ulid);

impl Ulid {
    // Add methods as needed
    pub fn new() -> Self {
        Ulid(ulid::Ulid::new())
    }
}

impl FromStr for Ulid {
    type Err = DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = ulid::Ulid::from_str(s)?;
        Ok(Ulid(value))
    }
}

impl Deref for Ulid {
    type Target = ulid::Ulid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl JsonSchema for Ulid {
    fn schema_name() -> Cow<'static, str> {
        "Ulid".into()
    }

    fn json_schema(_gen: &mut schemars::SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "format": "ulid",
        })
    }
}
