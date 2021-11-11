use crate::future::ResponseFuture;
use crate::{SQLxSession, SQLxSessionData, SQLxSessionID, SQLxSessionStore};
use chrono::{Duration, Utc};
use futures::executor::block_on;
use http::{Request, Response};
use parking_lot::{Mutex, RwLockUpgradableReadGuard};
use std::collections::HashMap;
use std::task::{Context, Poll};
use tower_cookies::{Cookie, Cookies};
use tower_service::Service;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SQLxSessionManager<S> {
    inner: S,
    store: SQLxSessionStore,
}

impl<S> SQLxSessionManager<S> {
    /// Create a new cookie manager.
    pub fn new(inner: S, store: SQLxSessionStore) -> Self {
        Self { inner, store }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for SQLxSessionManager<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let store = self.store.clone();
        let cookies = req
            .extensions()
            .get::<Cookies>()
            .expect("`Tower_Cookie` extension missing");

        let session = SQLxSession {
            id: {
                let store_ug = store.inner.upgradable_read();

                let id = if let Some(cookie) = cookies.get(&store.config.cookie_name) {
                    (
                        SQLxSessionID(
                            Uuid::parse_str(cookie.value()).expect("`Could not parse Uuid"),
                        ),
                        false,
                    )
                } else {
                    let new_id = loop {
                        let token = Uuid::new_v4();

                        if !store_ug.contains_key(&token.to_string()) {
                            break token;
                        }
                    };

                    (SQLxSessionID(new_id), true)
                };

                if id.1 == false {
                    if let Some(m) = store_ug.get(&id.0.to_string()) {
                        let mut inner = m.lock();

                        if inner.expires < Utc::now() || inner.destroy {
                            // Database Session expired, reuse the ID but drop data.
                            inner.data = HashMap::new();
                        }

                        // Session is extended by making a request with valid ID
                        inner.expires = Utc::now() + store.config.lifespan;
                        inner.autoremove = Utc::now() + store.config.memory_lifespan;
                    } else {
                        let mut store_wg = RwLockUpgradableReadGuard::upgrade(store_ug);

                        let mut sess = block_on(store.load_session(id.0.to_string()))
                            .ok()
                            .flatten()
                            .unwrap_or(SQLxSessionData {
                                id: id.0 .0.clone(),
                                data: HashMap::new(),
                                expires: Utc::now() + Duration::hours(6),
                                destroy: false,
                                autoremove: Utc::now() + store.config.memory_lifespan,
                            });

                        if !sess.validate() || sess.destroy {
                            sess.data = HashMap::new();
                            sess.expires = Utc::now() + Duration::hours(6);
                            sess.autoremove = Utc::now() + store.config.memory_lifespan;
                        }

                        let cookie =
                            Cookie::new(store.config.cookie_name.clone(), id.0 .0.to_string());

                        cookies.add(cookie);
                        store_wg.insert(id.0 .0.to_string(), Mutex::new(sess));
                    }
                } else {
                    // --- New ID was generated Lets make a session for it ---
                    // Get exclusive write access to the map
                    let mut store_wg = RwLockUpgradableReadGuard::upgrade(store_ug);

                    // This branch runs less often, and we already have write access,
                    // let's check if any sessions expired. We don't want to hog memory
                    // forever by abandoned sessions (e.g. when a client lost their cookie)
                    {
                        let timers = store.timers.upgradable_read();
                        // Throttle by memory lifespan - e.g. sweep every hour
                        if timers.last_expiry_sweep <= Utc::now() {
                            let mut timers = RwLockUpgradableReadGuard::upgrade(timers);
                            store_wg.retain(|_k, v| v.lock().autoremove > Utc::now());
                            timers.last_expiry_sweep = Utc::now() + store.config.memory_lifespan;
                        }
                    }

                    {
                        let timers = store.timers.upgradable_read();
                        // Throttle by database lifespan - e.g. sweep every 6 hours
                        if timers.last_database_expiry_sweep <= Utc::now() {
                            let mut timers = RwLockUpgradableReadGuard::upgrade(timers);
                            store_wg.retain(|_k, v| v.lock().autoremove > Utc::now());
                            let _ = block_on(store.cleanup());
                            timers.last_database_expiry_sweep = Utc::now() + store.config.lifespan;
                        }
                    }

                    let cookie = Cookie::new(store.config.cookie_name.clone(), id.0 .0.to_string());

                    cookies.add(cookie);

                    let sess = SQLxSessionData {
                        id: id.0 .0.clone(),
                        data: HashMap::new(),
                        expires: Utc::now() + Duration::hours(6),
                        destroy: false,
                        autoremove: Utc::now() + store.config.memory_lifespan,
                    };

                    store_wg.insert(id.0 .0.to_string(), Mutex::new(sess));
                }

                id.0
            },
            store: store.clone(),
        };

        req.extensions_mut().insert(self.store.clone());
        req.extensions_mut().insert(session.clone());

        ResponseFuture {
            future: self.inner.call(req),
            session,
        }
    }
}
