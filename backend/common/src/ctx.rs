use crate::error::*;
use crate::mw_ctx::Claims;
use crate::role::Roles;
use axum::extract::FromRequestParts;

use cookie::Cookie;
use surrealdb::sql::{thing, Thing};
use tower_cookies::Cookies;
use uuid::Uuid;

//#[cfg(debug_assertions)]
use mockall::*;

#[derive(Clone, Debug)]
pub struct CtxStruct {
    claims: Result<Claims>,
    req_id: Uuid,
    cookies: Cookies,
}

//#[cfg_attr(debug_assertions, automock)]
#[automock]
pub trait Ctx: Send + Sync {
    fn user_id(&self) -> ApiResult<String>;

    fn user_id_thing(&self) -> ApiResult<Thing>;

    fn roles(&self) -> ApiResult<Roles>;

    /// DO NOT USE THIS
    /// WHY? Because Cookies have a private new() method,
    /// so we can't test it. Use cookies_add/cookies_remove instead
    ///
    fn cookies(&self) -> Cookies;

    fn cookies_add(&self, cookie: Cookie<'static>) -> ApiResult<()>;

    fn req_id(&self) -> Uuid;

    //type Rejection: IntoResponse;
    //fn from_request_parts<'life0, 'life1, 'async_trait, S>(
    //    parts: &'life0 mut axum::http::request::Parts,
    //    _state: &'life1 S,
    //) -> core::pin::Pin<
    //    Box<dyn core::future::Future<Output = ApiResult<Self>> + core::marker::Send + 'async_trait>,
    //>
    //where
    //    'life0: 'async_trait,
    //    'life1: 'async_trait,
    //    Self: 'async_trait,
    //    S: Send + Sync;
}

impl CtxStruct {
    pub fn new(claims: Result<Claims>, uuid: Uuid, cookies: Cookies) -> Self {
        Self {
            claims,
            req_id: uuid,
            cookies,
        }
    }
    //#[cfg(test)]
    //pub fn mock() -> Self {
    //    Self {
    //        claims: Err(Error::Generic { description: "generic".to_string() }),
    //        req_id: uuid::Uuid::new_v4(),
    //        cookies: Cookies::new(vec![]),
    //    }
    //}
}

impl Ctx for CtxStruct {
    fn user_id(&self) -> ApiResult<String> {
        Ok(self
            .claims
            .clone()
            .map_err(|error| ApiError {
                error,
                req_id: self.req_id,
            })?
            .id)
    }
    fn user_id_thing(&self) -> ApiResult<Thing> {
        thing(&self.user_id()?).map_err(|_| ApiError {
            error: Error::Generic {
                description: "Problem with converting user_id".to_string(),
            },
            req_id: self.req_id,
        })
    }

    fn roles(&self) -> ApiResult<Roles> {
        Ok(self
            .claims
            .clone()
            .map_err(|error| ApiError {
                error,
                req_id: self.req_id,
            })?
            .roles)
    }

    fn cookies(&self) -> Cookies {
        self.cookies.clone()
    }

    fn cookies_add(&self, cookie: Cookie<'static>) -> ApiResult<()> {
        self.cookies().add(cookie);
        Ok(())
    }

    fn req_id(&self) -> Uuid {
        self.req_id
    }

    //type Rejection = ApiError;
    //async fn from_request_parts<S>(
    //    parts: &mut axum::http::request::Parts,
    //    _state: & S,
    //) -> core::pin::Pin<
    //    Box<dyn core::future::Future<Output = ApiResult<Self>> + core::marker::Send>,
    //>
    //where
    //    S: Send + Sync,
    //{
    //    Box::pin(async {
    //        println!(
    //            "->> {:<12} - Ctx::from_request_parts - extract Ctx from extension",
    //            "EXTRACTOR"
    //        );
    //        parts
    //            .extensions
    //            .get::<CtxStruct>()
    //            .cloned()
    //            .ok_or(ApiError {
    //                req_id: Uuid::new_v4(),
    //                error: Error::AuthFailCtxNotInRequestExt,
    //            })
    //    })
    //}
}

// ugly but direct implementation from axum, until "async trait fn" are in stable rust, instead of importing some 3rd party macro
// Extractor - makes it possible to specify Ctx as a param - fetches the result from the header parts extension
impl<S: Send + Sync> FromRequestParts<S> for CtxStruct {
    type Rejection = ApiError;
    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut axum::http::request::Parts,
        _state: &'life1 S,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = ApiResult<Self>> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async {
            println!(
                "->> {:<12} - Ctx::from_request_parts - extract Ctx from extension",
                "EXTRACTOR"
            );
            parts
                .extensions
                .get::<CtxStruct>()
                .cloned()
                .ok_or(ApiError {
                    req_id: Uuid::new_v4(),
                    error: Error::AuthFailCtxNotInRequestExt,
                })
        })
    }
}
