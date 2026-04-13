//! Serde roundtrip tests — every payload type must survive JSON serialize → deserialize.

use qgl_types::commands::*;
use qgl_types::display::*;
use qgl_types::envelope::Envelope;
use qgl_types::events::*;
use qgl_types::input::*;
use qgl_types::logs::*;
use qgl_types::metrics::*;
use qgl_types::state::*;
use qgl_types::validate::PayloadValidation;

use chrono::Utc;
use uuid::Uuid;

/// Helper: serialize to JSON, deserialize back, assert equality via Debug repr.
fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug>(val: &T) {
    let json = serde_json::to_string(val).expect("serialize");
    let back: T = serde_json::from_str(&json).expect("deserialize");
    // Compare Debug output (avoids PartialEq requirement on all types)
    assert_eq!(format!("{:?}", val), format!("{:?}", back));
}

// --- Envelope ---

#[test]
fn envelope_roundtrip() {
    let env = Envelope::new("test-service", GpuState {
        utilization_pct: 75.0,
        memory_used_mb: 4096,
        memory_total_mb: 8192,
        temperature_c: Some(72.0),
    });
    roundtrip(&env);
}

#[test]
fn envelope_child_preserves_correlation() {
    let parent = Envelope::new("parent", CpuState {
        utilization_pct: 50.0,
        core_count: 8,
        frequency_mhz: Some(3200),
    });
    let child = Envelope::child(&parent, "child", RamState {
        used_mb: 16000,
        total_mb: 32000,
        swap_used_mb: 0,
    });
    assert_eq!(child.correlation_id, parent.correlation_id);
    assert_eq!(child.parent_span_id, Some(parent.correlation_id));
}

// --- State types ---

#[test]
fn gpu_state_roundtrip() {
    let s = GpuState { utilization_pct: 95.5, memory_used_mb: 6000, memory_total_mb: 8192, temperature_c: Some(80.0) };
    roundtrip(&s);
    assert!(s.is_valid());
}

#[test]
fn gpu_state_invalid_utilization() {
    let s = GpuState { utilization_pct: 150.0, memory_used_mb: 0, memory_total_mb: 8192, temperature_c: None };
    assert!(!s.is_valid());
}

#[test]
fn cpu_state_roundtrip() {
    let s = CpuState { utilization_pct: 45.0, core_count: 10, frequency_mhz: Some(3600) };
    roundtrip(&s);
    assert!(s.is_valid());
}

#[test]
fn ram_state_roundtrip() {
    let s = RamState { used_mb: 16000, total_mb: 32000, swap_used_mb: 1024 };
    roundtrip(&s);
    assert!(s.is_valid());
}

#[test]
fn ram_state_invalid_overflow() {
    let s = RamState { used_mb: 64000, total_mb: 32000, swap_used_mb: 0 };
    assert!(!s.is_valid());
}

#[test]
fn thermal_state_roundtrip() {
    let s = ThermalState { cpu_temp_c: 65.0, gpu_temp_c: Some(72.0), fan_speed_rpm: Some(2400) };
    roundtrip(&s);
    assert!(s.is_valid());
}

#[test]
fn battery_state_roundtrip() {
    let s = BatteryState { level_pct: 87.5, charging: true, time_remaining_min: Some(120) };
    roundtrip(&s);
    assert!(s.is_valid());
}

#[test]
fn network_address_info_roundtrip() {
    let s = NetworkAddressInfo {
        internal_ip: Some("192.168.1.5".into()),
        lan_ip: Some("10.0.0.5".into()),
        public_ip: None,
        vpn_ip: None,
        vpn_context: None,
        nic_candidates: vec![
            NicCandidate { name: "en0".into(), ip: "192.168.1.5".into(), is_wired: true },
        ],
    };
    roundtrip(&s);
    assert!(s.is_valid());
}

#[test]
fn network_address_all_none_invalid() {
    let s = NetworkAddressInfo {
        internal_ip: None, lan_ip: None, public_ip: None, vpn_ip: None,
        vpn_context: None, nic_candidates: vec![],
    };
    assert!(!s.is_valid());
}

#[test]
fn node_resource_sample_roundtrip() {
    let s = NodeResourceSample {
        resource: NodeResource::Gpu,
        data: serde_json::json!({"utilization_pct": 50.0}),
    };
    roundtrip(&s);
    assert!(s.is_valid());
}

// --- Display types ---

#[test]
fn display_frame_roundtrip() {
    let f = DisplayFrame {
        display_id: "hdmi-0".into(),
        width: 1920,
        height: 1080,
        codec: Codec::Jpeg,
        timestamp_us: 1234567890,
        sequence: 42,
        data: vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG magic bytes
    };
    roundtrip(&f);
    assert!(f.is_valid());
}

#[test]
fn display_frame_empty_data_invalid() {
    let f = DisplayFrame {
        display_id: "hdmi-0".into(),
        width: 1920, height: 1080, codec: Codec::Raw,
        timestamp_us: 0, sequence: 0, data: vec![],
    };
    assert!(!f.is_valid());
}

#[test]
fn display_info_roundtrip() {
    let i = DisplayInfo {
        display_id: "scrcpy-0".into(),
        name: "Galaxy Tab S9".into(),
        width: 2560, height: 1600,
        source_type: DisplaySourceType::ScrcpyTablet,
    };
    roundtrip(&i);
    assert!(i.is_valid());
}

// --- Input types ---

#[test]
fn mouse_event_roundtrip() {
    let e = InputEvent::Mouse(MouseEvent {
        action: MouseAction::Click,
        x: 100.0, y: 200.0,
        button: Some(MouseButton::Left),
    });
    roundtrip(&e);
    assert!(e.is_valid());
}

