use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    datetime::DateTimeDerived,
    production_per_day::{ProductionPlanPerDay, ProductionPlanPerDayUseCases},
    thing_derived::ThingDerived,
};

pub struct ProductionPlanPerDayQuery;
#[Object]
impl ProductionPlanPerDayQuery {
    async fn select_by_id(&self, ctx: &Context<'_>, id: ThingDerived) -> Result<ProductionPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionPlanPerDayUseCases::select_by_id(&id, db, ctx).await?)
    }

    async fn list(
        &self,
        ctx: &Context<'_>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<ProductionPlanPerDay>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionPlanPerDayUseCases::list(offset, limit, db, ctx).await?)
    }
}
