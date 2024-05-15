use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    datetime::DateTimeDerived,
    measure_units::{MeasureUnits, MeasureUnitsUseCases},
    thing_derived::ThingDerived,
};

pub struct MeasureUnitsQuery;
#[Object]
impl MeasureUnitsQuery {
    async fn select_by_id(&self, ctx: &Context<'_>, id: ThingDerived) -> Result<MeasureUnits> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(MeasureUnitsUseCases::select_by_id(&id, db, ctx).await?)
    }

    async fn list(
        &self,
        ctx: &Context<'_>,
        offset: Option<usize>,
        limit: Option<usize>,
        name: Option<String>,
    ) -> Result<Vec<MeasureUnits>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(MeasureUnitsUseCases::list(offset, limit, name, db, ctx).await?)
    }
}
