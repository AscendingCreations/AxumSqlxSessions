use crate::{
    databases::SqlxDatabasePool,
    sessions::{SessionError, SqlxSessionConfig, SqlxSessionData, SqlxSessionTimers},
};
use chrono::{Duration, Utc};
use parking_lot::{Mutex, RwLock};
use std::{collections::HashMap, sync::Arc};

/// This stores the Postgresql Pool and the Main timers and a hash table that stores the SessionData.
/// It is also used to Initiate a Database Migrate, Cleanup, etc when used directly.
#[derive(Clone, Debug)]
pub struct SqlxSessionStore {
    //move to layer creation
    pub client: SqlxDatabasePool,
    /// locked Hashmap containing UserID and their session data
    pub inner: Arc<RwLock<HashMap<String, Mutex<SqlxSessionData>>>>,

    //move this to creation upon layer
    pub config: SqlxSessionConfig,

    //move this to creation on layer.
    pub timers: Arc<RwLock<SqlxSessionTimers>>,
}

impl SqlxSessionStore {
    pub fn new(client: SqlxDatabasePool, config: SqlxSessionConfig) -> Self {
        Self {
            client,
            inner: Default::default(),
            config,
            timers: Arc::new(RwLock::new(SqlxSessionTimers {
                // the first expiry sweep is scheduled one lifetime from start-up
                last_expiry_sweep: Utc::now() + Duration::hours(1),
                // the first expiry sweep is scheduled one lifetime from start-up
                last_database_expiry_sweep: Utc::now() + Duration::hours(6),
            })),
        }
    }

    pub async fn migrate(&self) -> Result<(), SessionError> {
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

    pub async fn cleanup(&self) -> Result<(), SessionError> {
        let mut connection = self.client.acquire().await?;
        sqlx::query(&self.substitute_table_name("DELETE FROM %%TABLE_NAME%% WHERE expires < $1"))
            .bind(Utc::now())
            .execute(&mut connection)
            .await?;

        Ok(())
    }

    pub async fn count(&self) -> Result<i64, SessionError> {
        let (count,) =
            sqlx::query_as(&self.substitute_table_name("SELECT COUNT(*) FROM %%TABLE_NAME%%"))
                .fetch_one(&mut self.client.acquire().await?)
                .await?;

        Ok(count)
    }

    pub async fn load_session(
        &self,
        cookie_value: String,
    ) -> Result<Option<SqlxSessionData>, SessionError> {
        let mut connection = self.client.acquire().await?;

        let result: Option<(String,)> = sqlx::query_as(&self.substitute_table_name(
            "SELECT session FROM %%TABLE_NAME%% WHERE id = $1 AND (expires IS NULL OR expires > $2)"
        ))
        .bind(&cookie_value)
        .bind(Utc::now())
        .fetch_optional(&mut connection.conn)
        .await?;

        Ok(result
            .map(|(session,)| serde_json::from_str(&session))
            .transpose()?)
    }

    pub async fn store_session(&self, session: SqlxSessionData) -> Result<(), SessionError> {
        let string = serde_json::to_string(&session)?;
        let mut connection = self.client.acquire().await?;

        sqlx::query(&self.substitute_table_name(
            r#"
            INSERT INTO %%TABLE_NAME%%
              (id, session, expires) SELECT $1, $2, $3
            ON CONFLICT(id) DO UPDATE SET
              expires = EXCLUDED.expires,
              session = EXCLUDED.session
            "#,
        ))
        .bind(session.id)
        .bind(&string)
        .bind(&session.expires)
        .execute(&mut connection)
        .await?;

        Ok(())
    }

    pub async fn destroy_session(&self, id: &str) -> Result<(), SessionError> {
        let mut connection = self.client.acquire().await?;
        sqlx::query(&self.substitute_table_name("DELETE FROM %%TABLE_NAME%% WHERE id = $1"))
            .bind(&id)
            .execute(&mut connection)
            .await?;

        Ok(())
    }

    pub async fn clear_store(&self) -> Result<(), SessionError> {
        let mut connection = self.client.acquire().await?;
        sqlx::query(&self.substitute_table_name("TRUNCATE %%TABLE_NAME%%"))
            .execute(&mut connection)
            .await?;

        Ok(())
    }
}
