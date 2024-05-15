use crate::common::Unwrapper;
use crate::machinery_type::{MachineryType, MachineryTypeUseCases};
use crate::service::guard::RoleGuard;
use crate::thing_derived::ThingDerived;
use crate::thing_wrapper::ObjectWithThing;
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use common::{
    ctx::{Ctx, CtxStruct},
    error::{ApiError, Error},
    role::Role,
    ApiResult,
};

use db::Db;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[allow(dead_code)]
const RESOURCE: &str = "Machinery";

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(guard = "RoleGuard::new(Role::User)")]
#[graphql(complex)]
pub struct Machinery {
    pub id: Option<ThingDerived>,
    pub name: String,
    #[graphql(skip)]
    pub machinery_type: ThingDerived,
}

#[ComplexObject]
impl Machinery {
    async fn machinery_type(&self, ctx: &Context<'_>) -> Result<MachineryType> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(MachineryTypeUseCases::select_by_id(&self.machinery_type.thing(ctx)?, db, ctx).await?)
    }
}

impl ObjectWithThing for Machinery {
    fn thing(&self, ctx: &dyn Ctx) -> ApiResult<Thing> {
        self.id
            .as_ref()
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::Generic {
                    description: "Can't get thing. Get none instead".to_string(),
                },
            })?
            .thing(ctx)
    }
}

pub struct MachineryRepository {}

impl MachineryRepository {
    pub async fn list(
        offset: usize,
        limit: usize,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<Machinery>> {
        let query = db.query(format!(
            "SELECT * FROM {RESOURCE} LIMIT {limit} START {offset};"
        ));
        Unwrapper::unwrapper_vec(query, 0, ctx).await
    }

    pub async fn filtered_list_by_name(
        offset: usize,
        limit: usize,
        name: String,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<Machinery>> {
        let query = db
            .query(format!(
                "SELECT * FROM {RESOURCE} 
                WHERE name @@ $name
                LIMIT {limit} START {offset}"
            ))
            .bind(("name", name));
        Unwrapper::unwrapper_vec(query, 0, ctx).await
    }

    pub async fn count(db: &Db, ctx: &dyn Ctx) -> ApiResult<usize> {
        let query = db.query(format!("(SELECT count() FROM {RESOURCE} GROUP ALL).count"));
        Unwrapper::unwrapper_option(query, 0, &format!("Cant't {RESOURCE} count"), ctx).await
    }

    pub async fn select_by_id(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Machinery> {
        let thing_id = id.thing(ctx)?;
        if thing_id.tb != RESOURCE {
            return Err(ApiError {
                error: Error::Generic {
                    description: "Wrong table name in select_by_id".to_string(),
                },
                req_id: ctx.req_id(),
            });
        }
        let query = db
            .query("SELECT * FROM $thing_id".to_string())
            .bind(("thing_id", thing_id));

        Unwrapper::unwrapper_option(query, 0, "Can't get tag by id", ctx).await
    }

    pub async fn create(
        name: String,
        machinery_type: ThingDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Machinery> {
        db.create(RESOURCE)
            .content(Machinery {
                id: None,
                name,
                machinery_type,
            })
            .await
            .map_err(ApiError::from(ctx))
            .map(|v: Vec<Machinery>| {
                v.into_iter().next().ok_or(ApiError {
                    req_id: ctx.req_id(),
                    error: Error::SurrealDbNoResult {
                        source: "internal".to_string(),
                        id: "Error while creating ".to_string(),
                    },
                })
            })?
    }

    pub async fn update(
        ct_input: CreateMachineryInput,
        id: Thing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Machinery> {
        db.update((RESOURCE, id.id.to_string()))
            .content(ct_input)
            .await
            .map_err(ApiError::from(ctx))?
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: "internal".to_string(),
                    id: id.to_string(),
                },
            })
    }

    pub async fn delete(id: Thing, db: &Db, ctx: &dyn Ctx) -> ApiResult<Machinery> {
        db.delete((RESOURCE, id))
            .await
            .map_err(ApiError::from(ctx))?
            .ok_or(ApiError {
                req_id: ctx.req_id(),
                error: Error::SurrealDbNoResult {
                    source: "internal".to_string(),
                    id: "Error while deleting ".to_string(),
                },
            })
    }
}

#[derive(Deserialize, InputObject, Clone, Serialize, Debug)]
pub struct CreateMachineryInput {
    #[graphql(validator(min_length = 4))]
    pub name: String,
    pub machinery_type: ThingDerived,
}

pub struct MachineryUseCases {
    //pub db: &'a Db,
    //pub ctx: &'a dyn Ctx,
}

impl MachineryUseCases {
    pub async fn create(
        ct_input: CreateMachineryInput,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Machinery> {
        MachineryRepository::create(ct_input.name, ct_input.machinery_type, db, ctx).await
    }

    pub async fn update(
        ct_input: CreateMachineryInput,
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Machinery> {
        MachineryRepository::update(ct_input, id.thing(ctx)?, db, ctx).await
    }

    pub async fn delete(id: &dyn ObjectWithThing, db: &Db, ctx: &dyn Ctx) -> ApiResult<Machinery> {
        MachineryRepository::delete(id.thing(ctx)?, db, ctx).await
    }

    pub async fn select_by_id(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Machinery> {
        MachineryRepository::select_by_id(id, db, ctx).await
    }

    pub async fn count(db: &Db, ctx: &dyn Ctx) -> ApiResult<usize> {
        MachineryRepository::count(db, ctx).await
    }

    pub async fn list(
        offset: Option<usize>,
        limit: Option<usize>,
        name: Option<String>,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<Machinery>> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(10);

        match name {
            Some(name) => {
                MachineryRepository::filtered_list_by_name(offset, limit, name, db, ctx).await
            }
            None => MachineryRepository::list(offset, limit, db, ctx).await,
        }
    }
}
