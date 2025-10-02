use axum::{
    extract::Extension,
    http::{header::{AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method},
    response::{Html, IntoResponse},
    routing::get,
    serve, Router,
};
use dotenvy::dotenv;
use std::{env, net::SocketAddr};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod db;
mod graphql;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum_extra::typed_header::TypedHeader;
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

async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    let mut request = req.into_inner();

    let auth_result = if let Some(TypedHeader(Authorization(bearer))) = auth {
        match crate::auth::jwt::validate_jwt(bearer.token()) {
            Some(claims) => Ok(crate::auth::extractor::AuthUser(claims.sub)),
            None => Err("Invalid token"),
        }
    } else {
        Err("Missing token")
    };

    request = request.data(auth_result);
    schema.execute(request).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(
        async_graphql::http::GraphiQLSource::build()
            .endpoint("/")
            .finish(),
    )
}
