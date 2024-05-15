use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    pipe::{CreatePipeInput, Pipe, PipeUseCases},
    thing_derived::ThingDerived,
};

pub struct PipeMutation;
#[Object]
impl PipeMutation {
    async fn create(&self, ctx: &Context<'_>, ct_input: CreatePipeInput) -> Result<Pipe> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeUseCases::create(ct_input, db, ctx).await?)
    }

    async fn update(
        &self,
        ctx: &Context<'_>,
        ct_input: CreatePipeInput,
        id: ThingDerived,
    ) -> Result<Pipe> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeUseCases::update(ct_input, &id, db, ctx).await?)
    }
}
