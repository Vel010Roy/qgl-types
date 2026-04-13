//! Display stream types — frames from screen capture, window capture, scrcpy.
//!
//! Published on `nexox/up/display/<node>/<display_id>`.
//! Consumed by minox (VLM input) and argox (live view).

use serde::{Deserialize, Serialize};

use crate::validate::PayloadValidation;

/// Video/image codec for display frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    Jpeg,
    H264,
    Vp9,
    Raw,
}

/// A single captured display frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayFrame {
    pub display_id: String,
    pub width: u32,
    pub height: u32,
    pub codec: Codec,
    pub timestamp_us: u64,
    pub sequence: u64,
    /// Encoded frame data (JPEG bytes, H264 NAL, etc.).
    /// Base64 encoded when serialized to JSON.
    #[serde(with = "base64_bytes")]
    pub data: Vec<u8>,
}

impl PayloadValidation for DisplayFrame {
    fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0 && !self.data.is_empty() && !self.display_id.is_empty()
    }
}

/// Static metadata about a display source (published on discovery).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub display_id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub source_type: DisplaySourceType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplaySourceType {
    NativeScreen,
    NativeWindow,
    ScrcpyTablet,
    VirtualDisplay,
}

impl PayloadValidation for DisplayInfo {
    fn is_valid(&self) -> bool {
        !self.display_id.is_empty() && self.width > 0 && self.height > 0
    }
}

/// Base64 serde helper for Vec<u8>.
mod base64_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
        encoded.serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        use base64::Engine;
        let s = String::deserialize(deserializer)?;
        base64::engine::general_purpose::STANDARD
            .decode(&s)
            .map_err(serde::de::Error::custom)
    }
}
