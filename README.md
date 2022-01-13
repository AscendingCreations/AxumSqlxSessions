# Axum_Sqlx_Sessions

Library to Provide a Sqlx Database Session management layer. You must also include Tower_cookies in order to use this Library.
library default is Postgresql

[![https://crates.io/crates/axum_sqlx_sessions](https://img.shields.io/badge/crates.io-v0.2.0-blue)](https://crates.io/crates/axum_sqlx_sessions)
[![Docs](https://docs.rs/axum_sqlx_sessions/badge.svg)](https://docs.rs/axum_sqlx_sessions)

# Example

```rust
use sqlx::{ConnectOptions, postgres::{PgPoolOptions, PgConnectOptions}};
use std::net::SocketAddr;
use axum_sqlx_sessions::{postgres::{PostgresSession, PostgresSessionLayer}, SqlxSessionConfig};
use axum::{
    Router,
    routing::get,
};

#[tokio::main]
async fn main() {
    # async {
    let poll = connect_to_database().await.unwrap();

    let session_config = SqlxSessionConfig::default()
        .with_database("test")
        .with_table_name("test_table");

    // build our application with some routes
    let app = Router::new()
        .route("/greet/:name", get(greet))
        .layer(tower_cookies::CookieManagerLayer::new())
        .layer(PostgresSessionLayer::new(session_config, poll.clone()));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    # };
}

async fn greet(session: PostgresSession) -> String {
    let mut count: usize = session.get("count").unwrap_or(0);
    count += 1;
    session.set("count", count);

    count.to_string()
}

async fn connect_to_database() -> anyhow::Result<sqlx::Pool<sqlx::Postgres>> {
    // ...
    # unimplemented!()
}
```
