use async_graphql::{Object, SimpleObject};

pub struct Query;

#[Object]
impl Query {
    async fn health(&self) -> HealthStatus {
        HealthStatus {
            status: "ok".to_string(),
        }
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    async fn noop(&self) -> bool {
        true
    }
}

#[derive(SimpleObject)]
struct HealthStatus {
    status: String,
}
