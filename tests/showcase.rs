use pixcam_showcase::{core::*, nc, showcase};
fn close(a: Vec3, b: Vec3) {
    assert!(norm(sub(a, b)) < 1e-8, "{a:?} != {b:?}");
}
#[test]
fn synthetic_geometry_and_groups_are_consistent() {
    let d = showcase::demo().unwrap();
    assert_eq!(d.features.len(), 9);
    assert_eq!(d.groups.len(), 5);
    assert_eq!(d.stock.max_mm, [120., 76., 20.]);
    let ids = d
        .groups
        .values()
        .flatten()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 9);
    for f in &d.features {
        assert!(ids.contains(&f.id));
        close(f.axis_into_stock, f.face.normal());
        assert_eq!(f.outline_mm.first(), f.outline_mm.last());
        for p in &f.outline_mm {
            for (i, v) in p.iter().enumerate() {
                assert!(*v >= d.stock.min_mm[i] && *v <= d.stock.max_mm[i]);
            }
        }
    }
    let slot = d.features.iter().find(|f| f.id == "S01").unwrap();
    assert!(slot.diameter_mm.is_none());
    let xs = slot.outline_mm.iter().map(|p| p[0]).collect::<Vec<_>>();
    assert_eq!(xs.iter().copied().reduce(f64::min), Some(13.));
    assert_eq!(xs.iter().copied().reduce(f64::max), Some(51.));
}
#[test]
fn exports_include_noncircular_geometry_and_provenance() {
    let d = showcase::demo().unwrap();
    let svg = showcase::svg(&d);
    let csv = showcase::csv(&d);
    let dxf = showcase::dxf(&d);
    assert_eq!(csv.lines().count(), 10);
    assert_eq!(dxf.matches("\nCIRCLE\n").count(), 7);
    assert!(dxf.matches("\nLINE\n").count() > 90);
    assert!(dxf.contains("$INSUNITS\n70\n4"));
    for f in &d.features {
        assert!(svg.contains(&format!("data-id=\"{}\"", f.id)));
        assert!(csv.contains(&f.id));
        assert!(dxf.contains(&f.id));
    }
    assert!(csv.contains("Original synthetic definition"));
}
#[test]
fn nc_known_endpoints_and_tools() {
    let d = showcase::demo().unwrap();
    assert_eq!(d.program.motions.len(), 27);
    let m = &d.program.motions;
    close(m[0].end_mm, [0., 0., 30.]);
    close(m[2].end_mm, [16., 14., 12.]);
    assert_eq!(m[2].tool, Some(1));
    assert!(m.iter().any(|m| m.kind == "arc" && m.tool == Some(3)));
    for pair in m.windows(2) {
        close(pair[0].end_mm, pair[1].start_mm);
    }
}
#[test]
fn modal_units_and_incremental_moves() {
    let p = nc::parse("G20 G90 G1 X1 F2\nG21 G91 X2", &nc::NcConfig::default()).unwrap();
    close(p.motions[0].end_mm, [25.4, 0., 0.]);
    close(p.motions[1].end_mm, [27.4, 0., 0.]);
    assert_eq!(p.motions[0].feed_mm_min, Some(50.8));
}
#[test]
fn arcs_all_planes_have_bounded_chord_error() {
    for (g, start, end, ijk, u, v) in [
        ("G17", "X10", "X0 Y10", "I-10 J0", 0, 1),
        ("G18", "Z10", "Z0 X10", "K-10 I0", 2, 0),
        ("G19", "Y10", "Y0 Z10", "J-10 K0", 1, 2),
    ] {
        let p = nc::parse(
            &format!("G0 {start}\n{g} G3 {end} {ijk} F100"),
            &nc::NcConfig::default(),
        )
        .unwrap();
        let m = &p.motions[1];
        assert!((nc::length(m) - 5. * std::f64::consts::PI).abs() < 0.03);
        for pair in m.points_mm.windows(2) {
            let mid: Vec3 = std::array::from_fn(|i| (pair[0][i] + pair[1][i]) / 2.);
            assert!(10. - mid[u].hypot(mid[v]) <= 0.020000001);
        }
    }
}
#[test]
fn helix_full_circle_and_clockwise() {
    let p = nc::parse(
        "G0 X10\nG2 X0 Y-10 I-10 J0 F100\nG3 I0 J10 Z5",
        &nc::NcConfig::default(),
    )
    .unwrap();
    assert!(p.motions[1].points_mm[1][1] < 0.);
    close(p.motions[2].end_mm, [0., -10., 5.]);
    assert!(nc::length(&p.motions[2]) > 62.);
}
#[test]
fn unsupported_nc_fails_closed() {
    for input in [
        "CYCLE800()",
        "G81 X1",
        "G41 G1 X1 F100",
        "G43 H1",
        "G92 X0",
        "G54 G0 X1",
        "G2 R2 F100",
        "G0 X1 X2",
        "G90 G91",
        "G0 G1 X1",
        "G1 X1",
        "G2 X5 I1 F100",
        "G0 XNaN",
        "G0 X1 (open",
        "G0 X1\nM30\nG0 X2",
        "M98 P1",
        "G0 A30",
        "G0 X1 $",
        "G0 I1",
    ] {
        assert!(
            nc::parse(input, &nc::NcConfig::default()).is_err(),
            "{input}"
        );
    }
}
#[test]
fn transform_rotation_translation_and_roundtrip() {
    let t = Transform {
        rotation: [[0., -1., 0.], [1., 0., 0.], [0., 0., 1.]],
        translation_mm: [11., 23., 7.],
    };
    t.validate().unwrap();
    close(t.point([2., 3., 4.]), [8., 25., 11.]);
    close(t.vector([2., 3., 4.]), [-3., 2., 4.]);
    close(t.inverse_point(t.point([4., 6., 8.])), [4., 6., 8.]);
    let mut config = nc::NcConfig::default();
    config.work_offsets.insert("G54".into(), t);
    let p = nc::parse("G54 G0 X2 Y3 Z4\nG91 X1", &config).unwrap();
    close(p.motions[0].end_mm, [8., 25., 11.]);
    close(p.motions[1].end_mm, [8., 26., 11.]);
}
#[test]
fn reflection_scale_and_nonfinite_are_rejected() {
    for value in [-1., 2., f64::NAN, f64::INFINITY] {
        let mut t = Transform::default();
        t.rotation[0][0] = value;
        assert!(t.validate().is_err());
    }
}
#[test]
fn build_is_deterministic_and_embedded_scene_is_parseable() {
    let root = std::env::temp_dir().join(format!("pixcam-public-test-{}", std::process::id()));
    showcase::build(&root).unwrap();
    let first = std::fs::read(root.join("index.html")).unwrap();
    showcase::build(&root).unwrap();
    assert_eq!(first, std::fs::read(root.join("index.html")).unwrap());
    let text = String::from_utf8(first).unwrap();
    let json = text
        .split("<script id=\"scene-data\" type=\"application/json\">")
        .nth(1)
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    let d: showcase::Demo = serde_json::from_str(json).unwrap();
    assert_eq!(d.features.len(), 9);
    assert!(!text.contains("__DATA__"));
}
