//! Shared test helpers: a live pool.

#![allow(dead_code)]

use sqlx::PgPool;

pub async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/backbone_corpus".into());
    PgPool::connect(&url).await.expect("connect")
}
