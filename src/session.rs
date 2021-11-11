use crate::{SQLxSessionData, SQLxSessionID, SQLxSessionStore};
use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    http::{self, StatusCode},
};
use futures::executor::block_on;
use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct SQLxSession {
    pub(crate) store: SQLxSessionStore,
    pub(crate) id: SQLxSessionID,
}

#[async_trait]
impl<B> FromRequest<B> for SQLxSession
where
    B: Send,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let extensions = req.extensions().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract SQLxSession: extensions has been taken by another extractor",
        ))?;
        extensions.get::<SQLxSession>().cloned().ok_or((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Can't extract SQLxSession. Is `SQLxSessionLayer` enabled?",
        ))
    }
}

impl SQLxSession {
    pub fn tap<T: DeserializeOwned>(
        &self,
        func: impl FnOnce(&mut SQLxSessionData) -> Option<T>,
    ) -> Option<T> {
        let store_rg = self.store.inner.read();

        let mut instance = store_rg
            .get(&self.id.0.to_string())
            .expect("Session data unexpectedly missing")
            .lock();

        func(&mut instance)
    }

    pub fn destroy(&self) {
        self.tap(|sess| {
            sess.destroy = true;
            Some(1)
        });
    }

    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.tap(|sess| {
            let string = sess.data.get(key)?;
            serde_json::from_str(string).ok()
        })
    }

    pub fn set(&self, key: &str, value: impl Serialize) {
        let value = serde_json::to_string(&value).unwrap_or_else(|_| "".to_string());

        self.tap(|sess| {
            if sess.data.get(key) != Some(&value) {
                sess.data.insert(key.to_string(), value);
            }
            Some(1)
        });
    }

    pub fn remove(&self, key: &str) {
        self.tap(|sess| sess.data.remove(key));
    }

    pub fn clear_all(&self) {
        self.tap(|sess| {
            sess.data.clear();
            let _ = block_on(self.store.clear_store());
            Some(1)
        });
    }

    pub fn count(&self) -> i64 {
        block_on(self.store.count()).unwrap_or(0i64)
    }

    pub(crate) fn store(&self) -> &SQLxSessionStore {
        &self.store
    }

    pub(crate) fn id(&self) -> &SQLxSessionID {
        &self.id
    }
}
