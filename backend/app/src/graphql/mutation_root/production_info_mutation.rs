use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    production_info::{CreateProductionInfoInput, ProductionInfo, ProductionInfoUseCases},
    thing_derived::ThingDerived,
};

pub struct ProductionInfoMutation;
#[Object]
impl ProductionInfoMutation {
    async fn create(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateProductionInfoInput,
    ) -> Result<ProductionInfo> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionInfoUseCases::create(ct_input, db, ctx).await?)
    }

    async fn update(
        &self,
        ctx: &Context<'_>,
        ct_input: CreateProductionInfoInput,
        id: ThingDerived,
    ) -> Result<ProductionInfo> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionInfoUseCases::update(ct_input, &id, db, ctx).await?)
    }
}
