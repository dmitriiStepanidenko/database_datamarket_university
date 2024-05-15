use crate::thing_wrapper::ObjectWithThing;
use async_graphql::{InputValueError, Scalar, ScalarType};

use serde::{Deserialize, Serialize};
use surrealdb::sql::{thing, Id, Thing};

/// This is just ThingWrapper
/// Values passed in and out in format of string: 'TABLE:UUID'
/// In fufuture, maybe, will add another format
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThingDerived(Thing);

//scalar!(ThingDerived, "Thing", "Thing value in format: 'TABLE:UUID'");

#[Scalar(name = "Thing")]
impl ScalarType for ThingDerived {
    fn parse(value: async_graphql::Value) -> async_graphql::InputValueResult<Self> {
        match value {
            async_graphql::Value::String(val) => Ok(Self(thing(&val)?)),
            _ => Err(InputValueError::expected_type(value)),
        }
    }

    // TODO
    //fn is_valid(_value: &async_graphql::Value) -> bool {
    //
    //}

    fn to_value(&self) -> async_graphql::Value {
        async_graphql::Value::String(self.0.to_string())
    }
}

impl From<ThingDerived> for String {
    fn from(value: ThingDerived) -> Self {
        value.0.to_string()
    }
}

impl From<Thing> for ThingDerived {
    fn from(value: Thing) -> Self {
        Self(value)
    }
}

impl Serialize for ThingDerived {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ThingDerived {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(ThingDerived(Thing::deserialize(deserializer)?))
    }
}

impl ThingDerived {
    pub fn id(&self) -> Id {
        self.0.id.clone()
    }
    pub fn tb(&self) -> String {
        self.0.tb.clone()
    }
}

impl std::fmt::Display for ThingDerived {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ObjectWithThing for ThingDerived {
    fn thing(&self, _: &dyn common::ctx::Ctx) -> common::ApiResult<Thing> {
        Ok(self.0.clone())
    }
}

pub fn thing_into_string(value: Thing) -> String {
    value.to_string()
}
