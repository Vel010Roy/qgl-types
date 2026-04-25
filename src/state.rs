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
    /// Node identity / deployment metadata (see `NodeIdentity`).
    /// Published on `nexox/up/state/<node>/identity` at low
    /// frequency (5 min) — git commit / branch / build time +
    /// hostname + public IP. Dashboard status tab surfaces these
    /// so the operator knows which build is running and whether
    /// public IP changed.
    Identity,
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
    /// LAN/zenoh bind IP — leaf 가 zenoh router 에 bind 한 안정적인 IP
    /// (deploy 시 `--ip 192.168.10.X` 인자 또는 `detect_ethernet_ip`
    /// 결과). NIC 우선순위 흔들림 없이 dashboard Internal IP 컬럼이
    /// 이 값을 우선 표시 → 깜빡임 fix.
    pub bind_ip: Option<String>,
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

/// Node identity / deployment metadata. Published on
/// `nexox/up/state/<node>/identity` at low cadence (every 5 min).
///
/// Why a separate topic from the 6 high-frequency resources:
///   - Git commit / build time are static for a process lifetime —
///     no point spamming them at 1 Hz.
///   - Public IP changes rarely (router DHCP, ISP). 5 min poll keeps
///     the external API call rate trivial.
///   - Subscribers want to render this in a header / status row, not
///     in the time-series resource panels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeIdentity {
    /// Short host name (`hostname` POSIX, `COMPUTERNAME` Windows).
    pub hostname: String,
    /// Short git commit hash of the build (`NEXOX_GIT_COMMIT` env at
    /// build time). `"unknown"` on local dev builds without ssh-deploy
    /// providing the env var.
    pub git_commit: String,
    /// Git branch name at build time (`NEXOX_GIT_BRANCH`).
    pub git_branch: String,
    /// Build timestamp (RFC 3339 UTC, e.g. `2026-04-25T11:23:45Z`).
    pub build_time: String,
    /// LAN IP the node is binding zenoh to (eth/wifi auto-detect or
    /// `--ip` flag override). Differs from `public_ip` when behind NAT.
    pub local_ip: String,
    /// External (public) IP as observed via a low-frequency external
    /// API call (`api.ipify.org` or fallback). `None` when the call
    /// failed (no internet, API down, behind firewall blocking egress).
    pub public_ip: Option<String>,
    /// RFC 3339 UTC of the most recent successful public IP fetch.
    /// Stale value with `None` public_ip = the cached IP from before
    /// the egress went down — useful for the dashboard to show an
    /// "IP last verified at X" warning.
    pub public_ip_observed_at: Option<String>,
}

impl PayloadValidation for NodeIdentity {
    fn is_valid(&self) -> bool {
        !self.hostname.is_empty() && !self.git_commit.is_empty() && !self.build_time.is_empty()
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
