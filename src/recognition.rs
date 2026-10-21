//! Deterministic geometry registry. Measured surfaces and manufacturing hypotheses stay separate.
use crate::{
    Result,
    coordinates::Mat4,
    core::{Vec3, dot, finite, norm},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
const TOL: f64 = 1e-5;
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub schema_version: u32,
    pub solidcraft_version: String,
    pub inspect_design: Design,
    pub body_topology: Vec<Topology>,
    #[serde(default)]
    pub measurements: Vec<Value>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Design {
    #[serde(default)]
    pub name: String,
    #[serde(default = "mm")]
    pub units: String,
    #[serde(default)]
    pub revision: Value,
    pub bodies: Vec<Body>,
    #[serde(default)]
    pub timeline: Vec<Value>,
}
fn mm() -> String {
    "mm".into()
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Body {
    pub name: String,
    pub bbox: Bounds,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topology {
    pub body: String,
    pub faces: Vec<Face>,
    #[serde(default)]
    pub edges: Vec<Edge>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Edge {
    pub index: u32,
    pub name: String,
    pub start: Vec3,
    pub mid: Vec3,
    pub end: Vec3,
    pub length: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Face {
    pub index: u32,
    pub name: String,
    pub area: f64,
    pub centroid: Vec3,
    pub point: Vec3,
    #[serde(default, alias = "normal")]
    pub plane_normal: Option<Vec3>,
    #[serde(default)]
    pub cylinder: Option<Cylinder>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cylinder {
    pub origin: Vec3,
    pub axis: Vec3,
    pub radius: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub body: String,
    pub face_indices: Vec<u32>,
    pub face_names: Vec<String>,
    pub evidence_level: String,
    pub diameter_mm: f64,
    pub axis: Vec3,
    pub axis_origin_mm: Vec3,
    pub axial_range_mm: Option<[f64; 2]>,
    pub boundary_span_mm: Option<f64>,
    pub cad_from_local: Mat4,
    pub inferred_feature: String,
    pub inference_level: String,
    pub unknown: Vec<String>,
    pub evidence: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub product: String,
    pub document: String,
    pub units: String,
    pub read_only: bool,
    pub bodies: Vec<Body>,
    pub features: Vec<Feature>,
    pub surfaces: Vec<Value>,
    pub groups: BTreeMap<String, Vec<String>>,
    pub coaxial_groups: Vec<Vec<String>>,
    pub potential_operations: Vec<Value>,
    pub limitations: Vec<String>,
    pub snapshot: Snapshot,
}
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] - b[i])
}
fn canonical(mut a: Vec3) -> Vec3 {
    if a.iter().find(|x| x.abs() > TOL).is_some_and(|x| *x < 0.) {
        a = a.map(|x| -x);
    }
    a.map(|x| if x.abs() < 1e-12 { 0. } else { x })
}
pub fn same_axis(a: Vec3, p: Vec3, b: Vec3, q: Vec3) -> bool {
    norm(cross(a, b)) < TOL && norm(cross(sub(q, p), a)) < TOL
}
fn frame(axis: Vec3, origin: Vec3) -> Result<Mat4> {
    let seed = if axis[0].abs() < 0.8 {
        [1., 0., 0.]
    } else {
        [0., 1., 0.]
    };
    let x = cross(seed, axis);
    let n = norm(x);
    let x = x.map(|v| v / n);
    let y = cross(axis, x);
    let mut m = Mat4([
        [x[0], y[0], axis[0], origin[0]],
        [x[1], y[1], axis[1], origin[1]],
        [x[2], y[2], axis[2], origin[2]],
        [0., 0., 0., 1.],
    ]);
    for value in m.0.iter_mut().flatten() {
        if *value == 0.0 {
            *value = 0.0;
        }
    }
    m.validate()?;
    Ok(m)
}
pub fn validate(s: &Snapshot) -> Result<()> {
    if s.schema_version != 1 || s.solidcraft_version != "0.17.0" || s.inspect_design.units != "mm" {
        return Err("Require schema 1, SolidCraft 0.17.0 and millimetres".into());
    }
    if s.inspect_design.bodies.len() > 256 {
        return Err("Too many bodies".into());
    }
    let mut bodies = BTreeSet::new();
    for b in &s.inspect_design.bodies {
        if b.name.is_empty()
            || !bodies.insert(&b.name)
            || !finite(b.bbox.min)
            || !finite(b.bbox.max)
            || (0..3).any(|i| b.bbox.max[i] < b.bbox.min[i])
        {
            return Err("Invalid/duplicate body or bounding box".into());
        }
    }
    let mut topology_bodies = BTreeSet::new();
    for t in &s.body_topology {
        if !bodies.contains(&t.body)
            || !topology_bodies.insert(&t.body)
            || t.faces.len() > 100_000
            || t.edges.len() > 200_000
        {
            return Err("Unknown/duplicate topology body or topology limit".into());
        }
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for f in &t.faces {
            if !ids.insert(f.index)
                || f.name.is_empty()
                || !names.insert(&f.name)
                || !finite(f.point)
                || !finite(f.centroid)
                || !f.area.is_finite()
                || f.area < 0.
                || f.plane_normal
                    .is_some_and(|n| !finite(n) || (norm(n) - 1.).abs() > TOL)
            {
                return Err("Invalid/duplicate face geometry".into());
            }
            if let Some(c) = &f.cylinder {
                if !finite(c.origin)
                    || !finite(c.axis)
                    || (norm(c.axis) - 1.).abs() > 1e-8
                    || !c.radius.is_finite()
                    || c.radius <= 0.
                {
                    return Err("Invalid cylinder geometry".into());
                }
                let radial = norm(cross(sub(f.point, c.origin), c.axis));
                if (radial - c.radius).abs() > 1e-3 {
                    return Err("Cylinder surface point disagrees with radius".into());
                }
            }
        }
        let mut ids = BTreeSet::new();
        for e in &t.edges {
            if !ids.insert(e.index)
                || !finite(e.start)
                || !finite(e.mid)
                || !finite(e.end)
                || !e.length.is_finite()
                || e.length < 0.
            {
                return Err("Invalid/duplicate edge geometry".into());
            }
        }
    }
    Ok(())
}
pub fn analyze(text: &str) -> Result<Report> {
    if text.len() > 16 * 1024 * 1024 {
        return Err("Snapshot exceeds 16 MiB".into());
    }
    let s: Snapshot =
        serde_json::from_str(text).map_err(|_| "Incomplete/invalid geometry snapshot")?;
    validate(&s)?;
    let mut features: Vec<Feature> = Vec::new();
    let mut surfaces = Vec::new();
    for t in &s.body_topology {
        for f in &t.faces {
            surfaces.push(json!({"body":t.body,"face_index":f.index,"persistent_name":f.name,"area_mm2":f.area,
                "centroid_mm":f.centroid,"normal":f.plane_normal,"kind":if f.cylinder.is_some(){"cylindrical"}else if f.plane_normal.is_some(){"planar"}else{"unknown"},"evidence_level":"VERIFIED_GEOMETRY"}));
            let Some(c) = &f.cylinder else { continue };
            let axis = canonical(c.axis);
            // Persistent edge names encode their adjacent face names, with optional ~edge suffix.
            // Only exact face tokens count: substrings must never associate unrelated faces.
            let pts: Vec<Vec3> = t
                .edges
                .iter()
                .filter(|e| {
                    e.name
                        .split('|')
                        .any(|part| part.split('~').next() == Some(f.name.as_str()))
                })
                .flat_map(|e| [e.start, e.mid, e.end])
                .collect();
            let range = if pts.is_empty() {
                None
            } else {
                let lo = pts
                    .iter()
                    .map(|p| dot(*p, axis))
                    .fold(f64::INFINITY, f64::min);
                let hi = pts
                    .iter()
                    .map(|p| dot(*p, axis))
                    .fold(f64::NEG_INFINITY, f64::max);
                Some([lo, hi])
            };
            let origin = sub(c.origin, axis.map(|v| v * dot(c.origin, axis)));
            let span = range.map(|r| r[1] - r[0]);
            let merge=features.iter().position(|other|other.body==t.body && (other.diameter_mm-c.radius*2.).abs()<TOL
                && same_axis(other.axis,other.axis_origin_mm,axis,origin)
                && matches!((other.axial_range_mm,range),(Some(a),Some(b)) if a[0]<=b[1]+TOL && b[0]<=a[1]+TOL));
            if let Some(i) = merge {
                let other = &mut features[i];
                other.face_indices.push(f.index);
                other.face_names.push(f.name.clone());
                let a = other.axial_range_mm.unwrap();
                let b = range.unwrap();
                other.axial_range_mm = Some([a[0].min(b[0]), a[1].max(b[1])]);
                other.boundary_span_mm = other.axial_range_mm.map(|r| r[1] - r[0]);
                continue;
            }
            let local_origin = if let Some(r) = range {
                std::array::from_fn(|i| origin[i] + axis[i] * r[0])
            } else {
                c.origin
            };
            features.push(Feature {id:format!("CY{:02}",features.len()+1),body:t.body.clone(),face_indices:vec![f.index],face_names:vec![f.name.clone()],
                evidence_level:"VERIFIED_GEOMETRY".into(),diameter_mm:c.radius*2.,axis,axis_origin_mm:origin,
                axial_range_mm:range,boundary_span_mm:span,cad_from_local:frame(axis,local_origin)?,
                inferred_feature:if f.name.contains(":bore:")||f.name.contains(":cbore:"){"cylindrical_hole_candidate"}else{"cylindrical_surface_unknown"}.into(),
                inference_level:if f.name.contains(":bore:")||f.name.contains(":cbore:"){"INFERRED_FEATURE"}else{"UNKNOWN"}.into(),
                unknown:vec!["Closed 360-degree hole / inward material side".into(),"Blind versus through termination (not certified by topology alone)".into(),"Threads, tolerances, process and tool".into()],
                evidence:"Radius/axis from CAD cylinder; span is projection of named adjacent edge endpoints/midpoints, not an invented hole depth. Hole hypothesis uses CAD persistent naming only.".into()});
        }
    }
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut coaxial_groups: Vec<Vec<String>> = Vec::new();
    for f in &features {
        groups
            .entry(format!("diameter / {:.5} mm", f.diameter_mm))
            .or_default()
            .push(f.id.clone());
        groups
            .entry(format!(
                "parallel axis / {:.5},{:.5},{:.5}",
                f.axis[0], f.axis[1], f.axis[2]
            ))
            .or_default()
            .push(f.id.clone());
    }
    for (i, f) in features.iter().enumerate() {
        if coaxial_groups.iter().any(|g| g.contains(&f.id)) {
            continue;
        }
        let group: Vec<String> = features
            .iter()
            .skip(i)
            .filter(|o| {
                o.body == f.body && same_axis(f.axis, f.axis_origin_mm, o.axis, o.axis_origin_mm)
            })
            .map(|o| o.id.clone())
            .collect();
        if group.len() > 1 {
            coaxial_groups.push(group);
        }
    }
    let mut operations = Vec::new();
    for group in &coaxial_groups {
        let members: Vec<&Feature> = features.iter().filter(|f| group.contains(&f.id)).collect();
        if members.iter().any(|a|members.iter().any(|b|(a.diameter_mm-b.diameter_mm).abs()>TOL
            && matches!((a.axial_range_mm,b.axial_range_mm),(Some(x),Some(y)) if (x[1]-y[0]).abs()<TOL||(y[1]-x[0]).abs()<TOL))) {
            operations.push(json!({"level":"INFERRED_FEATURE","kind":"stepped_diameters_possible_counterbore","features":group,"requires_review":"Material side, complete circumference, access and tooling"}));
        }
    }
    for (direction, axis) in [
        ("+Z", [0., 0., 1.]),
        ("-Z", [0., 0., -1.]),
        ("+X", [1., 0., 0.]),
        ("-X", [-1., 0., 0.]),
        ("+Y", [0., 1., 0.]),
        ("-Y", [0., -1., 0.]),
    ] {
        let ids: Vec<_> = features
            .iter()
            .filter(|f| norm(cross(f.axis, axis)) < TOL)
            .map(|f| f.id.clone())
            .collect();
        if !ids.is_empty() {
            operations.push(json!({"level":"INFERRED_FEATURE","kind":"axis_aligned_setup_candidates","tool_direction":direction,"features":ids,"requires_review":"Direction sign, access, collisions, material and operation must be verified"}));
        }
    }
    let mut limitations=vec!["Planning aid only; no NC generation, certified machining, access/collision proof or CYCLE800.".into(),
        "Cylindrical patches are not automatically complete holes. Opposite axis signs describe the same geometric line, not a verified tool direction.".into(),
        "Boundary spans require exact persistent edge adjacency; missing spans stay null. Edge wireframe samples are not a CAD tessellation.".into(),
        "Timeline and evaluated faces are not automatically associated; no feature-to-face mapping is invented.".into()];
    if s.body_topology.len() != s.inspect_design.bodies.len() {
        limitations.push(
            "Incomplete topology: some bodies have no surface data; analysis is partial.".into(),
        );
    }
    Ok(Report {
        schema_version: 1,
        product: "pixCAM 0.3.0".into(),
        document: s.inspect_design.name.clone(),
        units: "mm".into(),
        read_only: true,
        bodies: s.inspect_design.bodies.clone(),
        features,
        surfaces,
        groups,
        coaxial_groups,
        potential_operations: operations,
        limitations,
        snapshot: s,
    })
}
pub fn csv(r: &Report) -> String {
    let mut out = String::from(
        "id,body,evidence_level,inference_level,diameter_mm,axis_x,axis_y,axis_z,axis_origin_x,axis_origin_y,axis_origin_z,boundary_span_mm,hypothesis\r\n",
    );
    for f in &r.features {
        let row = [
            f.id.clone(),
            f.body.clone(),
            f.evidence_level.clone(),
            f.inference_level.clone(),
            f.diameter_mm.to_string(),
            f.axis[0].to_string(),
            f.axis[1].to_string(),
            f.axis[2].to_string(),
            f.axis_origin_mm[0].to_string(),
            f.axis_origin_mm[1].to_string(),
            f.axis_origin_mm[2].to_string(),
            f.boundary_span_mm
                .map(|x| x.to_string())
                .unwrap_or_default(),
            f.inferred_feature.clone(),
        ];
        out.push_str(
            &row.iter()
                .map(|v| format!("\"{}\"", v.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(","),
        );
        out.push_str("\r\n");
    }
    out
}
