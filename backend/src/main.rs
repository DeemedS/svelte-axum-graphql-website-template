use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    serve, Router,
};
use dotenvy::dotenv;
use std::net::SocketAddr;
use tokio::net::TcpListener;

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

    let app = Router::new()
        .route("/", get(graphiql).post(graphql_handler))
        .layer(Extension(schema));

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

    // Inject into GraphQL context
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