#[test]
fn key_event_roundtrip() {
    let e = InputEvent::Key(KeyEvent {
        action: KeyAction::Press,
        key_code: 54, // RMeta (한/영)
        modifiers: vec![],
    });
    roundtrip(&e);
    assert!(e.is_valid());
}

#[test]
fn touch_event_roundtrip() {
    let e = InputEvent::Touch(TouchEvent {
        action: TouchAction::Swipe,
        x: 100.0, y: 200.0,
        x2: Some(300.0), y2: Some(400.0),
        duration_ms: Some(500),
    });
    roundtrip(&e);
    assert!(e.is_valid());
}

#[test]
fn hw_serial_event_roundtrip() {
    let e = InputEvent::HwSerial(HwSerialEvent {
        port: "/dev/ttyACM0".into(),
        data: vec![0x01, 0x02, 0x03],
    });
    roundtrip(&e);
    assert!(e.is_valid());
}

// --- Commands types ---

#[test]
fn schedule_decision_roundtrip() {
    let c = ScheduleDecision {
        task_id: Uuid::new_v4(),
        target_node: "mac-mini".into(),
        target_tier: 2,
        reason: "GPU load < 70%".into(),
    };
    roundtrip(&c);
    assert!(c.is_valid());
}

#[test]
fn schedule_decision_invalid_tier() {
    let c = ScheduleDecision {
        task_id: Uuid::new_v4(),
        target_node: "mac-mini".into(),
        target_tier: 5, // invalid
        reason: "test".into(),
    };
    assert!(!c.is_valid());
}

#[test]
fn tier_route_roundtrip() {
    let r = TierRoute {
        node: "win-a1".into(),
        from_tier: 1, to_tier: 3,
        reason: "overload predicted".into(),
    };
    roundtrip(&r);
    assert!(r.is_valid());
}

#[test]
fn throttle_command_roundtrip() {
    let c = ThrottleCommand {
        node: "relay-1".into(),
        action: ThrottleAction::ReduceFps { target_fps: 15 },
    };
    roundtrip(&c);
    assert!(c.is_valid());
}

#[test]
fn task_descriptor_roundtrip() {
    let t = TaskDescriptor {
        task_id: Uuid::new_v4(),
        task_type: "vlm-inference".into(),
        priority: TaskPriority::High,
        payload: serde_json::json!({"model": "qwen-7b", "prompt": "describe this"}),
    };
    roundtrip(&t);
    assert!(t.is_valid());
}

#[test]
fn operator_command_roundtrip() {
    let c = OperatorCommand {
        service: "screen-pub".into(),
        command: "restart".into(),
        args: serde_json::json!({}),
    };
    roundtrip(&c);
    assert!(c.is_valid());
}

// --- Events types ---

#[test]
fn tablet_ai_result_roundtrip() {
    let r = TabletAiResult {
        tablet_id: "galaxy-001".into(),
        model_id: "yolo-nano".into(),
        inference_ms: 45,
        result: serde_json::json!({"detected": ["button", "text_field"]}),
        success: true,
    };
    roundtrip(&r);
    assert!(r.is_valid());
}

#[test]
fn decision_event_roundtrip() {
    let d = DecisionEvent {
        task_id: Uuid::new_v4(),
        decision: "route to tier 2".into(),
        target_node: "mac-mini".into(),
        target_tier: 2,
        latency_ms: 15,
    };
    roundtrip(&d);
    assert!(d.is_valid());
}

#[test]
fn overload_forecast_roundtrip() {
    let o = OverloadForecast {
        node: "controller".into(),
        predicted_at_secs: 120,
        confidence: 0.85,
        recommended_action: "throttle FPS to 15".into(),
    };
    roundtrip(&o);
    assert!(o.is_valid());
}

#[test]
fn health_status_roundtrip() {
    let h = HealthStatus {
        service: "screen-pub".into(),
        status: HealthState::Healthy,
        uptime_secs: 3600,
        restart_count: 0,
        last_error: None,
    };
    roundtrip(&h);
    assert!(h.is_valid());
}

#[test]
fn device_connected_event_roundtrip() {
    let d = DeviceConnectedEvent {
        serial: "ABC123".into(),
        model: "Galaxy Tab S9".into(),
        os: "Android 14".into(),
        manufacturer: "Samsung".into(),
        capabilities: vec!["display".into(), "touch".into()],
    };
    roundtrip(&d);
    assert!(d.is_valid());
}

// --- Logs types ---

#[test]
fn log_entry_roundtrip() {
    let l = LogEntry {
        level: LogLevel::Warn,
        service: "tablet-pub".into(),
        message: "ADB reconnect attempt".into(),
        timestamp: Utc::now(),
        fields: serde_json::json!({"attempt": 3, "tablet": "galaxy-001"}),
    };
    roundtrip(&l);
    assert!(l.is_valid());
}

// --- Metrics types ---

#[test]
fn metric_sample_roundtrip() {
    let m = MetricSample {
        name: "frame_latency_ms".into(),
        value: 45.2,
        unit: Some("ms".into()),
        timestamp: Utc::now(),
        tags: vec![("node".into(), "mac-mini".into())],
    };
    roundtrip(&m);
    assert!(m.is_valid());
}

#[test]
fn metric_batch_roundtrip() {
    let b = MetricBatch {
        service: "screen-pub".into(),
        samples: vec![
            MetricSample {
                name: "fps".into(),
                value: 29.8,
                unit: Some("fps".into()),
                timestamp: Utc::now(),
                tags: vec![],
            },
        ],
    };
    roundtrip(&b);
    assert!(b.is_valid());
}
