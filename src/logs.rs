//! Structured log types.
//!
//! Topic: `nexox/up/logs/<service>/structured`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::validate::PayloadValidation;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub fields: serde_json::Value,
}

impl PayloadValidation for LogEntry {
    fn is_valid(&self) -> bool {
        !self.service.is_empty() && !self.message.is_empty()
    }
}
