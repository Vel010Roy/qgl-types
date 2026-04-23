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
    /// Per-display capture + encoder state (see `StreamState`).
    /// Published on `nexox/up/state/<node>/stream` once Task #34
    /// lands; argox Status tab surfaces it as `Capture` + `EncFmt`
    /// columns.
    Stream,
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

/// Per-display capture + encoder observability sample.
///
/// Published on `nexox/up/state/<node>/stream` so subscribers (argox,
/// operator CLIs) can see *which backend* a leaf is using (DXGI vs
/// WGC vs ScreenCaptureKit vs screenshots crate) and *what pixel
/// format* the encoder is consuming, without tailing remote logs.
///
/// Phase 1 of Task #27 motivated this: DXGI vs WGC selection was only
/// visible in SSH log dumps. Exposing it as structured state lets the
/// dashboard surface a `Capture` column and lets future automation
/// (minox scheduler, argox alerts) make decisions on capture backend
/// directly.
///
/// Multi-display note: first implementation publishes just the
/// primary display (`display_index = 0`). When screen_pub gains
/// per-display publish fan-out, state-pub emits one envelope per
/// active display with the matching `display_index`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamState {
    /// 0-based display index, matching `screenshots::Screen::all()`
    /// ordering so frame / state topics line up per display.
    pub display_index: u32,
    /// Short backend identifier: `"dxgi"`, `"wgc"`, `"sck"`,
    /// `"screenshots"`. Kept as a string (not an enum) so adding
    /// new backends in nexox doesn't force a qgl-types bump before
    /// publish works; subscribers that render to UI treat unknown
    /// values as fallthrough labels.
    pub capture_backend: String,
    /// FFmpeg encoder ID: `"h264_nvenc"`, `"h264_videotoolbox"`,
    /// `"h264_amf"`, `"h264_qsv"`, `"libx264"`, `"mjpeg"`, or
    /// `"none"` if the node is raw-publishing (tablet / preview).
    pub encoder_name: String,
    /// FFmpeg pixel format the encoder consumes: `"nv12"`,
    /// `"yuv420p"`, `"bgra"`, `"rgba"`. This is the encoder's *input*
    /// format (after any swscale pass), which differs from
    /// `DisplayFrame::codec` on the wire and from `DecFmt` (the
    /// subscriber-side decoder output).
    pub encoder_pix_fmt: String,
    /// Capture width in pixels (native, before any downscale).
    pub width: u32,
    /// Capture height in pixels.
    pub height: u32,
    /// Actual publish rate the leaf is hitting right now, in frames
    /// per second. Differs from the configured target fps when the
    /// encoder or capture is throttled.
    pub fps: f32,
}

impl PayloadValidation for StreamState {
    fn is_valid(&self) -> bool {
        self.width > 0
            && self.height > 0
            && self.fps >= 0.0
            && !self.capture_backend.is_empty()
            && !self.encoder_name.is_empty()
            && !self.encoder_pix_fmt.is_empty()
    }
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
