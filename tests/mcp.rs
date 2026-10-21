use pixcam_showcase::mcp;
use serde_json::json;
#[test]
fn inspection_filters_synthetic_only_and_rejects_paths() {
    let r = mcp::call("inspect_features", json!({"face":"Y_MIN","diameter_mm":6})).unwrap();
    assert_eq!(r["count"], 2);
    assert_eq!(r["synthetic_only"], true);
    assert!(mcp::call("inspect_features", json!({"path":"secrets.json"})).is_err());
    assert!(mcp::call("inspect_features", json!({"diameter_mm":-1})).is_err());
}
#[test]
fn explanation_reports_unsupported_cycles_without_inventing_moves() {
    let r = mcp::call("explain_nc_cycle", json!({"nc":"G81 X10"})).unwrap();
    assert_eq!(r["supported"], false);
    assert_eq!(r["motion_count"], 0);
    assert_eq!(r["machining_verified"], false);
    let r = mcp::call("explain_nc_cycle", json!({"nc":"G0 X2\nG1 Y3 F100"})).unwrap();
    assert_eq!(r["supported"], true);
    assert_eq!(r["motion_count"], 2);
    assert!(mcp::call("explain_nc_cycle", json!({"nc":"x".repeat(4097)})).is_err());
}
#[test]
fn transform_tool_validates_handedness_and_reports_point() {
    let args = json!({"transform":{"rotation":[[0,-1,0],[1,0,0],[0,0,1]],"translation_mm":[11,23,7]},"point_mm":[2,3,4]});
    let r = mcp::call("validate_coordinate_transform", args).unwrap();
    assert_eq!(r["valid"], true);
    assert_eq!(r["transformed_point_mm"], json!([8., 25., 11.]));
    let r=mcp::call("validate_coordinate_transform",json!({"transform":{"rotation":[[-1,0,0],[0,1,0],[0,0,1]],"translation_mm":[0,0,0]},"point_mm":[1,2,3]})).unwrap();
    assert_eq!(r["valid"], false);
    assert!(r["transformed_point_mm"].is_null());
}
#[test]
fn lifecycle_tools_and_error_classes() {
    let mut s = mcp::Server::default();
    let req =
        |method: &str, params| json!({"jsonrpc":"2.0","id":1,"method":method,"params":params});
    assert_eq!(
        s.respond(req("tools/list", json!({}))).unwrap()["error"]["code"],
        -32000
    );
    let init=s.respond(req("initialize",json!({"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}))).unwrap();
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert!(
        s.respond(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .is_none()
    );
    let list = s.respond(req("tools/list", json!({}))).unwrap();
    let tools = list["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 3);
    for t in tools {
        assert_eq!(t["annotations"]["readOnlyHint"], true);
        assert!(t["outputSchema"].is_object());
    }
    let result = s
        .respond(req(
            "tools/call",
            json!({"name":"inspect_features","arguments":{}}),
        ))
        .unwrap();
    assert_eq!(result["result"]["structuredContent"]["count"], 9);
    let result = s
        .respond(req(
            "tools/call",
            json!({"name":"inspect_features","arguments":{"unexpected":1}}),
        ))
        .unwrap();
    assert_eq!(result["result"]["isError"], true);
    assert_eq!(
        s.respond(req("bad", json!({}))).unwrap()["error"]["code"],
        -32601
    );
}
#[test]
fn real_stdio_transport_has_no_extra_output() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_pixcam-showcase"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = child.stdin.take().unwrap();
    for request in [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"test","version":"1"}}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"inspect_features","arguments":{}}}),
    ] {
        writeln!(input, "{request}").unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let responses = text
        .lines()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[1]["result"]["structuredContent"]["count"], 9);
}
