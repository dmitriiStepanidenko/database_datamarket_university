mod graphql;

pub use common::mw_req_logger;
pub use common::{error, mw_ctx};
pub use db::db;

#[cfg(test)]
pub use db::set_database;
pub use db::Db;

use async_graphql::{EmptySubscription, Schema};
use axum::{
    extract::Extension,
    middleware,
    routing::{get_service, post},
    Router,
};
use error::Result;
use graphql::{
    graphiql, graphql_handler, mutation_root::MutationRoot, query_root::QueryRoot, ApiSchema,
};
use http::HeaderValue;
use http::Method;
use jsonwebtoken::{DecodingKey, EncodingKey};
use mw_ctx::CtxState;
use mw_req_logger::mw_req_logger;
use std::net::{Ipv4Addr, SocketAddr};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[cfg(debug_assertions)]
lazy_static::lazy_static! {
    static ref SECRET: String = "testEnc".to_string();
    static ref PORT: u16 = std::env::var("PORT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or( 55000 );
    static ref ADDR: Ipv4Addr =
        std::env::var("ADDR")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or( [127,0, 0, 1].into() );
}

#[cfg(not(debug_assertions))]
lazy_static::lazy_static! {
    static ref SECRET: String =
        std::env::var("SECRET").expect("SECRET must be set");
    static ref ADDR: Ipv4Addr=
        std::env::var("ADDR").expect("ADDR must be set")
        .parse().expect("ADDR expected Ipv4Addr value");
    static ref PORT: u16 =
        std::env::var("PORT").expect("PORT must be set").parse().expect("PORT expected u16 value");
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("App started");
    let _ = db::set_database().await;
    println!("Migrations finished");

    // no-DB in-memory
    // let mc = ModelController::new().await?;

    let key_enc = EncodingKey::from_secret(SECRET.as_bytes());
    let key_dec = DecodingKey::from_secret(SECRET.as_bytes());

    //#[cfg(not(debug_assertions))]
    service::prod_populate::check_and_recreate_admin_user().await;
    service::prod_populate::seed_data().await;

    // GQL
    let schema: ApiSchema = Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        //.data(mc.clone())
        .data(db::DB.clone())
        .data(key_enc.clone())
        .finish();

    let mut include_graphiql = cfg!(debug_assertions);

    if !include_graphiql {
        match std::env::var("INCLUDE_GRAPHIQL") {
            Err(_) => {}
            Ok(value) => {
                include_graphiql = value
                    .parse::<bool>()
                    .expect("WRONG TYPE OF ENV: INCLUDE_GRAPHIQL. MUST BE BOOL");
            }
        }
    }
    let mut gql = Router::new();
    // FIXME
    include_graphiql = true;
    if include_graphiql {
        gql = gql.route(
            "/",
            post(graphql_handler).options(graphql_handler).get(graphiql),
        );
    } else {
        gql = gql.route("/", post(graphql_handler));
    }
    gql = gql
        .layer(Extension(schema))
        // Require auth to access gql
        .route_layer(middleware::from_fn(mw_ctx::mw_require_auth));

    let ctx_state = CtxState {
        _db: db::DB.clone(),
        key_enc,
        key_dec,
    };

    let origins = match std::env::var("EXTRA_ORIGINS") {
        Ok(extra_origins) => {
            let mut origins = vec![
                "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                "http://localhost:4173".parse::<HeaderValue>().unwrap(),
                format!("http://localhost:{}", *PORT)
                    .parse::<HeaderValue>()
                    .unwrap(),
            ];
            extra_origins.split(',').for_each(|origin| {
                if let Ok(parsed_origin) = origin.parse::<HeaderValue>() {
                    origins.push(parsed_origin);
                }
            });
            println!("Origins: {:?}", origins);

            origins
        }
        Err(_) => vec![
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
            "http://localhost:4173".parse::<HeaderValue>().unwrap(),
            format!("http://localhost:{}", *PORT)
                .parse::<HeaderValue>()
                .unwrap(),
        ],
    };
    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // allow requests from any origin
        .allow_origin(origins)
        .allow_credentials(true)
        .allow_headers([http::header::HeaderName::from_static("content-type")]);

    // Main router
    let routes_all = Router::new()
        .merge(gql)
        .layer(middleware::map_response(mw_req_logger))
        // This is where Ctx gets created, with every new request
        .layer(middleware::from_fn_with_state(
            ctx_state.clone(),
            mw_ctx::mw_ctx_constructor,
        ))
        // Layers are executed from bottom up, so CookieManager has to be under ctx_constructor
        .layer(CookieManagerLayer::new())
        .layer(cors)
        .fallback_service(routes_static());

    //Ipv4Addr::LOCALHOST
    //let addr: Ipv4Addr = [127, 0, 0, 1].into();
    let addr = SocketAddr::from((*ADDR, *PORT));
    println!("->> LISTENING on {addr}\n");

    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();

    // fallback fs
    fn routes_static() -> Router {
        Router::new().nest_service("/", get_service(ServeDir::new("./")))
    }

    Ok(())
}
