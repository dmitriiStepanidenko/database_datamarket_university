use crate::role::Roles;
use crate::{
    ctx::{Ctx, CtxStruct},
    error::Error,
    error::Result,
    ApiResult,
};
use axum::{extract::State, http::Request, middleware::Next, response::Response};
use db::Db;
use jsonwebtoken::{decode, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;


#[derive(Clone)]
pub struct CtxState {
    // NOTE: with DB, because a real login would check the DB
    pub _db: Db,
    pub key_enc: EncodingKey,
    pub key_dec: DecodingKey,
}

pub const JWT_KEY: &str = "jwt";
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub exp: usize,
    pub email: String,
    pub id: String,
    pub roles: Roles,
}

pub async fn mw_require_auth<B>(
    ctx: CtxStruct,
    req: Request<B>,
    next: Next<B>,
) -> ApiResult<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");
    return Ok(next.run(req).await);
    ctx.user_id()?;
    Ok(next.run(req).await)
}

pub async fn mw_ctx_constructor<B>(
    State(CtxState { _db, key_dec, .. }): State<CtxState>,
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Response {
    println!("->> {:<12} - mw_ctx_constructor", "MIDDLEWARE");

    let uuid = Uuid::new_v4();
    let claims: Result<Claims> = extract_token(key_dec, &cookies).map_err(|err| {
        // Remove an invalid cookie
        if let Error::AuthFailJwtInvalid { .. } = err {
            cookies.remove(Cookie::named(JWT_KEY))
        }
        err
    });
    // NOTE: DB should be checked here

    // Store Ctx in the request extension, for extracting in rest handlers
    let ctx = CtxStruct::new(claims, uuid, cookies);
    req.extensions_mut().insert(ctx);

    next.run(req).await
}

fn verify_token(key: DecodingKey, token: &str) -> Result<Claims> {
    Ok(decode::<Claims>(token, &key, &Validation::default())?.claims)
}
fn extract_token(key: DecodingKey, cookies: &Cookies) -> Result<Claims> {
    cookies
        .get(JWT_KEY)
        .ok_or(Error::AuthFailNoJwtCookie)
        .and_then(|cookie| verify_token(key, cookie.value()))
}

