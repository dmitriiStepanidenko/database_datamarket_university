use common::{
    ctx::Ctx,
    error::{ApiError, ApiResult, Error},
};
use surrealdb::sql::{thing, Thing};

pub fn to_thing_format(id: &str, table: &str, ctx: &dyn Ctx) -> ApiResult<Thing> {
    to_thing(format!("{}:{}", table, id).as_str(), ctx)
}

pub fn to_thing(id: &str, ctx: &dyn Ctx) -> ApiResult<Thing> {
    thing(id).map_err(|_| ApiError {
        req_id: ctx.req_id(),
        error: Error::Generic {
            description: format!("Can't convert value={} to thing", id),
        },
    })
}
