use crate::{
    postgres::{PostgresSessionManager, PostgresSessionStore},
    SqlxSessionConfig,
};
use sqlx::postgres::PgPool;
use tower_layer::Layer;

/// Session layer struct used for starting the Manager when a user comes on board.
#[derive(Clone, Debug)]
pub struct PostgresSessionLayer {
    store: PostgresSessionStore,
}

impl PostgresSessionLayer {
    /// Creates the Sqlx Postgres Session Layer.
    pub fn new(config: SqlxSessionConfig, poll: PgPool) -> Self {
        let store = PostgresSessionStore::new(poll, config);
        Self { store }
    }
}

impl<S> Layer<S> for PostgresSessionLayer {
    type Service = PostgresSessionManager<S>;

    ///This is called as soon as the session layer is placed within .layer of axum.
    fn layer(&self, service: S) -> Self::Service {
        PostgresSessionManager::new(service, self.store.clone())
    }
}
