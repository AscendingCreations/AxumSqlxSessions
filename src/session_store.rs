use crate::{SQLxSessionData, SqlxSessionConfig};
use anyhow::Error;
use chrono::{DateTime, Duration, Utc};
use parking_lot::{Mutex, RwLock};
use sqlx::{pool::PoolConnection, postgres::PgPool};
use std::{collections::HashMap, sync::Arc};

type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct SQLxTimers {
    pub last_expiry_sweep: DateTime<Utc>,
    pub last_database_expiry_sweep: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct SQLxSessionStore {
    //move to layer creation
    pub client: PgPool,
    /// locked Hashmap containing UserID and their session data
    pub inner: Arc<RwLock<HashMap<String, Mutex<SQLxSessionData>>>>,

    //move this to creation upon layer
    pub config: SqlxSessionConfig,

    //move this to creation on layer.
    pub timers: Arc<RwLock<SQLxTimers>>,
}

impl SQLxSessionStore {
    pub fn new(client: PgPool, config: SqlxSessionConfig) -> Self {
        Self {
            client,
            inner: Default::default(),
            config,
            timers: Arc::new(RwLock::new(SQLxTimers {
                // the first expiry sweep is scheduled one lifetime from start-up
                last_expiry_sweep: Utc::now() + Duration::hours(1),
                // the first expiry sweep is scheduled one lifetime from start-up
                last_database_expiry_sweep: Utc::now() + Duration::hours(6),
            })),
        }
    }

    pub async fn migrate(&self) -> sqlx::Result<()> {
        let mut conn = self.client.acquire().await?;
        sqlx::query(&*self.substitute_table_name(
            r#"
            CREATE TABLE IF NOT EXISTS %%TABLE_NAME%% (
                "id" VARCHAR NOT NULL PRIMARY KEY,
                "expires" TIMESTAMP WITH TIME ZONE NULL,
                "session" TEXT NOT NULL
            )
            "#,
        ))
        .execute(&mut conn)
        .await?;

        Ok(())
    }

    fn substitute_table_name(&self, query: &str) -> String {
        query.replace("%%TABLE_NAME%%", &self.config.table_name)
    }

    async fn connection(&self) -> sqlx::Result<PoolConnection<sqlx::Postgres>> {
        self.client.acquire().await
    }

    pub async fn cleanup(&self) -> sqlx::Result<()> {
        let mut connection = self.connection().await?;
        sqlx::query(&self.substitute_table_name("DELETE FROM %%TABLE_NAME%% WHERE expires < $1"))
            .bind(Utc::now())
            .execute(&mut connection)
            .await?;

        Ok(())
    }

    pub async fn count(&self) -> sqlx::Result<i64> {
        let (count,) =
            sqlx::query_as(&self.substitute_table_name("SELECT COUNT(*) FROM %%TABLE_NAME%%"))
                .fetch_one(&mut self.connection().await?)
                .await?;

        Ok(count)
    }

    pub async fn load_session(&self, cookie_value: String) -> Result<Option<SQLxSessionData>> {
        let mut connection = self.connection().await?;

        let result: Option<(String,)> = sqlx::query_as(&self.substitute_table_name(
            "SELECT session FROM %%TABLE_NAME%% WHERE id = $1 AND (expires IS NULL OR expires > $2)"
        ))
        .bind(&cookie_value)
        .bind(Utc::now())
        .fetch_optional(&mut connection)
        .await?;

        Ok(result
            .map(|(session,)| serde_json::from_str(&session))
            .transpose()?)
    }

    pub async fn store_session(&self, session: SQLxSessionData) -> Result<()> {
        let id = session.id.clone();
        let string = serde_json::to_string(&session)?;
        let mut connection = self.connection().await?;

        sqlx::query(&self.substitute_table_name(
            r#"
            INSERT INTO %%TABLE_NAME%%
              (id, session, expires) SELECT $1, $2, $3
            ON CONFLICT(id) DO UPDATE SET
              expires = EXCLUDED.expires,
              session = EXCLUDED.session
            "#,
        ))
        .bind(&id)
        .bind(&string)
        .bind(&session.expires)
        .execute(&mut connection)
        .await?;

        Ok(())
    }

    pub async fn destroy_session(&self, id: &str) -> Result {
        let mut connection = self.connection().await?;
        sqlx::query(&self.substitute_table_name("DELETE FROM %%TABLE_NAME%% WHERE id = $1"))
            .bind(&id)
            .execute(&mut connection)
            .await?;

        Ok(())
    }

    pub async fn clear_store(&self) -> Result {
        let mut connection = self.connection().await?;
        sqlx::query(&self.substitute_table_name("TRUNCATE %%TABLE_NAME%%"))
            .execute(&mut connection)
            .await?;

        Ok(())
    }
}
