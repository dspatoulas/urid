mod ulid;

use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use schemars::schema::{InstanceType, Metadata, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};
use thiserror::Error;
use crate::ulid::Ulid;

#[derive(Debug, PartialEq, Eq, Error)]
pub enum ResourceIDError {
    #[error("Unable to decode internal Ulid: {0}")]
    UnableToDecodeUlid(ulid::DecodeError),

    #[error("Invalid resource type: {0}")]
    InvalidResourceType(String),

    #[error("Invalid ID length: {0} (expected 30)")]
    InvalidLength(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceID {
    pub resource: String,
    ulid: Ulid,
}

impl JsonSchema for ResourceID {
    fn schema_name() -> String {
        "ResourceID".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            format: Some("ResourceID".to_string()),
            metadata: Some(Box::new(Metadata {
                title: Some(String::from("ResourceID")),
                description: Some(String::from(
                    "A unique resource identifier",
                )),
                ..Default::default()
            })),
            ..Default::default()
        }
            .into()
    }
}

impl ResourceID {
    pub fn new<S: ToString>(resource: S) -> Result<Self, ResourceIDError> {
        let ulid = Ulid::new();

        let resource = resource.to_string().to_uppercase();

        Self::validate_resource(&resource)?;

        Ok(Self {
            resource,
            ulid,
        })
    }


    fn validate_resource<S: ToString>(resource: S) -> Result<(), ResourceIDError> {
        let value = resource.to_string();
        if value.len() != 4 {
            Err(ResourceIDError::InvalidResourceType(
                value,
            ))
        }
        else {
            Ok(())
        }
    }
}

impl Display for ResourceID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.resource, self.ulid.to_string())
    }
}

impl FromStr for ResourceID {
    type Err = ResourceIDError;

    fn from_str(s: &str) -> Result<Self, ResourceIDError> {
        if s.len() != 30 {
            return Err(ResourceIDError::InvalidLength(s.to_string()));
        }
        
        let resource_str = &s[..4];
        Self::validate_resource(resource_str)?;

        let ulid_str = &s[4..];
        let ulid = Ulid::from_str(ulid_str).map_err(ResourceIDError::UnableToDecodeUlid)?;

        Ok(ResourceID { resource: String::from(resource_str.to_uppercase()), ulid })
    }
}

impl Type<Postgres> for ResourceID {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        *ty == PgTypeInfo::with_name("VARCHAR")
    }
}

impl Encode<'_, Postgres> for ResourceID {
    fn encode_by_ref(
        &self,
        buf: &mut PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let value = self.to_string();
        buf.extend_from_slice(value.as_bytes());
        Ok(sqlx::encode::IsNull::No)
    }
}

impl<'r> Decode<'r, Postgres> for ResourceID {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let str_value = value.as_str()?;
        ResourceID::from_str(str_value).map_err(|e| e.into())
    }
}

impl Serialize for ResourceID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ResourceID::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use strum::{Display, EnumString};
    use super::*;

    #[test]
    fn resource_id_min_length_error() {
        let invalid_id = "FOO";

        let id = ResourceID::new(invalid_id);

        assert!(id.is_err());

        assert_eq!(id.err(), Some(ResourceIDError::InvalidResourceType(invalid_id.to_string())));
    }

    #[test]
    fn resource_id_max_length_error() {
        let invalid_id = "FOOBAR";

        let id = ResourceID::new(invalid_id);

        assert!(id.is_err());

        assert_eq!(id.err(), Some(ResourceIDError::InvalidResourceType(invalid_id.to_string())));
    }

    #[test]
    fn invalid_ulid_error() {
        let invalid_ulid = format!("USER{}", 1234);

        let result = ResourceID::from_str(&invalid_ulid);

        assert!(result.is_err());
    }

    #[test]
    fn serialize_resource_id() {
        let valid_resource = "user";

        let id = ResourceID::new(valid_resource).unwrap();

        assert_eq!(&id.to_string()[..4], valid_resource.to_uppercase());

        assert_eq!(id.resource, valid_resource.to_uppercase());
    }

    #[test]
    fn deserialize_resource_id() {
        let valid_resource = "USER";
        let valid_ulid = Ulid::new();

        let value = format!("{}{}", valid_resource, valid_ulid.to_string());

        let id = value.parse::<ResourceID>();

        assert!(id.is_ok());

        let id = id.unwrap();

        assert_eq!(id.resource, valid_resource);
        assert_eq!(id.ulid, valid_ulid);
    }

    #[test]
    fn strum_resource_id() {
        #[derive(Debug, Clone, PartialEq, Eq, EnumString, Display, JsonSchema)]
        enum ResourceIDResource {
            #[strum(serialize = "USER")]
            User,
            #[strum(serialize = "ACCT")]
            Account,
        }

        let account_resource = ResourceID::new(ResourceIDResource::Account);

        assert!(account_resource.is_ok());

        let account_resource = account_resource.unwrap();

        assert_eq!(account_resource.resource, ResourceIDResource::Account.to_string());
    }
}
