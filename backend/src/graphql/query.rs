use crate::auth::extractor::AuthUser;
use async_graphql::{Context, Object, Result};
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn me(&self, ctx: &Context<'_>) -> Result<String> {
        let user = ctx
            .data::<Result<AuthUser, &str>>()?
            .as_ref()
            .map_err(|e| async_graphql::Error::new(*e))?;

        Ok(format!("Hello user {}", user.0))
    }
}
