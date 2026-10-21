//! Read-only adapter boundary for documented SolidCraft v0.17 inspection results.
//!
//! The adapter consumes an already captured `inspect_design` result plus zero or
//! more `body_topology` results. It never starts SolidCraft, sends engine
//! commands or mutates a CAD document. A live-session transport belongs outside
//! this crate and must call only SolidCraft tools annotated as read-only.
use crate::{
    Result,
    core::{Vec3, finite, norm},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct ReadOnlySnapshot {
    pub schema_version: u32,
    pub solidcraft_version: String,
    pub inspect_design: InspectDesign,
    #[serde(default)]
    pub body_topology: Vec<BodyTopology>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct InspectDesign {
    #[serde(default)]
    pub timeline: Vec<TimelineFeature>,
    #[serde(default)]
    pub bodies: Vec<Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TimelineFeature {
    pub id: Value,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub suppressed: bool,
    #[serde(default)]
    pub error: Option<Value>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BodyTopology {
    pub body: String,
    #[serde(default)]
    pub faces: Vec<TopologyFace>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TopologyFace {
    pub index: u32,
    pub name: String,
    pub area: f64,
    pub centroid: Vec3,
    pub normal: Option<Vec3>,
    pub point: Vec3,
    pub cylinder: Option<Cylinder>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Cylinder {
    pub origin: Vec3,
    pub axis: Vec3,
    pub radius: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ImportedCylinder {
    pub body: String,
    pub face_index: u32,
    pub persistent_face_name: String,
    pub point_mm: Vec3,
    pub axis: Vec3,
    pub diameter_mm: f64,
    pub source: &'static str,
    pub timeline_feature_id: Option<Value>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ImportedFeatureMap {
    pub schema_version: u32,
    pub source_product: String,
    pub read_only: bool,
    pub timeline: Vec<TimelineFeature>,
    pub cylindrical_faces: Vec<ImportedCylinder>,
    pub per_feature_geometry_available: bool,
    pub limitation: String,
}

pub fn import_snapshot(text: &str) -> Result<ImportedFeatureMap> {
    let snapshot: ReadOnlySnapshot = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if snapshot.schema_version != 1 {
        return Err("Unsupported pixCAM SolidCraft snapshot schema".into());
    }
    if !snapshot.solidcraft_version.starts_with("0.17.") {
        return Err(
            "This adapter is pinned to documented SolidCraft 0.17.x inspection output".into(),
        );
    }
    let mut cylindrical_faces = Vec::new();
    for topology in snapshot.body_topology {
        if topology.body.trim().is_empty() {
            return Err("Body name must not be empty".into());
        }
        for face in topology.faces {
            if !face.area.is_finite()
                || face.area < 0.
                || !finite(face.centroid)
                || !finite(face.point)
                || face.normal.is_some_and(|normal| !finite(normal))
            {
                return Err("Non-finite or negative topology geometry".into());
            }
            if let Some(cylinder) = face.cylinder {
                if !finite(cylinder.origin)
                    || !finite(cylinder.axis)
                    || !cylinder.radius.is_finite()
                    || cylinder.radius <= 0.
                    || (norm(cylinder.axis) - 1.).abs() > 1e-8
                {
                    return Err("Invalid cylindrical face geometry".into());
                }
                cylindrical_faces.push(ImportedCylinder {
                    body: topology.body.clone(),
                    face_index: face.index,
                    persistent_face_name: face.name,
                    point_mm: cylinder.origin,
                    axis: cylinder.axis,
                    diameter_mm: cylinder.radius * 2.,
                    source: "SolidCraft body_topology read-only result",
                    timeline_feature_id: None,
                });
            }
        }
    }
    Ok(ImportedFeatureMap {
        schema_version: 1,
        source_product: format!("SolidCraft {}", snapshot.solidcraft_version),
        read_only: true,
        timeline: snapshot.inspect_design.timeline,
        cylindrical_faces,
        per_feature_geometry_available: false,
        limitation: "SolidCraft 0.17 exposes timeline definitions and body topology separately; this adapter does not invent a feature-to-evaluated-face association.".into(),
    })
}
