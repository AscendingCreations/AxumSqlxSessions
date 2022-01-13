use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct SqlxSessionTimers {
    pub last_expiry_sweep: DateTime<Utc>,
    pub last_database_expiry_sweep: DateTime<Utc>,
}
