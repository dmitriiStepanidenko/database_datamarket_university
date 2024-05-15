use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{pipe_stats::{CreatePipeStatsInput, PipeStats, PipeStatsUseCases}, thing_derived::ThingDerived};

pub struct PipeStatsMutation;
#[Object]
impl PipeStatsMutation {
    async fn create(&self, ctx: &Context<'_>, ct_input: CreatePipeStatsInput) -> Result<PipeStats> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeStatsUseCases::create(ct_input, db, ctx).await?)
    }

    async fn update(&self, ctx: &Context<'_>, ct_input: CreatePipeStatsInput, id: ThingDerived) -> Result<PipeStats> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeStatsUseCases::update(ct_input, &id, db, ctx).await?)
    }
}
