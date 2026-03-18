mod schema;

pub use schema::{Mutation, Query};

use async_graphql::EmptySubscription;

pub type AppSchema = async_graphql::Schema<Query, Mutation, EmptySubscription>;

pub fn schema() -> AppSchema {
    async_graphql::Schema::build(Query, Mutation, EmptySubscription)
        .extension(async_graphql::extensions::Tracing)
        .finish()
}
