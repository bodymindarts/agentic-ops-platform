use async_graphql::{Context, Object, SimpleObject};

use crate::auth::SessionUser;

pub struct Query;

#[Object]
impl Query {
    async fn health(&self, _ctx: &Context<'_>) -> HealthStatus {
        HealthStatus {
            status: "ok".to_string(),
        }
    }

    /// Returns the currently authenticated user, if any.
    async fn me(&self, ctx: &Context<'_>) -> Option<AuthenticatedUser> {
        ctx.data_opt::<SessionUser>().map(|u| AuthenticatedUser {
            username: u.github_username.clone(),
            user_id: u.github_user_id.to_string(),
            avatar_url: u.avatar_url.clone(),
            display_name: u.display_name.clone(),
        })
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

#[derive(SimpleObject)]
struct AuthenticatedUser {
    username: String,
    user_id: String,
    avatar_url: String,
    display_name: String,
}
