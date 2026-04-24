//! Recorder type roundtrip + validation tests.
//!
//! These are the RED tests for the leaf-recorder feature — they pin the
//! wire format of every RecordCommand / CapturedInputEvent / SessionManifest
//! so that downstream replay tools can rely on stable JSON.

use chrono::Utc;
use qgl_types::recorder::{
    CapturedInputEvent, InputAction, InputData, InputDevice, LockState, MouseButton,
    RecordCommand, SessionDisplay, SessionManifest, SessionStatus,
};
use qgl_types::validate::PayloadValidation;
use uuid::Uuid;

fn roundtrip<T>(value: &T) -> T
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    let json = serde_json::to_string(value).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

#[test]
fn record_command_start_without_dir_roundtrips() {
    let cmd = RecordCommand::Start { record_dir: None };
    let back: RecordCommand = roundtrip(&cmd);
    assert_eq!(back, cmd);
    assert!(cmd.is_valid());
}

#[test]
fn record_command_start_with_dir_roundtrips() {
    let cmd = RecordCommand::Start {
        record_dir: Some(r"E:\nexox-recordings".into()),
    };
    let back: RecordCommand = roundtrip(&cmd);
    assert_eq!(back, cmd);
    assert!(cmd.is_valid());
}

#[test]
fn record_command_rejects_empty_dir() {
    let cmd = RecordCommand::Start {
        record_dir: Some(String::new()),
    };
    assert!(!cmd.is_valid(), "empty record_dir must fail validation");
}

#[test]
fn record_command_stop_and_status_roundtrip() {
    for cmd in [RecordCommand::Stop, RecordCommand::Status] {
        let back: RecordCommand = roundtrip(&cmd);
        assert_eq!(back, cmd);
    }
}

#[test]
fn captured_key_main_row_and_numpad_are_distinct() {
    // Main number row and NumPad both carry '5' visually but must be
    // separable at replay time. rdev::Key debug format uses Num5 vs Kp5.
    let main_5 = CapturedInputEvent {
        seq_no: 0,
        offset_us: 1_234,
        device: InputDevice::Keyboard,
        action: InputAction::KeyDown,
        data: InputData::Key {
            name: "Num5".into(),
            scan_code: Some(0x06),
            locks: LockState::default(),
        },
    };
    let kp_5 = CapturedInputEvent {
        seq_no: 1,
        offset_us: 2_345,
        device: InputDevice::Keyboard,
        action: InputAction::KeyDown,
        data: InputData::Key {
            name: "Kp5".into(),
            // NumPad 5: scan code 0x4C; extended flag NOT set → bit 24 clear.
            scan_code: Some(0x4C),
            locks: LockState::default(),
        },
    };

    // Ensure JSON distinguishes them.
    let j_main = serde_json::to_string(&main_5).unwrap();
    let j_kp = serde_json::to_string(&kp_5).unwrap();
    assert!(j_main.contains("Num5"));
    assert!(j_kp.contains("Kp5"));
    assert_ne!(j_main, j_kp);

    // Roundtrip both.
    assert_eq!(roundtrip(&main_5), main_5);
    assert_eq!(roundtrip(&kp_5), kp_5);
}

#[test]
fn captured_key_system_cluster_preserved() {
    // Sanity check the keys the operator explicitly called out: Esc, F1..F12,
    // PrintScreen, ScrollLock, Pause, Insert, Home, PageUp/Down, Delete, End.
    let names = [
        "Escape", "F1", "F12", "PrintScreen", "ScrollLock", "Pause",
        "Insert", "Home", "PageUp", "PageDown", "Delete", "End",
    ];
    for name in names {
        let ev = CapturedInputEvent {
            seq_no: 0,
            offset_us: 0,
            device: InputDevice::Keyboard,
            action: InputAction::KeyDown,
            data: InputData::Key {
                name: name.into(),
                scan_code: None,
                locks: LockState::default(),
            },
        };
        assert!(ev.is_valid(), "{name} must validate");
        let back: CapturedInputEvent = roundtrip(&ev);
        assert_eq!(back, ev, "{name} must roundtrip");
    }
}

#[test]
fn captured_key_lockstate_flows() {
    let ev = CapturedInputEvent {
        seq_no: 9,
        offset_us: 10_000,
        device: InputDevice::Keyboard,
        action: InputAction::KeyUp,
        data: InputData::Key {
            name: "A".into(),
            scan_code: Some(0x1E),
            locks: LockState { caps: true, num: false, scroll: true },
        },
    };
    let back: CapturedInputEvent = roundtrip(&ev);
    assert_eq!(back, ev);
}

#[test]
fn captured_mouse_move_and_scroll_roundtrip() {
    let mv = CapturedInputEvent {
        seq_no: 0,
        offset_us: 100,
        device: InputDevice::Mouse,
        action: InputAction::MouseMove,
        data: InputData::Mouse {
            x: 512.0,
            y: 384.5,
            button: None,
            scroll_dx: None,
            scroll_dy: None,
        },
    };
    let scroll = CapturedInputEvent {
        seq_no: 1,
        offset_us: 200,
        device: InputDevice::Mouse,
        action: InputAction::MouseScroll,
        data: InputData::Mouse {
            x: 0.0,
            y: 0.0,
            button: None,
            scroll_dx: Some(0.0),
            scroll_dy: Some(-1.0),
        },
    };
    let down = CapturedInputEvent {
        seq_no: 2,
        offset_us: 300,
        device: InputDevice::Mouse,
        action: InputAction::MouseDown,
        data: InputData::Mouse {
            x: 100.0,
            y: 100.0,
            button: Some(MouseButton::Middle),
            scroll_dx: None,
            scroll_dy: None,
        },
    };

    for ev in [mv, scroll, down] {
        assert!(ev.is_valid());
        assert_eq!(roundtrip(&ev), ev);
    }
}

#[test]
fn captured_mouse_rejects_nan_coords() {
    let bad = CapturedInputEvent {
        seq_no: 0,
        offset_us: 0,
        device: InputDevice::Mouse,
        action: InputAction::MouseMove,
        data: InputData::Mouse {
            x: f64::NAN,
            y: 0.0,
            button: None,
            scroll_dx: None,
            scroll_dy: None,
        },
    };
    assert!(!bad.is_valid());
}

#[test]
fn session_manifest_roundtrips_with_displays() {
    let manifest = SessionManifest {
        session_id: Uuid::new_v4(),
        node_id: "win-a2".into(),
        start_utc: Utc::now(),
        stop_utc: None,
        status: SessionStatus::Recording,
        record_dir: r"E:\nexox-recordings\abc123".into(),
        displays: vec![
            SessionDisplay {
                display_index: 0,
                width: 1920,
                height: 1080,
                fps: 4,
                codec: "h264_nvenc".into(),
            },
        ],
    };
    assert!(manifest.is_valid());
    assert_eq!(roundtrip(&manifest), manifest);
}

#[test]
fn session_manifest_rejects_empty_node_id() {
    let m = SessionManifest {
        session_id: Uuid::new_v4(),
        node_id: String::new(),
        start_utc: Utc::now(),
        stop_utc: None,
        status: SessionStatus::Recording,
        record_dir: "/tmp".into(),
        displays: vec![],
    };
    assert!(!m.is_valid());
}
