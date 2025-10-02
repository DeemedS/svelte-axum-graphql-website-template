use async_graphql::{EmptySubscription, Schema};

mod mutation;
mod query;
mod types;

pub use mutation::MutationRoot;
pub use query::QueryRoot;

pub type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
