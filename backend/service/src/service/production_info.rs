use crate::measure_units::{MeasureUnits, MeasureUnitsUseCases};
use crate::pipe::{Pipe, PipeUseCases};
use crate::pipe_stats::{self, PipeStats, PipeStatsRepository};
use crate::pipe_type::PipeTypeUseCases;
use crate::production_info;
use crate::production_per_day::{
    CreateProductionPlanPerDayTypeInput, ProductionPlanPerDay, ProductionPlanPerDayUseCases,
    ProductionPlandPerDayRepository,
};
use crate::sales_per_day::{
    SalesPlanPerDay, SalesPlanPerDayUnitsUseCases, SalesPlandPerDayRepository,
};
use crate::service::guard::RoleGuard;
use crate::thing_derived::ThingDerived;
use crate::thing_wrapper::ObjectWithThing;
use crate::{common::Unwrapper, datetime::DateTimeDerived};
use async_graphql::{ComplexObject, Context, InputObject, Result, SimpleObject};
use chrono::{DateTime, Duration, Utc};
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
use tokio::net::unix::pipe;

#[allow(dead_code)]
const RESOURCE: &str = "ProductionInfo";

#[derive(Clone, Debug, Serialize, Deserialize, SimpleObject)]
#[graphql(guard = "RoleGuard::new(Role::User)")]
#[graphql(complex)]
pub struct ProductionInfo {
    pub id: Option<ThingDerived>,
    #[graphql(skip)]
    pub sales_plan: ThingDerived,
    #[graphql(skip)]
    pub production_plan: ThingDerived,
    #[graphql(skip)]
    pub final_pipe: ThingDerived,
    #[graphql(skip)]
    pub measure_units: ThingDerived,
    pub date: DateTimeDerived,
}

#[ComplexObject]
impl ProductionInfo {
    async fn production_plan(&self, ctx: &Context<'_>) -> Result<ProductionPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(
            ProductionPlanPerDayUseCases::select_by_id(&self.production_plan.thing(ctx)?, db, ctx)
                .await?,
        )
    }

    async fn sales_plan(&self, ctx: &Context<'_>) -> Result<SalesPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(
            SalesPlanPerDayUnitsUseCases::select_by_id(&self.sales_plan.thing(ctx)?, db, ctx)
                .await?,
        )
    }

    async fn final_pipe(&self, ctx: &Context<'_>) -> Result<Pipe> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(
            PipeUseCases::select_by_id(&self.final_pipe.thing(ctx)?, db, ctx)
                .await?,
        )
    }

    async fn measure_units(&self, ctx: &Context<'_>) -> Result<MeasureUnits> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(
            MeasureUnitsUseCases::select_by_id(&self.measure_units.thing(ctx)?, db, ctx)
                .await?,
        )
    }

    // TODO: добавить кейс когда за этот день данных нет
    async fn production_fact(&self, ctx: &Context<'_>) -> Result<Decimal> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;

        // По дате собираем все нужные pipe_stats +
        // последний pipe_stats предыдущего дня
        //
        // Все за нужный день
        let pipe_stats_by_date = PipeStatsRepository::select_by_pipe_and_date(
            &self.final_pipe,
            self.date.clone(),
            db,
            ctx,
        )
        .await?;

        let pipe_stats_by_previouse_date_lates =
            PipeStatsRepository::select_previous_reading_by_pipe_and_date(
                &self.final_pipe,
                self.date.clone(),
                db,
                ctx,
            )
            .await?;
        println!(
            "Previous date stats: {:?}",
            pipe_stats_by_previouse_date_lates
        );

        let mut result = Decimal::new(0, 0);

        fn time_delta(from: DateTime<Utc>, to: DateTime<Utc>) -> Decimal {
            let duration = to.signed_duration_since(from);
            Decimal::new(duration.num_seconds(), 0) / Decimal::new(3600, 0)
        }
        fn hours_until_next_day(dt: DateTime<Utc>) -> Decimal {
            // Calculate the start of the next day
            let next_day = dt.date().succ().and_hms(0, 0, 0);

            // Calculate the duration until the start of the next day
            let duration = next_day.signed_duration_since(dt);

            // Convert the duration to a floating-point number of hours
            Decimal::new(duration.num_seconds(), 0) / Decimal::new(3600, 0)
        }
        fn hours_from_current_day(dt: DateTime<Utc>) -> Decimal {
            // Get the start of the current day (midnight)
            let start_of_day = dt.date().and_hms(0, 0, 0);

            // Calculate the duration from the start of the current day
            let duration = dt.signed_duration_since(start_of_day);

            // Convert the duration to a floating-point number of hours
            Decimal::new(duration.num_seconds(), 0) / Decimal::new(3600, 0)
        }
        println!("Result = {:?}", result);

        match pipe_stats_by_previouse_date_lates {
            None => {}
            Some(pipe_stats) => {
                if pipe_stats_by_date.is_empty() {
                    result = pipe_stats.flow * Decimal::new(24, 0);
                    return Ok(result);
                }
                result = pipe_stats.flow * hours_from_current_day(pipe_stats_by_date[0].date.0.0);
            }
        }
        println!("pipe_stats_by_date = {:?}", pipe_stats_by_date);
        for (i, el) in pipe_stats_by_date.iter().enumerate() {
            println!("Result = {:?}", result);
            if i == pipe_stats_by_date.len() - 1 {
                result += el.flow * hours_until_next_day(el.date.0 .0);
            } else {
                println!("el.flow = {:?}", el.flow);
                result += el.flow * time_delta(el.date.0 .0, pipe_stats_by_date[i + 1].date.0 .0);
            }
        }
        println!("Result = {:?}", result);

        Ok(result)
    }
}

