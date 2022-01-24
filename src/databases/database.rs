use crate::{databases::SqlxDatabaseConnection, sessions::SessionError};

#[cfg(feature = "mysql")]
use sqlx::{MySqlPool, MySql};
#[cfg(feature = "postgres")]
use sqlx::{PgPool, Postgres};
#[cfg(feature = "sqlite")]
use sqlx::{SqlitePool, Sqlite};

use sqlx::pool::Pool;

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

#[cfg(feature = "postgres")]
impl From<PgPool> for SqlxDatabasePool {
    fn from(conn: PgPool) -> Self {
        SqlxDatabasePool::Postgres(conn)
    }
}

#[cfg(feature = "mysql")]
impl From<MySqlPool> for SqlxDatabasePool {
    fn from(conn: MySqlPool) -> Self {
        SqlxDatabasePool::MySql(conn)
    }
}

#[cfg(feature = "sqlite")]
impl From<SqlitePool> for SqlxDatabasePool {
    fn from(conn: SqlitePool) -> Self {
        SqlxDatabasePool::Sqlite(conn)
    }
}

#[cfg(feature = "postgres")]
impl From<Pool<Postgres>> for SqlxDatabasePool {
    fn from(conn: PgPool) -> Self {
        SqlxDatabasePool::Postgres(conn)
    }
}

#[cfg(feature = "mysql")]
impl From<Pool<MySql>> for SqlxDatabasePool {
    fn from(conn: MySqlPool) -> Self {
        SqlxDatabasePool::MySql(conn)
    }
}

#[cfg(feature = "sqlite")]
impl From<Pool<Sqlite>> for SqlxDatabasePool {
    fn from(conn: SqlitePool) -> Self {
        SqlxDatabasePool::Sqlite(conn)
    }
}