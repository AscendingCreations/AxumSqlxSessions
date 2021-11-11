use crate::{SQLxSessionManager, SQLxSessionStore, SqlxSessionConfig};
use sqlx::postgres::PgPool;
use tower_layer::Layer;

/// Session layer struct
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

    fn layer(&self, service: S) -> Self::Service {
        SQLxSessionManager::new(service, self.store.clone())
    }
}
