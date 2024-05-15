use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    datetime::DateTimeDerived,
    production_info::{CreateProductionInfoInput, ProductionInfo, ProductionInfoUseCases},
    thing_derived::ThingDerived,
};

pub struct ProductionInfoQuery;
#[Object]
impl ProductionInfoQuery {
    async fn select_by_id(&self, ctx: &Context<'_>, id: ThingDerived) -> Result<ProductionInfo> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionInfoUseCases::select_by_id(&id, db, ctx).await?)
    }

    async fn select_by_date(
        &self,
        ctx: &Context<'_>,
        date: DateTimeDerived,
    ) -> Result<Option<ProductionInfo>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(ProductionInfoUseCases::select_by_date(date, db, ctx).await?)
    }
}
