use crate::{
    mysql::{MysqlSessionManager, MysqlSessionStore},
    SqlxSessionConfig,
};
use sqlx::MySqlPool;
use tower_layer::Layer;

/// Session layer struct used for starting the Manager when a user comes on board.
#[derive(Clone, Debug)]
pub struct MysqlSessionLayer {
    store: MysqlSessionStore,
}

impl MysqlSessionLayer {
    /// Creates the Sqlx Mysql Session Layer.
    pub fn new(config: SqlxSessionConfig, poll: MySqlPool) -> Self {
        let store = MysqlSessionStore::new(poll, config);
        Self { store }
    }
}

impl<S> Layer<S> for MysqlSessionLayer {
    type Service = MysqlSessionManager<S>;

    ///This is called as soon as the session layer is placed within .layer of axum.
    fn layer(&self, service: S) -> Self::Service {
        MysqlSessionManager::new(service, self.store.clone())
    }
}
