//! Input event types — mouse, keyboard, touch, HW serial (Arduino HID).
//!
//! Published on `nexox/down/input/<node>/...` (downstream: controller → leaf).
//! Consumed by nexox input injection (enigo/ADB).

use serde::{Deserialize, Serialize};

use crate::validate::PayloadValidation;

/// Top-level input event dispatched to a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputEvent {
    Mouse(MouseEvent),
    Key(KeyEvent),
    Touch(TouchEvent),
    HwSerial(HwSerialEvent),
}

impl PayloadValidation for InputEvent {
    fn is_valid(&self) -> bool {
        match self {
            InputEvent::Mouse(e) => e.is_valid(),
            InputEvent::Key(e) => e.is_valid(),
            InputEvent::Touch(e) => e.is_valid(),
            InputEvent::HwSerial(e) => e.is_valid(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub action: MouseAction,
    pub x: f64,
    pub y: f64,
    pub button: Option<MouseButton>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseAction {
    Move,
    Click,
    DoubleClick,
    Down,
    Up,
    Scroll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

impl PayloadValidation for MouseEvent {
    fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub action: KeyAction,
    pub key_code: u32,
    pub modifiers: Vec<KeyModifier>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyAction {
    Press,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyModifier {
    Shift,
    Ctrl,
    Alt,
    Meta,
}

impl PayloadValidation for KeyEvent {
    fn is_valid(&self) -> bool {
        true // key_code 0 is valid (some platforms)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TouchEvent {
    pub action: TouchAction,
    pub x: f64,
    pub y: f64,
    pub x2: Option<f64>,
    pub y2: Option<f64>,
    pub duration_ms: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TouchAction {
    Tap,
    Swipe,
    LongPress,
}

impl PayloadValidation for TouchEvent {
    fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HwSerialEvent {
    pub port: String,
    pub data: Vec<u8>,
}

impl PayloadValidation for HwSerialEvent {
    fn is_valid(&self) -> bool {
        !self.port.is_empty() && !self.data.is_empty()
    }
}
