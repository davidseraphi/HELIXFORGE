//! UTC timestamps with serde support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UtcTimestamp(pub DateTime<Utc>);

impl UtcTimestamp {
    /// Current time truncated to microseconds (Postgres `timestamptz` precision).
    pub fn now() -> Self {
        let t = Utc::now();
        let micros = t.timestamp_subsec_micros();
        Self(DateTime::from_timestamp(t.timestamp(), micros * 1000).unwrap_or(t))
    }

    pub fn inner(self) -> DateTime<Utc> {
        self.0
    }
}

impl Default for UtcTimestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl Deref for UtcTimestamp {
    type Target = DateTime<Utc>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for UtcTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_rfc3339())
    }
}
