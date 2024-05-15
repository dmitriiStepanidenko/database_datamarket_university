use async_graphql::{Context, Guard};
use common::{
    ctx::{Ctx, CtxStruct},
    error::{ApiError, Error},
    role::Role,
};


#[derive(Eq, PartialEq, Copy, Clone)]
pub struct RoleGuard {
    role: Role,
}

impl RoleGuard {
    pub fn new(role: Role) -> Self {
        Self { role }
    }
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &'_ Context<'_>) -> Result<(), async_graphql::Error> {
        let ctx = ctx.data::<CtxStruct>()?;
        let roles = ctx.roles()?;
        if roles.contains(&self.role) {
            Ok(())
        } else {
            Err(ApiError {
                req_id: ctx.req_id(),
                error: Error::Forbidden,
            }
            .into())
        }
    }
}
