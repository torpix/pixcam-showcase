use pixcam_showcase::solidcraft;
use serde_json::json;

#[test]
fn read_only_snapshot_imports_timeline_and_cylindrical_geometry() {
    let snapshot = json!({
        "schema_version": 1,
        "solidcraft_version": "0.17.0",
        "inspect_design": {
            "timeline": [{"id": 7, "name": "Synthetic bore", "type": "cut", "suppressed": false}],
            "bodies": [{"name": "Demo block"}]
        },
        "body_topology": [{
            "body": "Demo block",
            "faces": [{
                "index": 3,
                "name": "face:3",
                "area": 188.495559,
                "centroid": [20.0, 15.0, 10.0],
                "normal": null,
                "point": [23.0, 15.0, 10.0],
                "cylinder": {"origin": [20.0, 15.0, 0.0], "axis": [0.0, 0.0, 1.0], "radius": 3.0}
            }]
        }]
    });
    let imported = solidcraft::import_snapshot(&snapshot.to_string()).unwrap();
    assert!(imported.read_only);
    assert_eq!(imported.timeline.len(), 1);
    assert_eq!(imported.cylindrical_faces.len(), 1);
    assert_eq!(imported.cylindrical_faces[0].diameter_mm, 6.0);
    assert!(imported.cylindrical_faces[0].timeline_feature_id.is_none());
    assert!(!imported.per_feature_geometry_available);
}

#[test]
fn adapter_fails_closed_on_versions_and_invalid_geometry() {
    let base = json!({
        "schema_version": 1,
        "solidcraft_version": "0.18.0",
        "inspect_design": {"timeline": [], "bodies": []},
        "body_topology": []
    });
    assert!(solidcraft::import_snapshot(&base.to_string()).is_err());
    let invalid = json!({
        "schema_version": 1,
        "solidcraft_version": "0.17.0",
        "inspect_design": {"timeline": [], "bodies": []},
        "body_topology": [{"body":"Demo","faces":[{"index":1,"name":"face:1","area":1,"centroid":[0,0,0],"normal":null,"point":[0,0,0],"cylinder":{"origin":[0,0,0],"axis":[0,0,2],"radius":3}}]}]
    });
    assert!(solidcraft::import_snapshot(&invalid.to_string()).is_err());
}
