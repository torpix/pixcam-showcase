use pixcam_showcase::bridge::{Client, Config, READ_ONLY_TOOLS, resolve_cli};
use serde_json::json;
use std::{
    ffi::OsStr,
    fs,
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
fn mock_executable() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/mock_mcp.py")
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
                executable: mock_executable(),
                connect: Some(address.into()),
                ..Config::default()
            }
            .validate()
            .is_err()
        );
    }
    assert!(
        Config {
            executable: mock_executable(),
            connect: Some("127.0.0.1:24567".into()),
            ..Config::default()
        }
        .validate()
        .is_ok()
    );
    assert!(
        Config {
            executable: mock_executable(),
            connect: Some("127.0.0.1:24567".into()),
            input: Some("specimen".into()),
            ..Config::default()
        }
        .validate()
        .is_err()
    );
}

#[test]
fn cli_resolution_has_explicit_environment_path_precedence_and_supports_spaces() {
    let explicit = PathBuf::from("/bin/true");
    let environment = OsStr::new("/bin/false");
    assert_eq!(
        resolve_cli(Some(&explicit), Some(environment), Some(OsStr::new("/bin"))).unwrap(),
        fs::canonicalize(&explicit).unwrap()
    );
    assert_eq!(
        resolve_cli(None, Some(environment), Some(OsStr::new("/bin"))).unwrap(),
        fs::canonicalize("/bin/false").unwrap()
    );

    let directory = std::env::temp_dir().join(format!(
        "pixcam solidcraft path test {}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    let candidate = directory.join("solidcraft-cli");
    fs::copy(std::env::current_exe().unwrap(), &candidate).unwrap();
    let found = resolve_cli(None, None, Some(directory.as_os_str())).unwrap();
    assert_eq!(found, fs::canonicalize(&candidate).unwrap());
    fs::remove_file(candidate).unwrap();
    fs::remove_dir(directory).unwrap();
}

#[test]
fn cli_resolution_and_configuration_fail_clearly() {
    let missing = PathBuf::from("/definitely/missing/solidcraft-cli");
    let error = resolve_cli(
        Some(&missing),
        Some(OsStr::new("/bin/true")),
        Some(OsStr::new("/bin")),
    )
    .unwrap_err();
    assert!(error.contains("--solidcraft-cli"));
    assert!(error.contains("unavailable"));
    assert!(
        resolve_cli(None, None, None)
            .unwrap_err()
            .contains("not found")
    );

    let args = vec![
        "--solidcraft-cli".into(),
        "/bin/true".into(),
        "--in".into(),
        format!(
            "{}/fixtures/bridge-part.solidcraft",
            env!("CARGO_MANIFEST_DIR")
        ),
    ];
    let config = pixcam_showcase::bridge::config(&args).unwrap();
    assert_eq!(config.executable, fs::canonicalize("/bin/true").unwrap());
    assert!(config.input.unwrap().is_absolute());
}
#[test]
#[ignore = "requires installed SolidCraft CLI 0.17.0; run explicitly for integration acceptance"]
fn actual_headless_mcp_roundtrip_and_restart() {
    let args = vec![
        "--in".into(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures/bridge-part.solidcraft")
            .display()
            .to_string(),
    ];
    let config = pixcam_showcase::bridge::config(&args).unwrap();
    for _ in 0..2 {
        let snapshot = Client::capture(&config).unwrap();
        let report = pixcam_showcase::recognition::analyze(&snapshot.to_string()).unwrap();
        assert_eq!(report.features.len(), 5);
        assert_eq!(report.document, "bridge-part");
        assert!(snapshot["inspect_design"].get("path").is_none());
        assert!(snapshot["inspect_design"].get("metadata").is_none());
    }
}
