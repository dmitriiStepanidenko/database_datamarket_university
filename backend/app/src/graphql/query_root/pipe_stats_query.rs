use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    datetime::DateTimeDerived,
    pipe_stats::{PipeStats, PipeStatsUseCases},
    thing_derived::ThingDerived,
};

pub struct PipeStatsQuery;
#[Object]
impl PipeStatsQuery {
    async fn select_by_id(&self, ctx: &Context<'_>, id: ThingDerived) -> Result<PipeStats> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeStatsUseCases::select_by_id(&id, db, ctx).await?)
    }

    async fn list(
        &self,
        ctx: &Context<'_>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<PipeStats>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeStatsUseCases::list(offset, limit, db, ctx).await?)
    }
}
