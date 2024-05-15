use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    datetime::DateTimeDerived,
    sales_per_day::{SalesPlanPerDay, SalesPlanPerDayUnitsUseCases},
    thing_derived::ThingDerived,
};

pub struct SalesPlanPerDayQuery;
#[Object]
impl SalesPlanPerDayQuery {
    async fn select_by_id(&self, ctx: &Context<'_>, id: ThingDerived) -> Result<SalesPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(SalesPlanPerDayUnitsUseCases::select_by_id(&id, db, ctx).await?)
    }

    async fn list(
        &self,
        ctx: &Context<'_>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<SalesPlanPerDay>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(SalesPlanPerDayUnitsUseCases::list(offset, limit, db, ctx).await?)
    }
}
