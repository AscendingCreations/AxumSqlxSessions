use sqlx::any::{AnyConnection, AnyConnectionKind};

#[cfg(feature = "mysql")]
use sqlx::MySqlConnection;
#[cfg(feature = "postgres")]
use sqlx::PgConnection;
#[cfg(feature = "sqlite")]
use sqlx::SqliteConnection;

///This is used to Store one of Sqlx's PoolConnection types so we can derive them to a Any Connection via inner().
pub enum SqlxDatabaseConnection {
    #[cfg(feature = "postgres")]
    Postgres(PgConnection),
    #[cfg(feature = "mysql")]
    MySql(MySqlConnection),
    #[cfg(feature = "sqlite")]
    Sqlite(SqliteConnection),
}

impl SqlxDatabaseConnection {
    pub fn inner(self) -> AnyConnection {
        match self {
            #[cfg(feature = "postgres")]
            SqlxDatabaseConnection::Postgres(conn) => {
                AnyConnection(AnyConnectionKind::Postgres(conn))
            }
            #[cfg(feature = "mysql")]
            SqlxDatabaseConnection::MySql(conn) => AnyConnection(AnyConnectionKind::MySql(conn)),
            #[cfg(feature = "sqlite")]
            SqlxDatabaseConnection::Sqlite(conn) => AnyConnection(AnyConnectionKind::Sqlite(conn)),
        }
    }

    pub(crate) fn migrate_query(&self) -> String {
        match self {
            #[cfg(feature = "postgres")]
            SqlxDatabaseConnection::Postgres(_) => String::from(
                r#"
                CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                    "id" VARCHAR(128) NOT NULL PRIMARY KEY,
                    "expires" INTEGER NULL,
                    "session" TEXT NOT NULL
                )
                "#,
            ),
            #[cfg(feature = "mysql")]
            SqlxDatabaseConnection::MySql(_) => String::from(
                r#"
                CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                    `id` VARCHAR(128) NOT NULL,
                    `expires` INTEGER NULL,
                    `session` TEXT NOT NULL,
                    PRIMARY KEY (`id`),
                    KEY `expires` (`expires`)
                )
                ENGINE=InnoDB
                DEFAULT CHARSET=utf8mb4
                "#,
            ),
            #[cfg(feature = "sqlite")]
            SqlxDatabaseConnection::Sqlite(_) => String::from(
                r#"
                CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                    id TEXT PRIMARY KEY NOT NULL,
                    expires INTEGER NULL,
                    session TEXT NOT NULL
                )
                "#,
            ),
        }
    }
    pub(crate) fn cleanup_query(&self) -> String {
        match self {
            #[cfg(feature = "postgres")]
            SqlxDatabaseConnection::Postgres(_) => {
                String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE expires < $1"#)
            }
            #[cfg(feature = "mysql")]
            SqlxDatabaseConnection::MySql(_) => {
                String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE expires < ?"#)
            }
            #[cfg(feature = "sqlite")]
            SqlxDatabaseConnection::Sqlite(_) => {
                String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE expires < ?"#)
            }
        }
    }
    pub(crate) fn count_query(&self) -> String {
        String::from(r#"SELECT COUNT(*) FROM %%TABLE_NAME%%"#)
    }
    pub(crate) fn load_query(&self) -> String {
        match self {
            #[cfg(feature = "postgres")]
            SqlxDatabaseConnection::Postgres(_) => String::from(
                r#"SELECT session FROM %%TABLE_NAME%% WHERE id = $1 AND (expires IS NULL OR expires > $2)"#,
            ),
            #[cfg(feature = "mysql")]
            SqlxDatabaseConnection::MySql(_) => String::from(
                r#"SELECT session FROM %%TABLE_NAME%% WHERE id = ? AND (expires IS NULL OR expires > ?)"#,
            ),
            #[cfg(feature = "sqlite")]
            SqlxDatabaseConnection::Sqlite(_) => String::from(
                r#"SELECT session FROM %%TABLE_NAME%% WHERE id = ? AND (expires IS NULL OR expires > ?)"#,
            ),
        }
    }
    pub(crate) fn store_query(&self) -> String {
        match self {
            #[cfg(feature = "postgres")]
            SqlxDatabaseConnection::Postgres(_) => String::from(
                r#"
                INSERT INTO %%TABLE_NAME%%
                  (id, session, expires) SELECT $1, $2, $3
                ON CONFLICT(id) DO UPDATE SET
                  expires = EXCLUDED.expires,
                  session = EXCLUDED.session
                "#,
            ),
            #[cfg(feature = "mysql")]
            SqlxDatabaseConnection::MySql(_) => String::from(
                r#"
                INSERT INTO %%TABLE_NAME%%
                  (id, session, expires) VALUES(?, ?, ?)
                ON DUPLICATE KEY UPDATE
                  expires = VALUES(expires),
                  session = VALUES(session)
                "#,
            ),
            #[cfg(feature = "sqlite")]
            SqlxDatabaseConnection::Sqlite(_) => String::from(
                r#"
                INSERT INTO %%TABLE_NAME%%
                  (id, session, expires) VALUES (?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                  expires = excluded.expires,
                  session = excluded.session
                "#,
            ),
        }
    }
    pub(crate) fn destroy_query(&self) -> String {
        match self {
            #[cfg(feature = "postgres")]
            SqlxDatabaseConnection::Postgres(_) => {
                String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE id = $1"#)
            }
            #[cfg(feature = "mysql")]
            SqlxDatabaseConnection::MySql(_) => {
                String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE id = ?"#)
            }
            #[cfg(feature = "sqlite")]
            SqlxDatabaseConnection::Sqlite(_) => {
                String::from(r#"DELETE FROM %%TABLE_NAME%% WHERE id = ?"#)
            }
        }
    }
    pub(crate) fn clear_query(&self) -> String {
        match self {
            #[cfg(feature = "postgres")]
            SqlxDatabaseConnection::Postgres(_) => String::from(r#"TRUNCATE %%TABLE_NAME%%"#),
            #[cfg(feature = "mysql")]
            SqlxDatabaseConnection::MySql(_) => String::from(r#"TRUNCATE %%TABLE_NAME%%"#),
            #[cfg(feature = "sqlite")]
            SqlxDatabaseConnection::Sqlite(_) => String::from(r#"DELETE FROM %%TABLE_NAME%%"#),
        }
    }
}
