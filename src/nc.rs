//! Deliberately bounded ISO dialect: incremental IJK centers, explicit WCS transforms.
use crate::{
    Result,
    core::{Transform, Vec3, finite, norm, sub},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::f64::consts::{PI, TAU};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NcConfig {
    pub initial_position_mm: Vec3,
    pub chord_error_mm: f64,
    pub work_offsets: BTreeMap<String, Transform>,
}
impl Default for NcConfig {
    fn default() -> Self {
        Self {
            initial_position_mm: [0.; 3],
            chord_error_mm: 0.02,
            work_offsets: BTreeMap::new(),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Motion {
    pub line: usize,
    pub kind: String,
    pub start_mm: Vec3,
    pub end_mm: Vec3,
    pub center_mm: Option<Vec3>,
    pub normal: Option<Vec3>,
    pub clockwise: Option<bool>,
    pub points_mm: Vec<Vec3>,
    pub feed_mm_min: Option<f64>,
    pub spindle_rpm: Option<f64>,
    pub tool: Option<u32>,
    pub work_system: String,
    pub source_units: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    pub schema_version: u32,
    pub units: String,
    pub chord_error_mm: f64,
    pub initial_position_mm: Vec3,
    pub motions: Vec<Motion>,
    pub limitations: Vec<String>,
}
fn words(line: &str) -> Result<Vec<(char, f64)>> {
    let mut cleaned = String::new();
    let mut comment = false;
    for c in line.chars() {
        match c {
            ';' if !comment => break,
            '(' if !comment => comment = true,
            ')' if comment => comment = false,
            '(' | ')' => return Err("Nieprawidłowy komentarz".into()),
            _ if !comment => cleaned.push(c.to_ascii_uppercase()),
            _ => (),
        }
    }
    if comment {
        return Err("Niedomknięty komentarz".into());
    }
    if cleaned.trim() == "%" {
        return Ok(vec![]);
    }
    let b = cleaned.as_bytes();
    let mut i = 0;
    let mut out = Vec::new();
    while i < b.len() {
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if !b[i].is_ascii_alphabetic() {
            return Err(format!("Nieobsługiwana składnia: {}", &cleaned[i..]));
        }
        let letter = b[i] as char;
        i += 1;
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        let start = i;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'.') {
            i += 1;
        }
        let value = cleaned[start..i]
            .parse::<f64>()
            .map_err(|_| format!("Nieobsługiwane słowo {letter}"))?;
        if !value.is_finite() {
            return Err("Nieskończona liczba".into());
        }
        out.push((letter, value));
    }
    Ok(out)
}
fn integer(x: f64) -> Result<u32> {
    if x < 0. || x > u32::MAX as f64 || x.fract() != 0. {
        Err("Wymagany nieujemny numer całkowity".into())
    } else {
        Ok(x as u32)
    }
}
pub fn parse(text: &str, config: &NcConfig) -> Result<Program> {
    if !finite(config.initial_position_mm)
        || !config.chord_error_mm.is_finite()
        || config.chord_error_mm <= 0.
    {
        return Err("Błędna pozycja początkowa lub tolerancja".into());
    }
    for (key, t) in &config.work_offsets {
        if !["G54", "G55", "G56", "G57", "G58", "G59"].contains(&key.as_str()) {
            return Err(format!("Nieznany układ {key}"));
        }
        t.validate()?;
    }
    let mut pos = config.initial_position_mm;
    let mut transform = Transform::default();
    let mut system = "LOCAL".to_string();
    let mut absolute = true;
    let mut scale = 1.;
    let mut plane = 17;
    let mut mode = None;
    let mut feed = None;
    let mut spindle = None;
    let mut selected_tool = None;
    let mut active_tool = None;
    let mut ended = false;
    let mut motions = Vec::new();
    for (idx, line) in text.lines().enumerate() {
        let result = (|| -> Result<()> {
            let ws = words(line)?;
            if ws.is_empty() {
                return Ok(());
            }
            if ended {
                return Err("Kod po M2/M30".into());
            }
            let mut seen = BTreeSet::new();
            let mut groups = BTreeSet::new();
            let mut values = BTreeMap::new();
            let mut end = false;
            let mut change_tool = false;
            for (k, v) in ws {
                if k != 'G' && k != 'M' && !seen.insert(k) {
                    return Err(format!("Powtórzone słowo {k}"));
                }
                match k {
                    'G' => {
                        let code = integer(v)?;
                        let group = match code {
                            0..=3 => 0,
                            17..=19 => 1,
                            20 | 21 => 2,
                            90 | 91 => 3,
                            54..=59 => 4,
                            94 => 5,
                            _ => {
                                return Err(format!(
                                    "Nieobsługiwane G{v} (cykle/korekcje/transformacje sterownika wymagają osobnego interpretera)"
                                ));
                            }
                        };
                        if !groups.insert(group) {
                            return Err("Sprzeczne kody jednej grupy modalnej".into());
                        }
                        match code {
                            0..=3 => mode = Some(code),
                            17..=19 => plane = code,
                            20 => scale = 25.4,
                            21 => scale = 1.,
                            90 => absolute = true,
                            91 => absolute = false,
                            54..=59 => {
                                system = format!("G{code}");
                                transform = config
                                    .work_offsets
                                    .get(&system)
                                    .ok_or_else(|| format!("Brak jawnej definicji {system}"))?
                                    .clone();
                            }
                            94 => (),
                            _ => unreachable!(),
                        }
                    }
                    'M' => match integer(v)? {
                        2 | 30 => {
                            if end {
                                return Err("Powtórzony koniec".into());
                            }
                            end = true;
                        }
                        6 => {
                            if change_tool {
                                return Err("Powtórzone M6".into());
                            }
                            change_tool = true;
                        }
                        3..=5 | 8 | 9 => (),
                        _ => return Err(format!("Nieobsługiwane M{v}")),
                    },
                    'X' | 'Y' | 'Z' | 'I' | 'J' | 'K' | 'F' | 'S' | 'T' | 'N' => {
                        values.insert(k, v);
                    }
                    _ => return Err(format!("Nieobsługiwane słowo {k}")),
                }
            }
            if let Some(v) = values.get(&'N') {
                integer(*v)?;
            }
            if let Some(v) = values.get(&'T') {
                selected_tool = Some(integer(*v)?);
            }
            if change_tool {
                active_tool = Some(selected_tool.ok_or("M6 bez T")?);
            }
            if let Some(v) = values.get(&'F') {
                if *v <= 0. {
                    return Err("F musi być dodatnie".into());
                }
                feed = Some(*v);
            }
            if let Some(v) = values.get(&'S') {
                if *v < 0. {
                    return Err("S nie może być ujemne".into());
                }
                spindle = Some(*v);
            }
            let has_xyz = ['X', 'Y', 'Z'].iter().any(|c| values.contains_key(c));
            let has_ijk = ['I', 'J', 'K'].iter().any(|c| values.contains_key(c));
            if has_xyz || has_ijk {
                if change_tool || end {
                    return Err("Ruch z M6/M2/M30 w jednym bloku nieobsługiwany".into());
                }
                let m = mode.ok_or("Współrzędne bez G0/G1/G2/G3")?;
                if m != 0 && feed.is_none() {
                    return Err("Ruch roboczy bez F".into());
                }
                let local = transform.inverse_point(pos);
                let mut target = local;
                for (i, c) in ['X', 'Y', 'Z'].into_iter().enumerate() {
                    if let Some(v) = values.get(&c) {
                        target[i] = if absolute {
                            v * scale
                        } else {
                            local[i] + v * scale
                        };
                    }
                }
                let end_pos = transform.point(target);
                if !finite(end_pos) {
                    return Err("Przepełnienie współrzędnych".into());
                }
                let mut motion = Motion {
                    line: idx + 1,
                    kind: if m == 0 {
                        "rapid"
                    } else if m == 1 {
                        "feed_line"
                    } else {
                        "arc"
                    }
                    .into(),
                    start_mm: pos,
                    end_mm: end_pos,
                    center_mm: None,
                    normal: None,
                    clockwise: None,
                    points_mm: vec![pos, end_pos],
                    feed_mm_min: if m == 0 {
                        None
                    } else {
                        feed.map(|v| v * scale)
                    },
                    spindle_rpm: spindle,
                    tool: active_tool,
                    work_system: system.clone(),
                    source_units: if scale == 1. { "mm" } else { "inch" }.into(),
                };
                if m <= 1 && has_ijk {
                    return Err("IJK przy ruchu liniowym".into());
                }
                if m >= 2 {
                    // Right-handed planes XY, ZX, YZ; clockwise as viewed from +normal.
                    let (u, v, w) = match plane {
                        17 => (0, 1, 2),
                        18 => (2, 0, 1),
                        _ => (1, 2, 0),
                    };
                    let letters = ['I', 'J', 'K'];
                    if !values.contains_key(&letters[u]) && !values.contains_key(&letters[v]) {
                        return Err("Łuk wymaga środka IJK (przyrostowo)".into());
                    }
                    if values.contains_key(&letters[w]) {
                        return Err("Składowa środka poza płaszczyzną łuku".into());
                    }
                    let mut center = local;
                    center[u] += values.get(&letters[u]).unwrap_or(&0.) * scale;
                    center[v] += values.get(&letters[v]).unwrap_or(&0.) * scale;
                    let a = sub(local, center);
                    let b = sub(target, center);
                    let radius = a[u].hypot(a[v]);
                    let r2 = b[u].hypot(b[v]);
                    if !radius.is_finite()
                        || radius <= 1e-9
                        || (radius - r2).abs() > 1e-7_f64.max(radius * 1e-9)
                    {
                        return Err("Niespójne promienie łuku".into());
                    }
                    let start = a[v].atan2(a[u]);
                    let finish = b[v].atan2(b[u]);
                    let mut sweep = if m == 3 {
                        (finish - start).rem_euclid(TAU)
                    } else {
                        -(start - finish).rem_euclid(TAU)
                    };
                    if sweep.abs() < 1e-12 {
                        sweep = if m == 3 { TAU } else { -TAU };
                    }
                    let max_step =
                        (2. * (1. - (config.chord_error_mm / radius).min(1.)).acos()).min(PI / 2.);
                    let n = (sweep.abs() / max_step).ceil();
                    if !n.is_finite() || n > 100_000. {
                        return Err("Za dużo segmentów łuku dla tolerancji".into());
                    }
                    let n = (n as usize).max(1);
                    motion.points_mm = (0..=n)
                        .map(|i| {
                            let t = i as f64 / n as f64;
                            let ang = start + sweep * t;
                            let mut p = center;
                            p[u] += radius * ang.cos();
                            p[v] += radius * ang.sin();
                            p[w] = local[w] + (target[w] - local[w]) * t;
                            transform.point(p)
                        })
                        .collect();
                    motion.points_mm[0] = pos;
                    motion.points_mm[n] = end_pos;
                    motion.center_mm = Some(transform.point(center));
                    let mut normal = [0.; 3];
                    normal[w] = 1.;
                    motion.normal = Some(transform.vector(normal));
                    motion.clockwise = Some(m == 2);
                }
                if motions.len() >= 100_000 {
                    return Err("Limit 100000 ruchów".into());
                }
                motions.push(motion);
                pos = end_pos;
            }
            ended = end;
            Ok(())
        })();
        result.map_err(|e| format!("Wiersz {}: {e}. Podgląd zablokowany.", idx + 1))?;
    }
    Ok(Program{schema_version:1,units:"mm".into(),chord_error_mm:config.chord_error_mm,initial_position_mm:config.initial_position_mm,motions,
        limitations:vec!["Podgląd geometryczny; bez certyfikacji kinematyki, korekcji i kolizji".into(),
            "M3/M4/M5/M8/M9 przyjęte jako metadane; brak modelu wrzeciona/chłodziwa".into(),
            "IJK przyrostowo, F w mm/min po konwersji; pozycja początkowa jawna w konfiguracji (domyślnie 0,0,0)".into()]})
}
pub fn length(m: &Motion) -> f64 {
    m.points_mm.windows(2).map(|p| norm(sub(p[1], p[0]))).sum()
}
/// Convenience API for an explicitly defined stock-to-machine placement.
pub fn translate_program(p: &mut Program, t: &Transform) -> Result<()> {
    t.validate()?;
    p.initial_position_mm = t.point(p.initial_position_mm);
    for m in &mut p.motions {
        m.start_mm = t.point(m.start_mm);
        m.end_mm = t.point(m.end_mm);
        m.center_mm = m.center_mm.map(|p| t.point(p));
        m.normal = m.normal.map(|n| t.vector(n));
        m.points_mm.iter_mut().for_each(|p| *p = t.point(*p));
    }
    Ok(())
}
