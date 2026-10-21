//! One registry envelope preserves imported CAD evidence and generated cycle provenance.
use crate::{
    Result,
    cycles::{self, CycleResult},
    recognition::Report,
};
use serde::Serialize;
#[derive(Serialize)]
pub struct FeatureRegistry {
    pub schema_version: u32,
    pub units: &'static str,
    pub imported_cad: Option<Report>,
    pub parametric_cycles: Vec<CycleResult>,
    pub feature_count: usize,
    pub production_nc_generated: bool,
}
pub fn combine(
    imported_cad: Option<Report>,
    parametric_cycles: Vec<CycleResult>,
) -> Result<FeatureRegistry> {
    let mut ids = std::collections::BTreeSet::new();
    for cycle in &parametric_cycles {
        if !ids.insert(&cycle.request.cycle_id) {
            return Err("Duplicate cycle ID in registry".into());
        }
        let regenerated = cycles::generate(&cycle.request)?;
        if serde_json::to_value(regenerated).map_err(|e| e.to_string())?
            != serde_json::to_value(cycle).map_err(|e| e.to_string())?
        {
            return Err("Cycle registry disagrees with source parameters".into());
        }
    }
    let feature_count = imported_cad.as_ref().map_or(0, |r| r.features.len())
        + parametric_cycles
            .iter()
            .map(|c| c.features.len())
            .sum::<usize>();
    Ok(FeatureRegistry {
        schema_version: 1,
        units: "mm",
        imported_cad,
        parametric_cycles,
        feature_count,
        production_nc_generated: false,
    })
}
