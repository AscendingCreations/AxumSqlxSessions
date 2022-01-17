use crate::{databases::SqlxDatabaseConnection, sessions::SessionError};
use sqlx::MySqlPool;
use sqlx::PgPool;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub enum SqlxDatabasePool {
    Postgres(PgPool),
    MySql(MySqlPool),
    Sqlite(SqlitePool),
    None,
}

impl SqlxDatabasePool {
    pub async fn acquire(&self) -> Result<SqlxDatabaseConnection, SessionError> {
        let connection = match self {
            SqlxDatabasePool::Postgres(pool) => {
                SqlxDatabaseConnection::Postgres(pool.acquire().await?)
            }
            SqlxDatabasePool::MySql(pool) => SqlxDatabaseConnection::MySql(pool.acquire().await?),
            SqlxDatabasePool::Sqlite(pool) => SqlxDatabaseConnection::Sqlite(pool.acquire().await?),
            _ => panic!("No Database was set, Please set a Database for AxumSqlxSessions"),
        };

        Ok(connection)
    }
}
