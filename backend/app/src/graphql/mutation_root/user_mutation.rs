use async_graphql::{Context, Object, Result};
use common::ctx::CtxStruct;
use db::Db;
use jsonwebtoken::EncodingKey;

use service::user::{CreateUserInput, User, UserUseCases};

pub struct UserMutation;
#[Object]
impl UserMutation {
    async fn create_user(&self, ctx: &Context<'_>, ct_input: CreateUserInput) -> Result<User> {
        let db = ctx.data::<Db>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(UserUseCases::create_user(ct_input, db, ctx).await?)
    }

    async fn login(&self, ctx: &Context<'_>, ct_input: CreateUserInput) -> Result<User> {
        let db = ctx.data::<Db>()?;
        let key_enc = ctx.data::<EncodingKey>()?;
        let ctx = ctx.data::<CtxStruct>()?;
        Ok(UserUseCases::login(ct_input, key_enc.clone(), db, ctx).await?)
    }

}
