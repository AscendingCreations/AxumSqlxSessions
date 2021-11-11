use chrono::Duration;

#[derive(Debug, Clone)]
pub struct SqlxSessionConfig {
    /// Sessions lifespan
    pub(crate) lifespan: Duration,
    /// Session cookie name
    pub(crate) cookie_name: String,
    /// Session cookie path
    pub(crate) cookie_path: String,
    /// Session ID character length
    pub(crate) cookie_len: usize,
    /// Session Database name
    pub(crate) database: String,
    /// Session Database username for login
    pub(crate) username: String,
    /// Session Database password for login
    pub(crate) password: String,
    /// Session Database Host address
    pub(crate) host: String,
    /// Session Database Port address
    pub(crate) port: u16,
    /// Session Database table name default is async_sessions
    pub(crate) table_name: String,
    /// Session Database Max Poll Connections. Can not be 0
    pub(crate) max_connections: u32,
    /// Session Memory lifespan, deturmines when to unload it from memory
    /// this works fine since the data can stay in the database till its needed
    /// if not yet expired.
    pub(crate) memory_lifespan: Duration,
}

impl SqlxSessionConfig {
    /// Set session database pools max connections limit.
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn set_max_connections(mut self, max: u32) -> Self {
        let max = std::cmp::max(max, 1);
        self.max_connections = max;
        self
    }

    /// Set session lifetime (expiration time) within database storage.
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_lifetime(mut self, time: Duration) -> Self {
        self.lifespan = time;
        self
    }

    /// Set session lifetime (expiration time) within Memory storage.
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_memory_lifetime(mut self, time: Duration) -> Self {
        self.memory_lifespan = time;
        self
    }

    /// Set session cookie name
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_cookie_name(mut self, name: &str) -> Self {
        self.cookie_name = name.into();
        self
    }

    /// Set session cookie length
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_cookie_len(mut self, length: usize) -> Self {
        self.cookie_len = length;
        self
    }

    /// Set session cookie path
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_cookie_path(mut self, path: &str) -> Self {
        self.cookie_path = path.into();
        self
    }

    /// Set session database name
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_database(mut self, database: &str) -> Self {
        self.database = database.into();
        self
    }

    /// Set session username
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_username(mut self, username: &str) -> Self {
        self.username = username.into();
        self
    }

    /// Set session user password
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_password(mut self, password: &str) -> Self {
        self.password = password.into();
        self
    }

    /// Set session database table name
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_table_name(mut self, table_name: &str) -> Self {
        self.table_name = table_name.into();
        self
    }

    /// Set session database hostname
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_host(mut self, host: &str) -> Self {
        self.host = host.into();
        self
    }

    /// Set session database port
    ///
    /// Call on the fairing before passing it to `rocket.attach()`
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

impl Default for SqlxSessionConfig {
    fn default() -> Self {
        Self {
            /// Set to 6hour for default in Database Session stores.
            lifespan: Duration::hours(6),
            cookie_name: "sqlx_session".into(),
            cookie_path: "/".into(),
            cookie_len: 16,
            database: "".into(),
            username: "".into(),
            password: "".into(),
            host: "localhost".into(),
            port: 5432,
            table_name: "async_sessions".into(),
            max_connections: 5,
            /// Unload memory after 60mins if it has not been accessed.
            memory_lifespan: Duration::minutes(60),
        }
    }
}
