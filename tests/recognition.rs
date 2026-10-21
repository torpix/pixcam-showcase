use pixcam_showcase::{
    coordinates::Mat4,
    recognition::{analyze, same_axis},
};
use serde_json::{Value, json};
const SNAPSHOT: &str = include_str!("../fixtures/bridge-snapshot.json");
#[test]
fn real_cad_specimen_matches_design_dimensions_and_axes() {
    let r = analyze(SNAPSHOT).unwrap();
    assert_eq!(r.document, "bridge-part");
    assert_eq!(r.bodies.len(), 1);
    for (actual, expected) in r.bodies[0].bbox.max.iter().zip([100., 70., 30.]) {
        assert!((actual - expected).abs() < 1e-7);
    }
    assert_eq!(r.features.len(), 5); // ten CAD patches, five cylindrical segments
    for (d, span, axis, p) in [
        (6., 30., [0., 0., 1.], [20., 20., 0.]),
        (10., 12., [0., 0., 1.], [45., 20., 0.]),
        (12., 5., [0., 0., 1.], [75., 20., 0.]),
        (6., 25., [0., 0., 1.], [75., 20., 0.]),
        (8., 15., [1., 0., 0.], [0., 50., 15.]),
    ] {
        let f = r
            .features
            .iter()
            .find(|f| {
                (f.diameter_mm - d).abs() < 1e-6 && same_axis(f.axis, f.axis_origin_mm, axis, p)
            })
            .unwrap();
        assert!((f.boundary_span_mm.unwrap() - span).abs() < 1e-6);
        assert_eq!(f.face_indices.len(), 2);
        assert_eq!(f.evidence_level, "VERIFIED_GEOMETRY");
        assert!(!f.unknown.is_empty());
        f.cad_from_local.validate().unwrap();
        let local = [2., 3., 4.];
        let back = f
            .cad_from_local
            .inverse()
            .unwrap()
            .point(f.cad_from_local.point(local));
        for i in 0..3 {
            assert!((local[i] - back[i]).abs() < 1e-8);
        }
    }
    assert_eq!(r.coaxial_groups.len(), 1);
    assert!(
        r.potential_operations
            .iter()
            .any(|o| o["kind"] == "stepped_diameters_possible_counterbore")
    );
    assert!(
        r.surfaces
            .iter()
            .any(|s| s["persistent_name"] == "F6:-z" && s["kind"] == "planar")
    );
}
#[test]
fn incomplete_edges_never_invent_span_or_closed_hole() {
    let mut v: Value = serde_json::from_str(SNAPSHOT).unwrap();
    v["body_topology"][0]["edges"] = json!([]);
    let r = analyze(&v.to_string()).unwrap();
    assert_eq!(r.features.len(), 10);
    assert!(
        r.features
            .iter()
            .all(|f| f.boundary_span_mm.is_none() && !f.unknown.is_empty())
    );
}
#[test]
fn invalid_and_inconsistent_geometry_rejected() {
    for (pointer, value) in [
        ("/solidcraft_version", json!("0.16.2")),
        ("/inspect_design/units", json!("in")),
        ("/body_topology/0/faces/4/cylinder/radius", json!(-1)),
        ("/body_topology/0/faces/4/cylinder/axis", json!([0, 0, 2])),
        ("/body_topology/0/faces/4/point", json!([999, 999, 999])),
        ("/inspect_design/bodies/0/bbox/max", json!([-1, 0, 0])),
        ("/body_topology/0/body", json!("absent")),
    ] {
        let mut v: Value = serde_json::from_str(SNAPSHOT).unwrap();
        *v.pointer_mut(pointer).unwrap() = value;
        assert!(analyze(&v.to_string()).is_err(), "{pointer}");
    }
    assert!(analyze("{}").is_err());
    assert!(analyze("{").is_err());
}
#[test]
fn external_cylinder_and_unlinked_edges_stay_unknown() {
    let mut v: Value = serde_json::from_str(SNAPSHOT).unwrap();
    for f in v["body_topology"][0]["faces"].as_array_mut().unwrap() {
        f["name"] = json!(format!("external{}", f["index"]));
    }
    let r = analyze(&v.to_string()).unwrap();
    assert!(
        r.features
            .iter()
            .all(|f| f.inference_level == "UNKNOWN" && f.boundary_span_mm.is_none())
    );
}
#[test]
fn opposite_axes_and_repeated_imports_are_deterministic() {
    assert!(same_axis(
        [0., 0., 1.],
        [2., 3., 4.],
        [0., 0., -1.],
        [2., 3., 40.]
    ));
    assert!(!same_axis(
        [0., 0., 1.],
        [2., 3., 4.],
        [0., 0., 1.],
        [2.1, 3., 4.]
    ));
    let a = analyze(SNAPSHOT).unwrap();
    let b = analyze(SNAPSHOT).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
    assert_eq!(Mat4::IDENTITY.point([1., 2., 3.]), [1., 2., 3.]);
}
