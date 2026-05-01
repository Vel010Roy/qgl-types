//! Leaf-local recording session types.
//!
//! A recording is stored on the leaf node's local disk (e.g.
//! `E:\nexox-recordings\<session_uuid>\`). Only control commands and
//! session metadata flow over Zenoh; frame/input streams stay local.
//!
//! Topics:
//!   `nexox/down/record/<node>/cmd`    — RecordCommand (operator → leaf)
//!   `nexox/up/record/<node>/status`   — SessionManifest snapshots
//!
//! The capture pipeline distinguishes **every** physical key on a 108-key
//! layout — main number row (`Num1`..`Num0`) vs numpad (`Kp1`..`Kp0`) are
//! separate variants, function keys / navigation cluster / system keys
//! (PrintScreen, ScrollLock, Pause) all have explicit names. A raw
//! platform scan code is stored alongside so future decoders can still
//! resolve anything the name set misses.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(feature = "ts")]
use ts_rs::TS;

use crate::validate::PayloadValidation;

// TS export 경로: leaf-tauri/nexox-leaf/src/types/ — frontend 가 import.
// `--features ts` 일 때만 적용.

// ---------------------------------------------------------------------
// Control plane: operator → leaf
// ---------------------------------------------------------------------

/// Command from an operator (CLI subcommand or future dashboard button)
/// to the leaf's recording subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub enum RecordCommand {
    /// Begin a new recording session.
    ///
    /// `record_dir` overrides the leaf's default storage path when
    /// provided (e.g. `E:\nexox-recordings` vs `F:\nexox-recordings`).
    /// `target` selects what gets captured — primary display if None.
    /// The field is `#[serde(default)]` so pre-target JSON (just
    /// `{"record_dir": ...}`) still parses.
    Start {
        record_dir: Option<String>,
        #[serde(default)]
        target: Option<RecordTarget>,
    },
    /// Flush open files and close the current session.
    Stop,
    /// Return current session state without side effects.
    Status,
}

/// What the recorder should capture for one session. Backward-compat
/// note: added after the initial RecordCommand shape, but thanks to
/// `Option<RecordTarget>` + `#[serde(default)]` on the Start variant,
/// older signal files that omit this key still deserialize — they
/// simply default to the primary display (the pre-target behavior).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub enum RecordTarget {
    /// One physical monitor, 0-based index matching
    /// `screenshots::Screen::all()` order (same as live screen-pub).
    Display { index: u32 },
    /// One specific window. `title_contains` is a case-insensitive
    /// substring of the window title; the leaf picks the first
    /// visible top-level window whose title matches. Empty string
    /// is invalid (would match everything).
    Window { title_contains: String },
}

impl PayloadValidation for RecordTarget {
    fn is_valid(&self) -> bool {
        match self {
            RecordTarget::Display { .. } => true,
            RecordTarget::Window { title_contains } => !title_contains.trim().is_empty(),
        }
    }
}

impl PayloadValidation for RecordCommand {
    fn is_valid(&self) -> bool {
        match self {
            RecordCommand::Start { record_dir, target } => {
                if let Some(p) = record_dir {
                    if p.is_empty() {
                        return false;
                    }
                }
                match target {
                    Some(t) => t.is_valid(),
                    None => true,
                }
            }
            _ => true,
        }
    }
}

// ---------------------------------------------------------------------
// Input event — persisted to <session>/input.jsonl
// ---------------------------------------------------------------------

/// One captured input action. Stored one-per-line as JSON in
/// `input.jsonl` for crash-tolerant append semantics.
///
/// `offset_us` is microseconds since the session's monotonic start and
/// is authoritative for replay sync (matches the frame timestamp file
/// `display-<n>.ts.jsonl`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub struct CapturedInputEvent {
    pub seq_no: u64,
    pub offset_us: u64,
    pub device: InputDevice,
    pub action: InputAction,
    pub data: InputData,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub enum InputDevice {
    Keyboard,
    Mouse,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub enum InputAction {
    KeyDown,
    KeyUp,
    MouseMove,
    MouseDown,
    MouseUp,
    MouseScroll,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub enum InputData {
    Key {
        /// Symbolic key name — `rdev::Key` Debug form preserves the
        /// main-row vs numpad distinction (e.g. `"Num5"` vs `"Kp5"`,
        /// `"KpPlus"` vs `"Equal"+Shift`).
        name: String,
        /// Platform scan code (Windows: `KBDLLHOOKSTRUCT.scanCode`)
        /// with the EXTENDED flag merged into bit 24 for unambiguous
        /// encoding. `None` on platforms where we can't retrieve it.
        scan_code: Option<u32>,
        /// Lock-key snapshot at event time.
        locks: LockState,
    },
    Mouse {
        x: f64,
        y: f64,
        /// Which button (for Down/Up/Click), `None` for Move/Scroll.
        button: Option<MouseButton>,
        scroll_dx: Option<f64>,
        scroll_dy: Option<f64>,
    },
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub struct LockState {
    pub caps: bool,
    pub num: bool,
    pub scroll: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    X1,
    X2,
}

impl PayloadValidation for CapturedInputEvent {
    fn is_valid(&self) -> bool {
        match &self.data {
            InputData::Key { name, .. } => !name.is_empty(),
            InputData::Mouse { x, y, .. } => x.is_finite() && y.is_finite(),
        }
    }
}

// ---------------------------------------------------------------------
// Session manifest — written once at start, mutated at stop/crash-recover
// ---------------------------------------------------------------------

/// Written to `<session>/manifest.json`. Serves both as session
/// metadata and as a crash marker — on restart, any manifest still in
/// `Recording` state is either resumed or promoted to `Crashed`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub struct SessionManifest {
    pub session_id: Uuid,
    pub node_id: String,
    pub start_utc: DateTime<Utc>,
    pub stop_utc: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub record_dir: String,
    pub displays: Vec<SessionDisplay>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub enum SessionStatus {
    Recording,
    Completed,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(
    feature = "ts",
    derive(TS),
    ts(export, export_to = "bindings/")
)]
pub struct SessionDisplay {
    pub display_index: u32,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub codec: String,
}

impl PayloadValidation for SessionManifest {
    fn is_valid(&self) -> bool {
        !self.node_id.is_empty() && !self.record_dir.is_empty()
    }
}
