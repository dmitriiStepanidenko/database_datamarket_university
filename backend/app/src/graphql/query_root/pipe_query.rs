use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;

use service::{
    datetime::DateTimeDerived,
    pipe::{ Pipe, PipeUseCases},
    thing_derived::ThingDerived,
};

pub struct PipeQuery;
#[Object]
impl PipeQuery {
    async fn select_by_id(&self, ctx: &Context<'_>, id: ThingDerived) -> Result<Pipe> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeUseCases::select_by_id(&id, db, ctx).await?)
    }

    async fn list(
        &self,
        ctx: &Context<'_>,
        offset: Option<usize>,
        limit: Option<usize>,
        name: Option<String>,
    ) -> Result<Vec<Pipe>> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(PipeUseCases::list(offset, limit, name, db, ctx).await?)
    }
}
