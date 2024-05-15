use async_graphql::{InputValueError, Scalar, ScalarType};
use chrono::{offset::Utc, DateTime};
use serde::{Deserialize, Serialize};
use surrealdb::sql::Datetime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateTimeDerived(pub Datetime);

#[Scalar(name = "DateTime")]
impl ScalarType for DateTimeDerived {
    fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
        match value {
            async_graphql::Value::String(val) => Ok(DateTimeDerived(Datetime(
                DateTime::parse_from_rfc3339(&val)
                    .map_err(|_| InputValueError::custom("Couldn't parse"))?
                    .with_timezone(&Utc),
            ))),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    // TODO
    //fn is_valid(_value: &async_graphql::Value) -> bool {
    //
    //}

    fn to_value(&self) -> async_graphql::Value {
        async_graphql::Value::String(self.0.to_raw())
    }
}

impl From<Datetime> for DateTimeDerived {
    fn from(value: Datetime) -> Self {
        Self(value)
    }
}

impl From<DateTime<Utc>> for DateTimeDerived {
    fn from(value: DateTime<Utc>) -> Self {
        Self(Datetime(value))
    }
}

impl From<DateTimeDerived> for String {
    fn from(value: DateTimeDerived) -> Self {
        value.0.to_raw()
    }
}

impl Serialize for DateTimeDerived {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DateTimeDerived {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(DateTimeDerived(Datetime::deserialize(deserializer)?))
    }
}
