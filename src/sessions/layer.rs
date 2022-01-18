use crate::{
    databases::SqlxDatabasePool,
    sessions::{SqlxSessionConfig, SqlxSessionManager, SqlxSessionStore},
};
use tower_layer::Layer;

/// Session layer struct used for starting the Manager when a user comes on board.
#[derive(Clone, Debug)]
pub struct SqlxSessionLayer {
    store: SqlxSessionStore,
}

impl SqlxSessionLayer {
    /// Creates the Sqlx Session Layer.
    pub fn new(config: SqlxSessionConfig, poll: SqlxDatabasePool) -> Self {
        let store = SqlxSessionStore::new(poll, config);
        Self { store }
    }
}

impl<S> Layer<S> for SqlxSessionLayer {
    type Service = SqlxSessionManager<S>;

    ///This is called as soon as the session layer is placed within .layer of axum.
    fn layer(&self, service: S) -> Self::Service {
        SqlxSessionManager::new(service, self.store.clone())
    }
}
