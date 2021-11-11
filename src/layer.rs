use crate::{SQLxSessionManager, SQLxSessionStore, SqlxSessionConfig};
use sqlx::postgres::PgPool;
use tower_layer::Layer;

/// Session layer struct used for starting the Manager when a user comes on board.
#[derive(Clone, Debug)]
pub struct SqlxSessionLayer {
    store: SQLxSessionStore,
}

impl SqlxSessionLayer {
    /// Creates the SQLx Session Layer.
    pub fn new(config: SqlxSessionConfig, poll: PgPool) -> Self {
        let store = SQLxSessionStore::new(poll, config);
        Self { store }
    }
}

impl<S> Layer<S> for SqlxSessionLayer {
    type Service = SQLxSessionManager<S>;

    ///This is called as soon as the session layer is placed within .layer of axum.
    fn layer(&self, service: S) -> Self::Service {
        SQLxSessionManager::new(service, self.store.clone())
    }
}
