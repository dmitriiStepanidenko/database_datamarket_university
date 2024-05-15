use crate::service::guard::RoleGuard;
use crate::thing_derived::ThingDerived;
use crate::thing_wrapper::ObjectWithThing;
use crate::{common::Unwrapper, datetime::DateTimeDerived};
use async_graphql::{InputObject, SimpleObject};
use common::{
    ctx::Ctx,
    error::{ApiError, Error},
    role::Role,
    ApiResult,
};

use db::Db;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use surrealdb::sql::Thing;

#[allow(dead_code)]
const RESOURCE: &str = "ProductionPlanPerDay";

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(guard = "RoleGuard::new(Role::User)")]
pub struct ProductionPlanPerDay {
    pub id: Option<ThingDerived>,
    pub amount: Decimal,
    pub units: ThingDerived,
    pub date: DateTimeDerived,
}

impl ObjectWithThing for ProductionPlanPerDay {
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

pub struct ProductionPlandPerDayRepository {}

impl ProductionPlandPerDayRepository {
    pub async fn list(
        offset: usize,
        limit: usize,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<ProductionPlanPerDay>> {
        let query = db.query(format!(
            "SELECT * FROM {RESOURCE} LIMIT {limit} START {offset};"
        ));
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
    ) -> ApiResult<ProductionPlanPerDay> {
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

    pub async fn select_by_date(
        date: DateTimeDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Option<ProductionPlanPerDay>> {
        let query = db
            .query(format!(
                "SELECT * FROM {RESOURCE} WHERE time::floor(date, 1d) = time::floor($date, 1d);"
            ))
            .bind(("date", date.0));
        let result = Unwrapper::unwrapper_vec::<ProductionPlanPerDay, _>(query, 0, ctx).await?;
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result[0].clone()))
    }

    pub async fn create(
        amount: Decimal,
        units: ThingDerived,
        date: DateTimeDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionPlanPerDay> {
        db.create(RESOURCE)
            .content(ProductionPlanPerDay {
                id: None,
                amount,
                units,
                date,
            })
            .await
            .map_err(ApiError::from(ctx))
            .map(|v: Vec<ProductionPlanPerDay>| {
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
        ct_input: CreateProductionPlanPerDayTypeInput,
        id: Thing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionPlanPerDay> {
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

    pub async fn delete(id: Thing, db: &Db, ctx: &dyn Ctx) -> ApiResult<ProductionPlanPerDay> {
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
pub struct CreateProductionPlanPerDayTypeInput {
    pub amount: Decimal,
    pub units: ThingDerived,
    pub date: DateTimeDerived,
}

pub struct ProductionPlanPerDayUseCases {
    //pub db: &'a Db,
    //pub ctx: &'a dyn Ctx,
}

impl ProductionPlanPerDayUseCases {
    pub async fn create(
        ct_input: CreateProductionPlanPerDayTypeInput,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionPlanPerDay> {
        ProductionPlandPerDayRepository::create(
            ct_input.amount,
            ct_input.units,
            ct_input.date,
            db,
            ctx,
        )
        .await
    }

    pub async fn update(
        ct_input: CreateProductionPlanPerDayTypeInput,
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionPlanPerDay> {
        ProductionPlandPerDayRepository::update(ct_input, id.thing(ctx)?, db, ctx).await
    }

    pub async fn delete(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionPlanPerDay> {
        ProductionPlandPerDayRepository::delete(id.thing(ctx)?, db, ctx).await
    }

    pub async fn select_by_id(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionPlanPerDay> {
        ProductionPlandPerDayRepository::select_by_id(id, db, ctx).await
    }

    pub async fn count(db: &Db, ctx: &dyn Ctx) -> ApiResult<usize> {
        ProductionPlandPerDayRepository::count(db, ctx).await
    }

    pub async fn list(
        offset: Option<usize>,
        limit: Option<usize>,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<ProductionPlanPerDay>> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(10);

        ProductionPlandPerDayRepository::list(offset, limit, db, ctx).await
    }
}
