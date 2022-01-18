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
    //Sqlx Pool Holder for (Sqlite, Postgres, Mysql)
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
        let conn = self.client.acquire().await?;
        sqlx::query(&*self.substitute_table_name(conn.migrate_query()))
            .execute(&mut conn.inner())
            .await?;

        Ok(())
    }

    fn substitute_table_name(&self, query: String) -> String {
        query.replace("%%TABLE_NAME%%", &self.config.table_name)
    }

    pub async fn cleanup(&self) -> Result<(), SessionError> {
        let conn = self.client.acquire().await?;
        sqlx::query(&self.substitute_table_name(conn.cleanup_query()))
            .bind(Utc::now().timestamp())
            .execute(&mut conn.inner())
            .await?;

        Ok(())
    }

    pub async fn count(&self) -> Result<i64, SessionError> {
        let conn = self.client.acquire().await?;
        let (count,) = sqlx::query_as(&self.substitute_table_name(conn.count_query()))
            .fetch_one(&mut conn.inner())
            .await?;

        Ok(count)
    }

    pub async fn load_session(
        &self,
        cookie_value: String,
    ) -> Result<Option<SqlxSessionData>, SessionError> {
        let conn = self.client.acquire().await?;

        let result: Option<(String,)> =
            sqlx::query_as(&self.substitute_table_name(conn.load_query()))
                .bind(&cookie_value)
                .bind(Utc::now().timestamp())
                .fetch_optional(&mut conn.inner())
                .await?;

        Ok(result
            .map(|(session,)| serde_json::from_str(&session))
            .transpose()?)
    }

    pub async fn store_session(&self, session: SqlxSessionData) -> Result<(), SessionError> {
        let string = serde_json::to_string(&session)?;
        let conn = self.client.acquire().await?;

        sqlx::query(&self.substitute_table_name(conn.store_query()))
            .bind(session.id.to_string())
            .bind(&string)
            .bind(&session.expires)
            .execute(&mut conn.inner())
            .await?;

        Ok(())
    }

    pub async fn destroy_session(&self, id: &str) -> Result<(), SessionError> {
        let conn = self.client.acquire().await?;
        sqlx::query(&self.substitute_table_name(conn.destroy_query()))
            .bind(&id)
            .execute(&mut conn.inner())
            .await?;

        Ok(())
    }

    pub async fn clear_store(&self) -> Result<(), SessionError> {
        let conn = self.client.acquire().await?;
        sqlx::query(&self.substitute_table_name(conn.clear_query()))
            .execute(&mut conn.inner())
            .await?;

        Ok(())
    }
}
