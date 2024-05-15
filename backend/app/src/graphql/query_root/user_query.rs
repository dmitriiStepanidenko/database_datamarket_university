use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;
use service::user::{User, UserUseCases};

pub struct UserQuery;

#[Object]
impl UserQuery {
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(UserUseCases::me(db, ctx).await?)
    }
}
