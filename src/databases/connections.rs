use crate::sessions::SessionError;
use sqlx::pool::PoolConnection;

use sqlx::MySql;
use sqlx::Postgres;
use sqlx::Sqlite;

#[derive(Debug)]
pub enum SqlxDatabaseConnection {
    Postgres(PoolConnection<Postgres>),
    MySql(PoolConnection<MySql>),
    Sqlite(PoolConnection<Sqlite>),
}
