use std::ops::Deref;
use std::str::FromStr;

use schemars::schema::{InstanceType, Schema, SchemaObject};
use schemars::JsonSchema;
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
    fn schema_name() -> String {
        String::from("Ulid")
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ulid".to_string()),
            ..Default::default()
        }
            .into()
    }
}