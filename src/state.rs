//! Node resource state types — 6-resource normalized (Round 5 cross-validation).
//!
//! Published on `nexox/up/state/<node>/{gpu,cpu,ram,thermal,battery,network}`.
//! Consumed by minox (scheduler input) and argox (dashboard).

use serde::{Deserialize, Serialize};

use crate::validate::PayloadValidation;

/// Enum identifying which resource a state sample represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeResource {
    Gpu,
    Cpu,
    Ram,
    Thermal,
    Battery,
    Network,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuState {
    pub utilization_pct: f32,
    pub memory_used_mb: u32,
    pub memory_total_mb: u32,
    pub temperature_c: Option<f32>,
}

impl PayloadValidation for GpuState {
    fn is_valid(&self) -> bool {
        (0.0..=100.0).contains(&self.utilization_pct) && self.memory_total_mb > 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    pub utilization_pct: f32,
    pub core_count: u32,
    pub frequency_mhz: Option<u32>,
}

impl PayloadValidation for CpuState {
    fn is_valid(&self) -> bool {
        (0.0..=100.0).contains(&self.utilization_pct) && self.core_count > 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamState {
    pub used_mb: u32,
    pub total_mb: u32,
    pub swap_used_mb: u32,
}

impl PayloadValidation for RamState {
    fn is_valid(&self) -> bool {
        self.total_mb > 0 && self.used_mb <= self.total_mb
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalState {
    pub cpu_temp_c: f32,
    pub gpu_temp_c: Option<f32>,
    pub fan_speed_rpm: Option<u32>,
}

impl PayloadValidation for ThermalState {
    fn is_valid(&self) -> bool {
        self.cpu_temp_c > 0.0 && self.cpu_temp_c < 150.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryState {
    pub level_pct: f32,
    pub charging: bool,
    pub time_remaining_min: Option<u32>,
}

impl PayloadValidation for BatteryState {
    fn is_valid(&self) -> bool {
        (0.0..=100.0).contains(&self.level_pct)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAddressInfo {
    pub internal_ip: Option<String>,
    pub lan_ip: Option<String>,
    pub public_ip: Option<String>,
    pub vpn_ip: Option<String>,
    pub vpn_context: Option<String>,
    pub nic_candidates: Vec<NicCandidate>,
}

impl PayloadValidation for NetworkAddressInfo {
    fn is_valid(&self) -> bool {
        // At least one IP must be present
        self.internal_ip.is_some()
            || self.lan_ip.is_some()
            || self.public_ip.is_some()
            || self.vpn_ip.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicCandidate {
    pub name: String,
    pub ip: String,
    pub is_wired: bool,
}

/// Generic resource sample wrapper (optional, for dynamic dispatch).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResourceSample {
    pub resource: NodeResource,
    pub data: serde_json::Value,
}

impl PayloadValidation for NodeResourceSample {
    fn is_valid(&self) -> bool {
        !self.data.is_null()
    }
}
