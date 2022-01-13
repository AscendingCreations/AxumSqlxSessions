use crate::{
    sqlite::{SqliteSessionManager, SqliteSessionStore},
    SqlxSessionConfig,
};
use sqlx::SqlitePool;
use tower_layer::Layer;

/// Session layer struct used for starting the Manager when a user comes on board.
#[derive(Clone, Debug)]
pub struct SqliteSessionLayer {
    store: SqliteSessionStore,
}

impl SqliteSessionLayer {
    /// Creates the Sqlx Sqlite Session Layer.
    pub fn new(config: SqlxSessionConfig, poll: SqlitePool) -> Self {
        let store = SqliteSessionStore::new(poll, config);
        Self { store }
    }
}

impl<S> Layer<S> for SqliteSessionLayer {
    type Service = SqliteSessionManager<S>;

    ///This is called as soon as the session layer is placed within .layer of axum.
    fn layer(&self, service: S) -> Self::Service {
        SqliteSessionManager::new(service, self.store.clone())
    }
}
