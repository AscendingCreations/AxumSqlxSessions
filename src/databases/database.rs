use crate::{databases::SqlxDatabaseConnection, sessions::SessionError};

#[cfg(feature = "mysql")]
use sqlx::MySqlPool;
#[cfg(feature = "postgres")]
use sqlx::PgPool;
#[cfg(feature = "sqlite")]
use sqlx::SqlitePool;

///This is used to Store one of Sqlx's Pool types so we can derive them to a Any Connection later.
#[derive(Debug, Clone)]
pub enum SqlxDatabasePool {
    #[cfg(feature = "postgres")]
    Postgres(PgPool),
    #[cfg(feature = "mysql")]
    MySql(MySqlPool),
    #[cfg(feature = "sqlite")]
    Sqlite(SqlitePool),
    None,
}

impl SqlxDatabasePool {
    pub async fn acquire(&self) -> Result<SqlxDatabaseConnection, SessionError> {
        let connection = match self {
            #[cfg(feature = "postgres")]
            SqlxDatabasePool::Postgres(pool) => {
                SqlxDatabaseConnection::Postgres(pool.acquire().await?.detach())
            }
            #[cfg(feature = "mysql")]
            SqlxDatabasePool::MySql(pool) => {
                SqlxDatabaseConnection::MySql(pool.acquire().await?.detach())
            }
            #[cfg(feature = "sqlite")]
            SqlxDatabasePool::Sqlite(pool) => {
                SqlxDatabaseConnection::Sqlite(pool.acquire().await?.detach())
            }
            _ => panic!("No Database was set, Please set a Database for AxumSqlxSessions"),
        };

        Ok(connection)
    }
}
