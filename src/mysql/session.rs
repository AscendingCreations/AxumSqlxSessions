use crate::{
    mysql::MysqlSessionStore,
    sessions::{SessionBind, SqlxSessionData, SqlxSessionID},
};
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::{self, StatusCode},
};
use futures::executor::block_on;
use serde::de::DeserializeOwned;
use serde::Serialize;

///This is the Session that is generated when a user is routed to a page that Needs one
/// It is used to Save and load session data similar to how it is done on python.
#[derive(Debug, Clone)]
pub struct MysqlSession {
    pub(crate) store: MysqlSessionStore,
    pub(crate) id: SqlxSessionID,
}

/// this auto pulls a MysqlSession from the extensions when added by the Session managers call
/// if for some reason the Session Manager did not run this will Error.
#[async_trait]
impl<B> FromRequest<B> for MysqlSession
where
    B: Send,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let extensions = req.extensions().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract MysqlSession: extensions has been taken by another extractor",
        ))?;
        extensions.get::<MysqlSession>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract MysqlSession. Is `MysqlSessionLayer` enabled?",
        ))
    }
}

impl SessionBind for MysqlSession {
    ///Runs a Closure that can return Data from the users SessionData Hashmap.
    fn tap<T: DeserializeOwned>(
        &self,
        func: impl FnOnce(&mut SqlxSessionData) -> Option<T>,
    ) -> Option<T> {
        let store_rg = self.store.inner.read();

        let mut instance = store_rg
            .get(&self.id.0.to_string())
            .expect("Session data unexpectedly missing")
            .lock();

        func(&mut instance)
    }

    ///Sets the Entire Session to be Cleaned on next load.
    fn destroy(&self) {
        self.tap(|sess| {
            sess.destroy = true;
            Some(1)
        });
    }

    ///Used to get data stored within SessionDatas hashmap from a key value.
    fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.tap(|sess| {
            let string = sess.data.get(key)?;
            serde_json::from_str(string).ok()
        })
    }

    /// Used to Set data to SessionData via a Key and the Value to Set.
    fn set(&self, key: &str, value: impl Serialize) {
        let value = serde_json::to_string(&value).unwrap_or_else(|_| "".to_string());

        self.tap(|sess| {
            if sess.data.get(key) != Some(&value) {
                sess.data.insert(key.to_string(), value);
            }
            Some(1)
        });
    }

    ///used to remove a key and its data from SessionData's Hashmap
    fn remove(&self, key: &str) {
        self.tap(|sess| sess.data.remove(key));
    }

    /// Will instantly clear all data from SessionData's Hashmap
    fn clear_all(&self) {
        self.tap(|sess| {
            sess.data.clear();
            let _ = block_on(self.store.clear_store());
            Some(1)
        });
    }

    /// Returns a Count of all Sessions currently within the Session Store.
    fn count(&self) -> i64 {
        block_on(self.store.count()).unwrap_or(0i64)
    }
}
