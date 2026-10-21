use pixcam_showcase::bridge::{Client, Config, READ_ONLY_TOOLS};
use serde_json::json;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
fn mock(mode: &str) -> Config {
    Config {
        executable: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/mock_mcp.py"),
        input: Some(mode.into()),
        timeout: Duration::from_millis(300),
        ..Config::default()
    }
}
#[test]
fn allowlist_denies_every_model_mutation_before_transport() {
    let mut c = Client::start(&mock("normal")).unwrap();
    assert_eq!(READ_ONLY_TOOLS.len(), 4);
    for tool in [
        "execute",
        "batch",
        "open",
        "save",
        "new_design",
        "undo",
        "redo",
        "export",
        "screenshot",
        "set_parameter",
        "crash_reports",
    ] {
        assert!(c.call(tool, json!({})).unwrap_err().contains("allowlist"));
    }
    assert_eq!(c.call("measure", json!({})).unwrap(), json!({"ok":true}));
    c.close();
    assert!(c.call("measure", json!({})).is_err());
}
#[test]
fn timeout_disconnect_versions_and_protocol_faults_fail_closed() {
    for fault in [
        "timeout",
        "disconnect",
        "malformed",
        "version",
        "wrong_id",
        "both",
    ] {
        let start = Instant::now();
        assert!(Client::start(&mock(fault)).is_err(), "{fault}");
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    for fault in ["tool_error", "rpc_error"] {
        let mut c = Client::start(&mock(fault)).unwrap();
        let error = c.call("measure", json!({})).unwrap_err();
        assert!(!error.contains("secret"));
    }
}
#[test]
fn rejects_external_endpoints_and_ambiguous_mode() {
    for address in [
        "0.0.0.0:37327",
        "192.168.1.2:2345",
        "example.com:1234",
        "127.0.0.1:0",
        "127.0.0.2:1234",
    ] {
        assert!(
            Config {
                connect: Some(address.into()),
                ..Config::default()
            }
            .validate()
            .is_err()
        );
    }
    assert!(
        Config {
            connect: Some("127.0.0.1:24567".into()),
            ..Config::default()
        }
        .validate()
        .is_ok()
    );
    assert!(
        Config {
            connect: Some("127.0.0.1:24567".into()),
            input: Some("specimen".into()),
            ..Config::default()
        }
        .validate()
        .is_err()
    );
}
#[test]
#[ignore = "requires installed SolidCraft CLI 0.17.0; run explicitly for integration acceptance"]
fn actual_headless_mcp_roundtrip_and_restart() {
    let config = Config {
        input: Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/bridge-part.solidcraft"),
        ),
        ..Config::default()
    };
    for _ in 0..2 {
        let snapshot = Client::capture(&config).unwrap();
        let report = pixcam_showcase::recognition::analyze(&snapshot.to_string()).unwrap();
        assert_eq!(report.features.len(), 5);
        assert_eq!(report.document, "bridge-part");
        assert!(snapshot["inspect_design"].get("path").is_none());
        assert!(snapshot["inspect_design"].get("metadata").is_none());
    }
}
