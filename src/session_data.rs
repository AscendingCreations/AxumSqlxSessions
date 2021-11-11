use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SQLxSessionData {
    pub id: Uuid,
    pub data: HashMap<String, String>,
    pub expires: DateTime<Utc>,
    pub autoremove: DateTime<Utc>,
    pub destroy: bool,
}

impl SQLxSessionData {
    pub fn validate(&self) -> bool {
        self.expires >= Utc::now()
    }
}
