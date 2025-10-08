use axum::{
    extract::Extension,
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method},
    response::{Html, IntoResponse},
    routing::get,
    serve, Router,
};
use axum_extra::{extract::cookie::{Cookie, CookieJar, SameSite}, TypedHeader};
use dotenvy::dotenv;
use std::{env, net::SocketAddr};
use time::Duration;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use serde_json::Value;

mod auth;
mod db;
mod graphql;
mod utils;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use graphql::{AppSchema, MutationRoot, QueryRoot};
use headers::{authorization::Bearer, Authorization};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let pool = db::init_pool().await?;

    let schema = AppSchema::build(QueryRoot, MutationRoot, async_graphql::EmptySubscription)
        .data(pool.clone())
        .finish();

    // Configure CORS
    let cors_layer = match env::var("CORS_ALLOWED_ORIGINS").ok().filter(|v| !v.trim().is_empty()) {
        Some(origins_str) if origins_str.trim() == "*" => {
            // Allow all origins but NO credentials
            Some(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST])
                    .allow_origin(Any)
                    .allow_headers([CONTENT_TYPE, AUTHORIZATION]),
            )
        }
        Some(origins_str) => {
            // Explicit origins with credentials
            let origins: Vec<HeaderValue> = origins_str
                .split(',')
                .map(|o| HeaderValue::from_str(o.trim()).expect("Invalid origin in CORS_ALLOWED_ORIGINS"))
                .collect();

            Some(
                CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST])
                    .allow_origin(origins)
                    .allow_headers([CONTENT_TYPE, AUTHORIZATION])
                    .allow_credentials(true),
            )
        }
        None => None,
    };
    
    let mut app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

    if let Some(cors) = cors_layer {
        app = app.layer(cors);
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("🚀 Running at http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    serve(listener, app.into_make_service()).await?;

    Ok(())
}


pub async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    req: GraphQLRequest,
) -> impl IntoResponse {
    let mut request = req.into_inner();

    // Inject user claims if JWT is valid
    if let Some(TypedHeader(Authorization(bearer))) = auth {
        if let Some(claims) = crate::auth::jwt::validate_jwt(bearer.token()) {
            request = request.data(crate::auth::extractor::AuthUser(claims.sub));
        }
    }

    let response = schema.execute(request).await;

    let json_response = serde_json::to_value(&response).unwrap_or(Value::Null);

    let mut jar = CookieJar::new();

    if let Some(access_token) = json_response["data"]["login"]["accessToken"].as_str() {
        if let Some(refresh_token) = json_response["data"]["login"]["refreshToken"].as_str() {
            jar = jar.add(
                Cookie::build(("access_token", access_token.to_string()))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::minutes(15))
                    .build(),
            );
            jar = jar.add(
                Cookie::build(("refresh_token", refresh_token.to_string()))
                    .path("/")
                    .http_only(true)
                    .secure(true)
                    .same_site(SameSite::Lax)
                    .max_age(Duration::days(7))
                    .build(),
            );
        }
    }

    let gql_resp = GraphQLResponse::from(response);
    (jar, gql_resp).into_response()
}


async fn graphiql() -> impl IntoResponse {
    Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/")
            .finish(),
    )
}
