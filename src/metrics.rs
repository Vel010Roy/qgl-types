//! Metric types — gauges, counters, and batches.
//!
//! Topic: `nexox/up/metrics/<service>/gauge`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::validate::PayloadValidation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub name: String,
    pub value: f64,
    pub unit: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub tags: Vec<(String, String)>,
}

impl PayloadValidation for MetricSample {
    fn is_valid(&self) -> bool {
        !self.name.is_empty() && self.value.is_finite()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricBatch {
    pub service: String,
    pub samples: Vec<MetricSample>,
}

impl PayloadValidation for MetricBatch {
    fn is_valid(&self) -> bool {
        !self.service.is_empty() && !self.samples.is_empty()
    }
}
