use async_graphql::{InputObject, Object};
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[Object]
impl User {
    async fn id(&self) -> i32 {
        self.id
    }
    async fn name(&self) -> &str {
        &self.name
    }
    async fn email(&self) -> &str {
        &self.email
    }
}

#[derive(InputObject)]
pub struct NewUser {
    pub name: String,
    pub email: String,
}
