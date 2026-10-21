use pixcam_showcase::local_server::authorized;
fn headers() -> Vec<(String, String)> {
    vec![
        ("host".into(), "127.0.0.1:45678".into()),
        ("origin".into(), "http://127.0.0.1:45678".into()),
        ("x-pixcam-session".into(), "test-session".into()),
        ("content-type".into(), "application/json".into()),
        ("sec-fetch-site".into(), "same-origin".into()),
    ]
}
#[test]
fn http_access_requires_same_origin_host_session_and_explicit_route() {
    let h = headers();
    let check =
        |method, path, h: &[_]| authorized(method, path, h, "127.0.0.1:45678", "test-session");
    assert!(check("POST", "/api/refresh", &h));
    assert!(check("POST", "/api/analyze", &h));
    assert!(check("GET", "/", &h));
    for name in [
        "host",
        "origin",
        "x-pixcam-session",
        "content-type",
        "sec-fetch-site",
    ] {
        let mut bad = h.clone();
        bad.iter_mut().find(|(k, _)| k == name).unwrap().1 = "evil.example".into();
        assert!(!check("POST", "/api/refresh", &bad), "{name}");
    }
    assert!(!check("GET", "/api/refresh", &h));
    assert!(!check("POST", "/api/execute", &h));
    assert!(!check("GET", "/../../private", &h));
    assert!(!check("OPTIONS", "/api/refresh", &h));
    assert!(!check("POST", "/api/refresh", &[]));
}
