use crate::measure_units::MeasureUnitsUseCases;
use crate::service::guard::RoleGuard;
use crate::thing_derived::ThingDerived;
use crate::thing_wrapper::ObjectWithThing;
use crate::{common::Unwrapper, measure_units::MeasureUnits};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use common::{
    ctx::{Ctx, CtxStruct},
    error::{ApiError, Error},
    role::Role,
    ApiResult,
};

use db::Db;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[allow(dead_code)]
const RESOURCE: &str = "MachineryType";

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(guard = "RoleGuard::new(Role::User)")]
#[graphql(complex)]
pub struct MachineryType {
    pub id: Option<ThingDerived>,
    pub name: String,
    pub wearout_max: Decimal,
    pub max_flow: Decimal,
    #[graphql(skip)]
    pub units: ThingDerived,
}

#[ComplexObject]
impl MachineryType {
    async fn units(&self, ctx: &Context<'_>) -> Result<MeasureUnits> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(MeasureUnitsUseCases::select_by_id(&self.units.thing(ctx)?, db, ctx).await?)
    }
}

impl ObjectWithThing for MachineryType {
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

pub struct MachineryTypeRepository {}

impl MachineryTypeRepository {
    pub async fn list(
        offset: usize,
        limit: usize,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<MachineryType>> {
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
    ) -> ApiResult<Vec<MachineryType>> {
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
    ) -> ApiResult<MachineryType> {
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
        max_flow: Decimal,
        wearout_max: Decimal,
        units: ThingDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<MachineryType> {
        db.create(RESOURCE)
            .content(MachineryType {
                id: None,
                name,
                max_flow,
                wearout_max,
                units,
            })
            .await
            .map_err(ApiError::from(ctx))
            .map(|v: Vec<MachineryType>| {
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
        ct_input: CreateMachineryTypeInput,
        id: Thing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<MachineryType> {
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

    pub async fn delete(id: Thing, db: &Db, ctx: &dyn Ctx) -> ApiResult<MachineryType> {
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
pub struct CreateMachineryTypeInput {
    #[graphql(validator(min_length = 4))]
    pub name: String,
    pub max_flow: Decimal,
    pub wearout_max: Decimal,
    pub units: ThingDerived,
}

pub struct MachineryTypeUseCases {
    //pub db: &'a Db,
    //pub ctx: &'a dyn Ctx,
}

impl MachineryTypeUseCases {
    pub async fn create(
        ct_input: CreateMachineryTypeInput,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<MachineryType> {
        MachineryTypeRepository::create(
            ct_input.name,
            ct_input.max_flow,
            ct_input.wearout_max,
            ct_input.units,
            db,
            ctx,
        )
        .await
    }

    pub async fn update(
        ct_input: CreateMachineryTypeInput,
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<MachineryType> {
        MachineryTypeRepository::update(ct_input, id.thing(ctx)?, db, ctx).await
    }

    pub async fn delete(id: &dyn ObjectWithThing, db: &Db, ctx: &dyn Ctx) -> ApiResult<MachineryType> {
        MachineryTypeRepository::delete(id.thing(ctx)?, db, ctx).await
    }

    pub async fn select_by_id(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<MachineryType> {
        MachineryTypeRepository::select_by_id(id, db, ctx).await
    }

    pub async fn count(db: &Db, ctx: &dyn Ctx) -> ApiResult<usize> {
        MachineryTypeRepository::count(db, ctx).await
    }

    pub async fn list(
        offset: Option<usize>,
        limit: Option<usize>,
        name: Option<String>,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<MachineryType>> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(10);

        match name {
            Some(name) => {
                MachineryTypeRepository::filtered_list_by_name(offset, limit, name, db, ctx).await
            }
            None => MachineryTypeRepository::list(offset, limit, db, ctx).await,
        }
    }
}