impl ObjectWithThing for ProductionInfo {
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

pub struct ProductionInfoRepository {}

impl ProductionInfoRepository {
    pub async fn list(
        offset: usize,
        limit: usize,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<ProductionInfo>> {
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
    ) -> ApiResult<Vec<ProductionInfo>> {
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
    ) -> ApiResult<ProductionInfo> {
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
    ) -> ApiResult<Option<ProductionInfo>> {
        let query = db
            .query(format!(
                "SELECT * FROM {RESOURCE} WHERE time::floor(date, 1d) = time::floor($date, 1d);"
            ))
            .bind(("date", date.0));
        let result = Unwrapper::unwrapper_vec::<ProductionInfo, _>(query, 0, ctx).await?;
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result[0].clone()))
    }

    pub async fn create(
        sales_plan: ThingDerived,
        production_plan: ThingDerived,
        final_pipe: ThingDerived,
        measure_units: ThingDerived,
        date: DateTimeDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionInfo> {
        db.create(RESOURCE)
            .content(ProductionInfo {
                id: None,
                sales_plan,
                production_plan,
                final_pipe,
                measure_units,
                date,
            })
            .await
            .map_err(ApiError::from(ctx))
            .map(|v: Vec<ProductionInfo>| {
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
        ct_input: CreateProductionInfoInput,
        id: Thing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionInfo> {
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

    pub async fn delete(id: Thing, db: &Db, ctx: &dyn Ctx) -> ApiResult<ProductionInfo> {
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
pub struct CreateProductionInfoInput {
    pub sales_plan: ThingDerived,
    pub production_plan: ThingDerived,
    pub final_pipe: ThingDerived,
    pub measure_units: ThingDerived,
    pub date: DateTimeDerived,
}

pub struct ProductionInfoUseCases {}

impl ProductionInfoUseCases {
    pub async fn create(
        ct_input: CreateProductionInfoInput,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionInfo> {
        ProductionInfoRepository::create(
            ct_input.sales_plan,
            ct_input.production_plan,
            ct_input.final_pipe,
            ct_input.measure_units,
            ct_input.date,
            db,
            ctx,
        )
        .await
    }

    /// If record is exist, then return the record
    /// If not, try to create record
    /// If not all data avaliable, return None
    pub async fn select_create(
        date: DateTimeDerived,
        final_pipe: Option<ThingDerived>,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Option<ProductionInfo>> {
        let production_info =
            ProductionInfoRepository::select_by_date(date.clone(), db, ctx).await?;
        if production_info.is_some() {
            return Ok(production_info);
        }
        // Record does not exist. Gather needed data then
        let sales_plan =
            match SalesPlandPerDayRepository::select_by_date(date.clone(), db, ctx).await? {
                None => return Ok(None),
                Some(plan) => plan,
            };
        let production_plan =
            match ProductionPlandPerDayRepository::select_by_date(date.clone(), db, ctx).await? {
                None => return Ok(None),
                Some(plan) => plan,
            };
        let final_pipe = match final_pipe {
            None => {
                return Ok(None);
            }
            Some(pipe) => PipeUseCases::select_by_id(&pipe, db, ctx).await?,
        };
        let pipe_type = PipeTypeUseCases::select_by_id(&final_pipe.pipe_type, db, ctx).await?;
        let input = CreateProductionInfoInput {
            sales_plan: sales_plan.thing(ctx)?.into(),
            production_plan: production_plan.thing(ctx)?.into(),
            final_pipe: final_pipe.thing(ctx)?.into(),
            date,
            measure_units: pipe_type.units,
        };
        let result = ProductionInfoUseCases::create(input, db, ctx).await?;

        Ok(Some(result))
    }

    pub async fn update(
        ct_input: CreateProductionInfoInput,
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionInfo> {
        ProductionInfoRepository::update(ct_input, id.thing(ctx)?, db, ctx).await
    }

    pub async fn delete(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionInfo> {
        ProductionInfoRepository::delete(id.thing(ctx)?, db, ctx).await
    }

    pub async fn select_by_id(
        id: &dyn ObjectWithThing,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<ProductionInfo> {
        ProductionInfoRepository::select_by_id(id, db, ctx).await
    }

    pub async fn select_by_date(
        date: DateTimeDerived,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Option<ProductionInfo>> {
        ProductionInfoRepository::select_by_date(date, db, ctx).await
    }

    pub async fn count(db: &Db, ctx: &dyn Ctx) -> ApiResult<usize> {
        ProductionInfoRepository::count(db, ctx).await
    }

    pub async fn list(
        offset: Option<usize>,
        limit: Option<usize>,
        name: Option<String>,
        db: &Db,
        ctx: &dyn Ctx,
    ) -> ApiResult<Vec<ProductionInfo>> {
        let limit = limit.unwrap_or(10);
        let offset = offset.unwrap_or(10);

        match name {
            Some(name) => {
                ProductionInfoRepository::filtered_list_by_name(offset, limit, name, db, ctx).await
            }
            None => ProductionInfoRepository::list(offset, limit, db, ctx).await,
        }
    }
}
