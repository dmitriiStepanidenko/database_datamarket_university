use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    sales_per_day::{
        CreateSalesPlanPerDayTypeInput, SalesPlanPerDay, SalesPlanPerDayUnitsUseCases,
    },
    thing_derived::ThingDerived,
};

pub struct SalesPlanPerDayMutation;
#[Object]
impl SalesPlanPerDayMutation {
    async fn create(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateSalesPlanPerDayTypeInput,
    ) -> Result<SalesPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(SalesPlanPerDayUnitsUseCases::create(ct_input, db, ctx).await?)
    }

    async fn update(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateSalesPlanPerDayTypeInput,
        id: ThingDerived,
    ) -> Result<SalesPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(SalesPlanPerDayUnitsUseCases::update(ct_input, &id, db, ctx).await?)
    }
}
