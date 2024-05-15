use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    production_per_day::{
        CreateProductionPlanPerDayTypeInput, ProductionPlanPerDay, ProductionPlanPerDayUseCases,
    },
    thing_derived::ThingDerived,
};

pub struct ProductionPlanPerDayMutation;
#[Object]
impl ProductionPlanPerDayMutation {
    async fn create(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateProductionPlanPerDayTypeInput,
    ) -> Result<ProductionPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionPlanPerDayUseCases::create(ct_input, db, ctx).await?)
    }

    async fn update(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateProductionPlanPerDayTypeInput,
        id: ThingDerived,
    ) -> Result<ProductionPlanPerDay> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionPlanPerDayUseCases::update(ct_input, &id, db, ctx).await?)
    }
}
