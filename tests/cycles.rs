use pixcam_showcase::{
    coordinates::Mat4,
    core::{Vec3, norm, sub},
    cycles::{self, Direction, Pattern, Request},
    registry,
};
use serde_json::{Value, json};
fn request() -> Request {
    serde_json::from_str(include_str!("../fixtures/cycle-circular.json")).unwrap()
}
fn close(a: Vec3, b: Vec3) {
    assert!(norm(sub(a, b)) < 1e-8, "{a:?} != {b:?}");
}
fn circular(n: u32, sweep: f64, start: f64, direction: Direction) -> Request {
    Request {
        cycle_id: "TEST".into(),
        global_from_local: Mat4::IDENTITY,
        pattern: Pattern::Circular {
            center_mm: [2., 3., 4.],
            radius_mm: Some(10.),
            pitch_diameter_mm: None,
            start_deg: start,
            sweep_deg: sweep,
            positions: n,
            direction,
            feature_diameter_mm: 2.,
        },
    }
}
#[test]
fn four_and_eight_full_circle_points_no_duplicate_closure() {
    for n in [4, 8] {
        let r = cycles::generate(&circular(n, 360., 0., Direction::CounterClockwise)).unwrap();
        assert_eq!(r.features.len(), n as usize);
        assert!(r.closed_pattern);
        for (i, f) in r.features.iter().enumerate() {
            let a = (i as f64 * 360. / n as f64).to_radians();
            close(f.local_xyz_mm, [2. + 10. * a.cos(), 3. + 10. * a.sin(), 4.]);
            assert_eq!(f.position_index, i as u32);
        }
        assert!(
            norm(sub(
                r.features[0].local_xyz_mm,
                r.features.last().unwrap().local_xyz_mm
            )) > 1.
        );
    }
}
#[test]
fn open_arc_endpoints_start_and_direction() {
    let ccw = cycles::generate(&circular(3, 90., 90., Direction::CounterClockwise)).unwrap();
    close(ccw.features[0].local_xyz_mm, [2., 13., 4.]);
    close(ccw.features[2].local_xyz_mm, [-8., 3., 4.]);
    assert!(!ccw.closed_pattern);
    let cw = cycles::generate(&circular(3, 90., 90., Direction::Clockwise)).unwrap();
    close(cw.features[2].local_xyz_mm, [12., 3., 4.]);
    let one = cycles::generate(&circular(1, 90., 450., Direction::Clockwise)).unwrap();
    close(one.features[0].local_xyz_mm, [2., 13., 4.]);
}
#[test]
fn pitch_diameter_and_radius_agree() {
    let mut q = circular(8, 360., -45., Direction::Clockwise);
    let a = cycles::generate(&q).unwrap();
    if let Pattern::Circular {
        radius_mm,
        pitch_diameter_mm,
        ..
    } = &mut q.pattern
    {
        *radius_mm = None;
        *pitch_diameter_mm = Some(20.);
    }
    let b = cycles::generate(&q).unwrap();
    for (a, b) in a.features.iter().zip(b.features) {
        close(a.local_xyz_mm, b.local_xyz_mm);
    }
}
#[test]
fn grid_four_columns_three_rows_and_rotation() {
    let mut q: Request =
        serde_json::from_str(include_str!("../fixtures/cycle-rectangular.json")).unwrap();
    if let Pattern::Rectangular { rotation_deg, .. } = &mut q.pattern {
        *rotation_deg = 0.;
    }
    let r = cycles::generate(&q).unwrap();
    assert_eq!(r.features.len(), 12);
    close(r.features[0].local_xyz_mm, [-30., -20., 0.]);
    close(r.features[3].local_xyz_mm, [30., -20., 0.]);
    close(r.features[11].local_xyz_mm, [30., 20., 0.]);
    if let Pattern::Rectangular { rotation_deg, .. } = &mut q.pattern {
        *rotation_deg = 90.;
    }
    let r = cycles::generate(&q).unwrap();
    close(r.features[3].local_xyz_mm, [-30., 40., 0.]);
    close(r.features[11].local_xyz_mm, [-70., 40., 0.]);
}
#[test]
fn arbitrary_frame_roundtrip_and_tool_axis() {
    let mut q = request();
    q.global_from_local = Mat4::translation([101., -23., 7.])
        .unwrap()
        .compose(Mat4::rotation_z(20.).unwrap())
        .compose(Mat4::rotation_y(-35.).unwrap())
        .compose(Mat4::rotation_x(30.).unwrap());
    let r = cycles::generate(&q).unwrap();
    for f in &r.features {
        close(
            q.global_from_local
                .inverse()
                .unwrap()
                .point(f.global_xyz_mm),
            f.local_xyz_mm,
        );
        close(
            f.tool_axis_global,
            q.global_from_local.vector([0., 0., -1.]),
        );
    }
    let b = cycles::generate(&q).unwrap();
    assert_eq!(
        serde_json::to_string(&r).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
#[test]
fn invalid_parameters_and_numerical_duplicate_positions_fail_closed() {
    let base = serde_json::to_value(request()).unwrap();
    for (path, value) in [
        ("/pattern/radius_mm", json!(0)),
        ("/pattern/pitch_diameter_mm", json!(50)),
        ("/pattern/sweep_deg", json!(0)),
        ("/pattern/sweep_deg", json!(361)),
        ("/pattern/positions", json!(0)),
        ("/pattern/positions", json!(1025)),
        ("/pattern/feature_diameter_mm", json!(-1)),
        ("/cycle_id", json!("<unsafe>")),
        (
            "/global_from_local",
            json!([[2, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]]),
        ),
    ] {
        let mut v = base.clone();
        *v.pointer_mut(path).unwrap() = value;
        let q: Request = serde_json::from_value(v).unwrap();
        assert!(cycles::generate(&q).is_err(), "{path}");
    }
    assert!(cycles::generate(&circular(10, 1e-12, 0., Direction::Clockwise)).is_err());
    let mut q = circular(4, 360., 0., Direction::Clockwise);
    if let Pattern::Circular { start_deg, .. } = &mut q.pattern {
        *start_deg = f64::NAN;
    }
    assert!(cycles::generate(&q).is_err());
    let mut q: Value =
        serde_json::from_str(include_str!("../fixtures/cycle-rectangular.json")).unwrap();
    q["pattern"]["spacing_x_mm"] = json!(1e-12);
    assert!(cycles::generate(&serde_json::from_value(q).unwrap()).is_err());
    let mut q = request();
    q.global_from_local = Mat4::translation([1e30, 0., 0.]).unwrap();
    assert!(cycles::generate(&q).is_err());
}
#[test]
fn rectangle_invalid_counts_spacing_and_angles() {
    let base: Value =
        serde_json::from_str(include_str!("../fixtures/cycle-rectangular.json")).unwrap();
    for (key, val) in [
        ("rows", json!(0)),
        ("columns", json!(1025)),
        ("rows", json!(1024)),
        ("spacing_y_mm", json!(0)),
        ("spacing_x_mm", json!(-1)),
    ] {
        let mut v = base.clone();
        v["pattern"][key] = val;
        assert!(cycles::generate(&serde_json::from_value(v).unwrap()).is_err());
    }
    let mut q: Request = serde_json::from_value(base).unwrap();
    if let Pattern::Rectangular { rotation_deg, .. } = &mut q.pattern {
        *rotation_deg = f64::INFINITY;
    }
    assert!(cycles::generate(&q).is_err());
}
#[test]
fn csv_svg_and_dxf_export_registry_coordinates_exactly() {
    let mut q = request();
    q.global_from_local = Mat4::translation([11., 23., 7.])
        .unwrap()
        .compose(Mat4::rotation_y(37.).unwrap());
    let r = cycles::generate(&q).unwrap();
    let csv = cycles::csv(&r);
    let svg = cycles::svg(&r);
    let dxf = cycles::dxf(&r);
    for (i, line) in csv.lines().skip(1).enumerate() {
        let fields: Vec<_> = line.split(',').collect();
        for j in 0..3 {
            let local = fields[j + 3].trim_matches('"').parse::<f64>().unwrap();
            let global = fields[j + 6].trim_matches('"').parse::<f64>().unwrap();
            assert_eq!(local, r.features[i].local_xyz_mm[j]);
            assert_eq!(global, r.features[i].global_xyz_mm[j]);
        }
    }
    assert_eq!(svg.matches("<circle ").count(), 8);
    assert_eq!(dxf.matches("\nCIRCLE\n").count(), 8);
    assert_eq!(dxf.matches("\nPOINT\n").count(), 8);
    // Independent OCS reconstruction for a 37-degree Y tilt: Ax=+Y,
    // Ay=(-cos,0,sin), normal=(sin,0,cos). Translation is included.
    let (sin, cos) = 37_f64.to_radians().sin_cos();
    for (entity, f) in dxf.split("0\nCIRCLE\n").skip(1).zip(&r.features) {
        let lines: Vec<_> = entity.lines().collect();
        let mut groups = std::collections::BTreeMap::new();
        for pair in lines.as_chunks::<2>().0 {
            if pair[0] == "0" {
                break;
            }
            groups.insert(pair[0], pair[1]);
        }
        let value = |code| groups[code].parse::<f64>().unwrap();
        let (x, y, z) = (value("10"), value("20"), value("30"));
        close([-cos * y + sin * z, x, sin * y + cos * z], f.global_xyz_mm);
        close([value("210"), value("220"), value("230")], [sin, 0., cos]);
        assert_eq!(value("40"), f.diameter_mm / 2.);
    }
    let lines: Vec<_> = dxf.lines().collect();
    let mut centers = Vec::new();
    for i in 0..lines.len() {
        if lines[i] == "POINT" {
            centers.push([
                lines[i + 4].parse().unwrap(),
                lines[i + 6].parse().unwrap(),
                lines[i + 8].parse().unwrap(),
            ]);
        }
    }
    for (p, f) in centers.into_iter().zip(&r.features) {
        close(p, f.global_xyz_mm);
    }
    for f in &r.features {
        assert!(svg.contains(&format!(
            "data-global-xyz=\"{} {} {}\"",
            f.global_xyz_mm[0], f.global_xyz_mm[1], f.global_xyz_mm[2]
        )));
    }
}
#[test]
fn shared_registry_preserves_cad_and_rejects_tampered_cycle() {
    let cad =
        pixcam_showcase::recognition::analyze(include_str!("../fixtures/bridge-snapshot.json"))
            .unwrap();
    let result = cycles::generate(&request()).unwrap();
    let registry = registry::combine(Some(cad), vec![result.clone()]).unwrap();
    assert_eq!(registry.feature_count, 13);
    assert_eq!(registry.imported_cad.unwrap().features.len(), 5);
    assert!(registry::combine(None, vec![result.clone(), result.clone()]).is_err());
    let mut tampered = result;
    tampered.features[0].global_xyz_mm[0] += 1.;
    assert!(registry::combine(None, vec![tampered]).is_err());
}
