use crate::service::thing_wrapper::ObjectWithThing;
use common::{
    ctx::Ctx,
    error::{ApiError, ApiResult, Error},
};
use serde::de::DeserializeOwned;
use surrealdb::sql::Thing;
use surrealdb::sql::Value;

//trait Queriable {
//    fn apply() ->
//}

pub struct Unwrapper {}

impl Unwrapper {
    pub async fn unwrapper_option_without_error<'a, R, C: surrealdb::Connection>(
        query: surrealdb::method::Query<'a, C>,
        index: usize,
        ctx: &dyn Ctx,
    ) -> ApiResult<Option<R>>
    where
        R: DeserializeOwned,
    {
        query
            .await
            .map_err(ApiError::from(ctx))?
            .take::<Option<R>>(index)
            .map_err(ApiError::from(ctx))
    }

    pub async fn unwrapper_option<'a, R, C: surrealdb::Connection>(
        query: surrealdb::method::Query<'a, C>,
        index: usize,
        err: &str,
        ctx: &dyn Ctx,
    ) -> ApiResult<R>
    where
        R: DeserializeOwned,
    {
        query
            .await
            .map_err(ApiError::from(ctx))?
            .take::<Option<R>>(index)
            .map_err(ApiError::from(ctx))?
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: "internal".to_string(),
                    id: err.to_string(),
                },
            })
    }
    pub async fn unwrapper_vec<'a, R, C: surrealdb::Connection>(
        query: surrealdb::method::Query<'a, C>,
        index: usize,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<R>>
    where
        R: DeserializeOwned,
    {
        query
            .await
            .map_err(ApiError::from(ctx))?
            .take::<Vec<R>>(index)
            .map_err(ApiError::from(ctx))
    }
}

#[derive(Debug, Clone)]
pub struct QueryResultReturn {
    pub variable: String,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub struct QueryVariable {
    pub variable: String,
}

#[derive(Debug, Clone)]
pub struct VariableNamePlaceholder {
    pub variable: String,
}

pub trait VariableName: std::fmt::Debug + Send + Sync + Clone {
    fn variable(&self) -> String;
}

impl VariableName for QueryResultReturn {
    fn variable(&self) -> String {
        self.variable.clone()
    }
}

impl VariableName for QueryVariable {
    fn variable(&self) -> String {
        self.variable.clone()
    }
}

impl VariableName for VariableNamePlaceholder {
    fn variable(&self) -> String {
        self.variable.clone()
    }
}

pub trait ToThingOrVariable<T: VariableName> {
    fn thing_or_variable(&self) -> ThingOrVariable<T>;
}

impl ToThingOrVariable<QueryResultReturn> for QueryResultReturn {
    fn thing_or_variable(&self) -> ThingOrVariable<QueryResultReturn> {
        ThingOrVariable::Variable(self.clone())
    }
}

impl ToThingOrVariable<QueryVariable> for QueryVariable {
    fn thing_or_variable(&self) -> ThingOrVariable<QueryVariable> {
        ThingOrVariable::Variable(self.clone())
    }
}

//impl<T> ToThingOrVariable for std::pin::Pin<&T>
//where
//    T: ObjectWithThing,
//{
//    fn thing_or_variable<'a>(&self) -> ThingOrVariable<'a> {
//        ThingOrVariable::Thing(self.clone())
//    }
//}

#[derive(Clone, Debug)]
pub enum ThingOrVariable<T: VariableName> {
    Thing(Thing),
    Variable(T),
}

impl ToThingOrVariable<VariableNamePlaceholder> for Thing {
    fn thing_or_variable(&self) -> ThingOrVariable<VariableNamePlaceholder> {
        ThingOrVariable::Thing(self.clone())
    }
}

impl<T: VariableName> ObjectWithThing for ThingOrVariable<T> {
    fn thing(&self, ctx: &dyn Ctx) -> ApiResult<Thing> {
        match self {
            Self::Thing(thing) => Ok(thing.clone()),
            Self::Variable(_) => Err(Error::Generic {
                description: "Error while converting from ThingOrVariable to Thing".to_string(),
            })
            .map_err(ApiError::from(ctx)),
        }
    }
}

impl<T: VariableName> ThingOrVariable<T> {
    pub fn bind(
        self,
        query: QueryBuilderHelper,
        variable: Option<String>,
    ) -> Result<QueryBuilderHelper, Error> {
        match self {
            Self::Thing(thing) => {
                let value = surrealdb::sql::to_value(thing).map_err(|_| Error::Generic {
                    description: "Error while converting VariableName::Thing to value in ThingOrVariable.bind".to_string(),
                })?;
                let variable = match variable {
                    Some(x) => x,
                    None => {
                        return Err(Error::Generic {
                            description: "Error while converting VariableName::Thing to value"
                                .to_string(),
                        })
                    }
                };
                Ok(query.bind((variable, value)))
            }
            Self::Variable(_) => Ok(query),
        }
    }
}

// impl ToThingOrVariable for String {
//     fn thing_or_variable<'a>(&'a self) -> ThingOrVariable<'a> {
//         ThingOrVariable::Thing(self.clone)
//     }
// }

#[derive(Debug)]
pub struct QueryBuilderHelper {
    pub statements: Vec<String>,
    pub binds: Vec<(String, Value)>,
}

impl QueryBuilderHelper {
    pub fn new() -> Self {
        Self {
            statements: vec![],
            binds: vec![],
        }
    }
    pub fn query(mut self, query_string: String) -> Self {
        self.statements.push(query_string);
        self
    }

    pub fn bind(mut self, bind: (String, Value)) -> Self {
        self.binds.push(bind);
        self
    }

    /// TODO: should return result
    pub fn apply_db<C: surrealdb::Connection>(
        self,
        client: &surrealdb::Surreal<C>,
    ) -> surrealdb::method::Query<'_, C> {
        if self.statements.is_empty() {
            return client.query("");
        }
        let mut query = client.query(self.statements[0].clone());
        if self.statements.len() > 1 {
            for (i, statement) in self.statements.iter().enumerate() {
                if i > 0 {
                    query = query.query(statement.clone());
                }
            }
        }
        for bind in self.binds.clone().into_iter() {
            query = query.bind(bind)
        }
        query
    }

    pub fn apply<C: surrealdb::Connection>(
        self,
        query: surrealdb::method::Query<'_, C>,
    ) -> surrealdb::method::Query<C> {
        let mut query = query;
        for statement in self.statements.clone().into_iter() {
            query = query.query(statement);
        }
        for bind in self.binds.clone().into_iter() {
            query = query.bind(bind)
        }
        query
    }
}

impl Default for QueryBuilderHelper {
    fn default() -> Self {
        Self::new()
    }
}
