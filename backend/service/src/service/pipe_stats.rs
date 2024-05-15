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
const RESOURCE: &str = "PipeStats";

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(guard = "RoleGuard::new(Role::User)")]
pub struct PipeStats {
    pub id: Option<ThingDerived>,
    pub date: DateTimeDerived,
    pub flow: Decimal,
    pub units: ThingDerived,
    pub wearout: Decimal,
    pub pipe: ThingDerived,
}

impl ObjectWithThing for PipeStats {
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

pub struct PipeStatsRepository {}

impl PipeStatsRepository {
    pub async fn list(
        offset: usize,
        limit: usize,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<PipeStats>> {
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
    ) -> ApiResult<PipeStats> {
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

    pub async fn select_by_pipe_and_date(
        pipe: &dyn ObjectWithThing,
        date: DateTimeDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<PipeStats>> {
        let query = db
            .query(
                "SELECT * FROM PipeStats WHERE pipe = $pipe AND time::floor(date, 1d) = time::floor($date, 1d) ORDER BY date ASC;"
                    .to_string(),
            )
            .bind(("pipe", pipe.thing(ctx)?))
            .bind(("date", date.0));

        Unwrapper::unwrapper_vec(query, 0, ctx).await
    }

    pub async fn select_previous_reading_by_pipe_and_date(
        pipe: &dyn ObjectWithThing,
        date: DateTimeDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Option<PipeStats>> {
        let query = db
            .query(
                "SELECT * FROM PipeStats WHERE pipe = $pipe AND time::floor(date, 1d) <= time::floor($date , 1d)-1d ORDER BY date DESC LIMIT 1;"
                    .to_string(),
            )
            .bind(("pipe", pipe.thing(ctx)?))
            .bind(("date", date.0));

        Unwrapper::unwrapper_option_without_error(query, 0, ctx).await
    }

    pub async fn create(
        date: DateTimeDerived,
        flow: Decimal,
        units: ThingDerived,
        wearout: Decimal,
        pipe: ThingDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<PipeStats> {
        db.create(RESOURCE)
            .content(PipeStats {
                id: None,
                date,
                flow,
                units,
                wearout,
                pipe,
            })
            .await
            .map_err(ApiError::from(ctx))
            .map(|v: Vec<PipeStats>| {
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
        ct_input: CreatePipeStatsInput,
        id: Thing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<PipeStats> {
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

    pub async fn delete(id: Thing, db: &Db, ctx: &dyn Ctx) -> ApiResult<PipeStats> {
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
pub struct CreatePipeStatsInput {
    pub date: DateTimeDerived,
    pub flow: Decimal,
    pub units: ThingDerived,
    pub wearout: Decimal,
    pub pipe: ThingDerived,
}

pub struct PipeStatsUseCases {
    //pub db: &'a Db,
    //pub ctx: &'a dyn Ctx,
}

impl PipeStatsUseCases {
    pub async fn create(
        ct_input: CreatePipeStatsInput,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<PipeStats> {
        PipeStatsRepository::create(
            ct_input.date,
            ct_input.flow,
            ct_input.units,
            ct_input.wearout,
            ct_input.pipe,
            db,
            ctx,
        )
        .await
    }

    pub async fn update(
        ct_input: CreatePipeStatsInput,
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<PipeStats> {
        PipeStatsRepository::update(ct_input, id.thing(ctx)?, db, ctx).await
    }

    pub async fn delete(id: &dyn ObjectWithThing, db: &Db, ctx: &dyn Ctx) -> ApiResult<PipeStats> {
        PipeStatsRepository::delete(id.thing(ctx)?, db, ctx).await
    }

    pub async fn select_by_id(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<PipeStats> {
        PipeStatsRepository::select_by_id(id, db, ctx).await
    }

    pub async fn count(db: &Db, ctx: &dyn Ctx) -> ApiResult<usize> {
        PipeStatsRepository::count(db, ctx).await
    }

    pub async fn list(
        offset: Option<usize>,
        limit: Option<usize>,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<PipeStats>> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(0);

        PipeStatsRepository::list(offset, limit, db, ctx).await
    }
}
