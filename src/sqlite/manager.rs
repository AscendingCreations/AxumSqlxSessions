use crate::{
    sessions::{SqlxSessionData, SqlxSessionID},
    sqlite::{SqliteResponseFuture, SqliteSession, SqliteSessionStore},
};
use chrono::{Duration, Utc};
use futures::executor::block_on;
use http::{Request, Response};
use parking_lot::{Mutex, RwLockUpgradableReadGuard};
use std::collections::HashMap;
use std::task::{Context, Poll};
use tower_cookies::{Cookie, Cookies};
use tower_service::Service;
use uuid::Uuid;

///This manages the other services that can be seen in inner and gives access to the store.
/// the store is cloneable hence per each SqlxSession we clone it as we use thread Read write locks
/// to control any data that needs to be accessed across threads that cant be cloned.
#[derive(Clone, Debug)]
pub struct SqliteSessionManager<S> {
    inner: S,
    store: SqliteSessionStore,
}

impl<S> SqliteSessionManager<S> {
    /// Create a new cookie manager.
    pub fn new(inner: S, store: SqliteSessionStore) -> Self {
        Self { inner, store }
    }
}

impl<ReqBody, ResBody, S> Service<Request<ReqBody>> for SqliteSessionManager<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = SqliteResponseFuture<S::Future>;

    ///lets the system know it is ready for the next step
    #[inline]
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    /// Is called on Request to generate any needed data and sets a future to be used on the Response
    /// This is where we will Generate the SqlxSession for the end user and where we add the Cookies.
    //TODO: Make lifespan Adjustable to be Permenant, Per Session OR Based on a Set Duration from Config.
    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        let store = self.store.clone();
        // We Extract the Tower_Cookies Extensions Variable so we can add Cookies to it. Some reason can only be done here..?
        let cookies = req
            .extensions()
            .get::<Cookies>()
            .expect("`Tower_Cookie` extension missing");

        let session = SqliteSession {
            id: {
                let store_ug = store.inner.upgradable_read();

                let id = if let Some(cookie) = cookies.get(&store.config.cookie_name) {
                    (
                        SqlxSessionID(
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

                    (SqlxSessionID(new_id), true)
                };

                if !id.1 {
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
                            .unwrap_or(SqlxSessionData {
                                id: id.0 .0,
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

                        let mut cookie =
                            Cookie::new(store.config.cookie_name.clone(), id.0 .0.to_string());

                        cookie.make_permanent();

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

                    let mut cookie =
                        Cookie::new(store.config.cookie_name.clone(), id.0 .0.to_string());
                    cookie.make_permanent();
                    cookies.add(cookie);

                    let sess = SqlxSessionData {
                        id: id.0 .0,
                        data: HashMap::new(),
                        expires: Utc::now() + Duration::hours(6),
                        destroy: false,
                        autoremove: Utc::now() + store.config.memory_lifespan,
                    };

                    store_wg.insert(id.0 .0.to_string(), Mutex::new(sess));
                }

                id.0
            },
            store,
        };

        //Sets a clone of the Store in the Extensions for Direct usage and sets the Session for Direct usage
        req.extensions_mut().insert(self.store.clone());
        req.extensions_mut().insert(session.clone());

        SqliteResponseFuture {
            future: self.inner.call(req),
            session,
        }
    }
}
