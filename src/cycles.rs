//! Parametric geometry only: no drilling technology, CAD mutation or NC generation.
use crate::{
    Result,
    coordinates::Mat4,
    core::{Vec3, finite, norm, sub},
};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
pub const MAX_POINTS: u32 = 1024;
pub const POSITION_TOLERANCE_MM: f64 = 1e-7;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub cycle_id: String,
    pub global_from_local: Mat4,
    pub pattern: Pattern,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Pattern {
    #[serde(rename = "CYCLE_CIRCULAR_PATTERN")]
    Circular {
        center_mm: Vec3,
        #[serde(default)]
        radius_mm: Option<f64>,
        #[serde(default)]
        pitch_diameter_mm: Option<f64>,
        start_deg: f64,
        sweep_deg: f64,
        positions: u32,
        #[serde(default)]
        direction: Direction,
        feature_diameter_mm: f64,
    },
    #[serde(rename = "CYCLE_RECTANGULAR_PATTERN")]
    Rectangular {
        start_mm: Vec3,
        rows: u32,
        columns: u32,
        spacing_x_mm: f64,
        spacing_y_mm: f64,
        rotation_deg: f64,
        feature_diameter_mm: f64,
    },
}
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Clockwise,
    #[default]
    CounterClockwise,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleFeature {
    pub id: String,
    pub cycle_id: String,
    pub position_index: u32,
    pub local_xyz_mm: Vec3,
    pub global_xyz_mm: Vec3,
    pub tool_axis_global: Vec3,
    pub diameter_mm: f64,
    pub source_parameters: Pattern,
    pub geometry_status: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CycleResult {
    pub schema_version: u32,
    pub units: String,
    pub request: Request,
    pub features: Vec<CycleFeature>,
    pub closed_pattern: bool,
    pub production_nc_generated: bool,
    pub limitations: Vec<String>,
}
fn positive(x: f64) -> bool {
    x.is_finite() && x > 0.
}
fn count(n: u32) -> Result<()> {
    if n == 0 || n > MAX_POINTS {
        Err("Point count must be 1..1024".into())
    } else {
        Ok(())
    }
}
fn radians(degrees: f64) -> f64 {
    degrees.rem_euclid(360.).to_radians()
}
pub fn generate(request: &Request) -> Result<CycleResult> {
    if request.cycle_id.is_empty()
        || request.cycle_id.len() > 64
        || !request
            .cycle_id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return Err("Cycle ID must use 1..64 ASCII letters, digits, underscore or hyphen".into());
    }
    request.global_from_local.validate()?;
    let inverse = request.global_from_local.inverse()?;
    let mut local = Vec::new();
    let diameter;
    let closed;
    match &request.pattern {
        Pattern::Circular {
            center_mm,
            radius_mm,
            pitch_diameter_mm,
            start_deg,
            sweep_deg,
            positions,
            direction,
            feature_diameter_mm,
        } => {
            count(*positions)?;
            let radius = match (radius_mm, pitch_diameter_mm) {
                (Some(r), None) if positive(*r) => *r,
                (None, Some(d)) if positive(*d) => *d / 2.,
                _ => return Err("Supply exactly one positive radius or pitch diameter".into()),
            };
            if !finite(*center_mm)
                || !start_deg.is_finite()
                || !sweep_deg.is_finite()
                || *sweep_deg <= 0.
                || *sweep_deg > 360.
                || !positive(*feature_diameter_mm)
                || !positive(radius)
            {
                return Err("Invalid circular pattern dimensions/angles".into());
            }
            closed = (*sweep_deg - 360.).abs() <= 1e-9;
            diameter = *feature_diameter_mm;
            let divisor = if closed {
                *positions
            } else {
                positions.saturating_sub(1).max(1)
            } as f64;
            let sign = match direction {
                Direction::Clockwise => -1.,
                Direction::CounterClockwise => 1.,
            };
            for index in 0..*positions {
                let angle =
                    radians(start_deg.rem_euclid(360.) + sign * sweep_deg * index as f64 / divisor);
                let (s, c) = angle.sin_cos();
                local.push([
                    center_mm[0] + radius * c,
                    center_mm[1] + radius * s,
                    center_mm[2],
                ]);
            }
        }
        Pattern::Rectangular {
            start_mm,
            rows,
            columns,
            spacing_x_mm,
            spacing_y_mm,
            rotation_deg,
            feature_diameter_mm,
        } => {
            count(*rows)?;
            count(*columns)?;
            let total = rows.checked_mul(*columns).ok_or("Grid count overflow")?;
            count(total)?;
            if !finite(*start_mm)
                || !positive(*spacing_x_mm)
                || !positive(*spacing_y_mm)
                || !rotation_deg.is_finite()
                || !positive(*feature_diameter_mm)
            {
                return Err("Invalid rectangular pattern dimensions/angle".into());
            }
            diameter = *feature_diameter_mm;
            closed = false;
            let (s, c) = radians(*rotation_deg).sin_cos();
            for row in 0..*rows {
                for column in 0..*columns {
                    let x = column as f64 * spacing_x_mm;
                    let y = row as f64 * spacing_y_mm;
                    local.push([
                        start_mm[0] + x * c - y * s,
                        start_mm[1] + x * s + y * c,
                        start_mm[2],
                    ]);
                }
            }
        }
    }
    let axis = request.global_from_local.vector([0., 0., -1.]);
    let mut features: Vec<CycleFeature> = Vec::new();
    for (index, p) in local.into_iter().enumerate() {
        let global = request.global_from_local.point(p);
        let back = inverse.point(global);
        if !finite(p)
            || !finite(global)
            || !finite(back)
            || !(2. * p[0].abs() + diameter + 4.).is_finite()
            || !(2. * p[1].abs() + diameter + 4.).is_finite()
            || norm(sub(back, p)) > 1e-6
        {
            return Err("Non-finite result or insufficient transform precision".into());
        }
        if features.iter().any(|f| {
            norm(sub(f.local_xyz_mm, p)) < POSITION_TOLERANCE_MM
                || norm(sub(f.global_xyz_mm, global)) < POSITION_TOLERANCE_MM
        }) {
            return Err("Duplicate or numerically indistinguishable positions (<1e-7 mm)".into());
        }
        features.push(CycleFeature {
            id: format!("{}:{index:04}", request.cycle_id),
            cycle_id: request.cycle_id.clone(),
            position_index: index as u32,
            local_xyz_mm: p,
            global_xyz_mm: global,
            tool_axis_global: axis,
            diameter_mm: diameter,
            source_parameters: request.pattern.clone(),
            geometry_status: "GENERATED_GEOMETRY".into(),
        });
    }
    Ok(CycleResult{schema_version:1,units:"mm".into(),request:request.clone(),features,closed_pattern:closed,production_nc_generated:false,
        limitations:vec!["User-defined geometric positions, not measured CAD holes or machining approval.".into(),"Tool axis is local -Z transformed into the global frame; access, depth, process, tolerances and tooling remain unspecified.".into(),"Full circles omit the repeated endpoint; open arcs include both ends (one position uses the start only). Rectangle order is row-major, indices start at zero.".into()]})
}
pub fn csv(result: &CycleResult) -> String {
    let mut out="id,cycle_id,position_index,local_x_mm,local_y_mm,local_z_mm,global_x_mm,global_y_mm,global_z_mm,axis_x,axis_y,axis_z,diameter_mm,geometry_status,source_parameters\r\n".to_string();
    for f in &result.features {
        let mut row = vec![
            f.id.clone(),
            f.cycle_id.clone(),
            f.position_index.to_string(),
        ];
        row.extend(
            f.local_xyz_mm
                .into_iter()
                .chain(f.global_xyz_mm)
                .chain(f.tool_axis_global)
                .map(|v| v.to_string()),
        );
        row.extend([
            f.diameter_mm.to_string(),
            f.geometry_status.clone(),
            serde_json::to_string(&f.source_parameters).unwrap(),
        ]);
        writeln!(
            out,
            "{}\r",
            row.iter()
                .map(|s| format!("\"{}\"", s.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(",")
        )
        .unwrap();
    }
    out
}
pub fn svg(result: &CycleResult) -> String {
    let minx = result
        .features
        .iter()
        .map(|f| f.local_xyz_mm[0] - f.diameter_mm / 2.)
        .fold(f64::INFINITY, f64::min);
    let miny = result
        .features
        .iter()
        .map(|f| f.local_xyz_mm[1] - f.diameter_mm / 2.)
        .fold(f64::INFINITY, f64::min);
    let maxx = result
        .features
        .iter()
        .map(|f| f.local_xyz_mm[0] + f.diameter_mm / 2.)
        .fold(f64::NEG_INFINITY, f64::max);
    let maxy = result
        .features
        .iter()
        .map(|f| f.local_xyz_mm[1] + f.diameter_mm / 2.)
        .fold(f64::NEG_INFINITY, f64::max);
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\"><title>pixCAM parametric positions / local XY / mm / not NC</title>",
        minx - 2.,
        -maxy - 2.,
        maxx - minx + 4.,
        maxy - miny + 4.
    );
    for f in &result.features {
        write!(out,"<circle data-id=\"{}\" data-global-xyz=\"{} {} {}\" cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"none\" stroke=\"#16a694\" stroke-width=\"0.3\"><title>{}</title></circle>",f.id,f.global_xyz_mm[0],f.global_xyz_mm[1],f.global_xyz_mm[2],f.local_xyz_mm[0],-f.local_xyz_mm[1],f.diameter_mm/2.,f.id).unwrap();
    }
    out.push_str("</svg>");
    out
}
/// 3D CIRCLE entities: OCS centers and extrusion normals preserve the global work plane.
pub fn dxf(result: &CycleResult) -> String {
    let mut out="0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n".to_string();
    let normal = result.request.global_from_local.vector([0., 0., 1.]);
    let seed = if normal[0].abs() < 1. / 64. && normal[1].abs() < 1. / 64. {
        [0., 1., 0.]
    } else {
        [0., 0., 1.]
    };
    fn cross(a: Vec3, b: Vec3) -> Vec3 {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    }
    let x = cross(seed, normal);
    let n = norm(x);
    let x = x.map(|v| v / n);
    let y = cross(normal, x);
    for f in &result.features {
        let p = f.global_xyz_mm;
        let o = [
            crate::core::dot(p, x),
            crate::core::dot(p, y),
            crate::core::dot(p, normal),
        ];
        write!(out,"0\nCIRCLE\n8\n{}\n10\n{}\n20\n{}\n30\n{}\n40\n{}\n210\n{}\n220\n{}\n230\n{}\n0\nPOINT\n8\n{}_CENTERS\n10\n{}\n20\n{}\n30\n{}\n",f.cycle_id,o[0],o[1],o[2],f.diameter_mm/2.,normal[0],normal[1],normal[2],f.cycle_id,p[0],p[1],p[2]).unwrap();
    }
    out.push_str("0\nENDSEC\n0\nEOF\n");
    out
}
pub fn export(result: &CycleResult, path: &std::path::Path) -> Result<()> {
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    for (name, text) in [
        (
            "cycle.json",
            serde_json::to_string_pretty(result).map_err(|e| e.to_string())?,
        ),
        ("cycle.csv", csv(result)),
        ("cycle.svg", svg(result)),
        ("cycle.dxf", dxf(result)),
    ] {
        std::fs::write(path.join(name), text).map_err(|e| e.to_string())?;
    }
    Ok(())
}
