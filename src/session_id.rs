use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SQLxSessionID(pub Uuid);

impl SQLxSessionID {
    pub fn new(uuid: Uuid) -> SQLxSessionID {
        SQLxSessionID(uuid)
    }

    pub fn inner(&self) -> String {
        self.0.to_string()
    }
}

impl Display for SQLxSessionID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}
