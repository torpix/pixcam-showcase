use crate::{
    Result,
    core::{Face, Stock, Vec3},
    nc,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Write as _};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Hole,
    Slot,
    Pocket,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: String,
    pub kind: Kind,
    pub face: Face,
    pub entry_mm: Vec3,
    pub axis_into_stock: Vec3,
    pub diameter_mm: Option<f64>,
    pub depth_mm: f64,
    pub tool: String,
    pub source: String,
    pub outline_mm: Vec<Vec3>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Demo {
    pub schema_version: u32,
    pub part: String,
    pub units: String,
    pub stock: Stock,
    pub features: Vec<Feature>,
    pub groups: BTreeMap<String, Vec<String>>,
    pub program: nc::Program,
    pub nc_text: String,
    pub limitations: Vec<String>,
}
pub fn demo() -> Result<Demo> {
    let stock = Stock {
        min_mm: [0.; 3],
        max_mm: [120., 76., 20.],
    };
    let mut features = Vec::new();
    for (id, face, uv, diameter, depth) in [
        ("A01", Face::ZMax, [16., 14.], 6., 8.),
        ("A02", Face::ZMax, [43., 17.], 6., 8.),
        ("A03", Face::ZMax, [78., 13.], 6., 8.),
        ("A04", Face::ZMax, [96., 49.], 10., 10.),
        ("B01", Face::YMin, [24., 10.], 6., 12.),
        ("B02", Face::YMin, [89., 10.], 6., 12.),
        ("C01", Face::XMax, [38., 10.], 10., 14.),
    ] {
        let mut outline_mm: Vec<Vec3> = (0..=96)
            .map(|i| {
                let a = i as f64 / 96. * std::f64::consts::TAU;
                face.point(
                    [
                        uv[0] + diameter / 2. * a.cos(),
                        uv[1] + diameter / 2. * a.sin(),
                    ],
                    &stock,
                )
            })
            .collect();
        outline_mm[96] = outline_mm[0];
        features.push(Feature {
            id: id.into(),
            kind: Kind::Hole,
            face,
            entry_mm: face.point(uv, &stock),
            axis_into_stock: face.normal(),
            diameter_mm: Some(diameter),
            depth_mm: depth,
            tool: if diameter == 6. {
                "T1 · drill Ø6"
            } else {
                "T2 · drill Ø10"
            }
            .into(),
            source: "Original synthetic definition · src/showcase.rs · millimetres".into(),
            outline_mm,
        });
    }
    let mut slot = Vec::new();
    for (cx, start) in [
        (44., -std::f64::consts::FRAC_PI_2),
        (20., std::f64::consts::FRAC_PI_2),
    ] {
        for i in 0..=48 {
            let a = start + i as f64 / 48. * std::f64::consts::PI;
            slot.push([cx + 7. * a.cos(), 54. + 7. * a.sin(), 20.]);
        }
    }
    slot.push(slot[0]);
    for (id, kind, entry, outline, depth) in [
        ("S01", Kind::Slot, [32., 54., 20.], slot, 6.),
        (
            "P01",
            Kind::Pocket,
            [73., 46., 20.],
            vec![
                [58., 33., 20.],
                [88., 33., 20.],
                [88., 59., 20.],
                [58., 59., 20.],
                [58., 33., 20.],
            ],
            7.,
        ),
    ] {
        features.push(Feature {
            id: id.into(),
            kind,
            face: Face::ZMax,
            entry_mm: entry,
            axis_into_stock: [0., 0., -1.],
            diameter_mm: None,
            depth_mm: depth,
            tool: "T3 · end mill Ø6 (illustrative)".into(),
            source: "Original synthetic definition · src/showcase.rs · millimetres".into(),
            outline_mm: outline,
        });
    }
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for f in &features {
        groups
            .entry(format!("{} / {}", f.face.name(), f.tool))
            .or_default()
            .push(f.id.clone());
    }
    let nc_text = include_str!("../fixtures/aurora.nc").to_string();
    let mut program = nc::parse(&nc_text, &nc::NcConfig::default())?;
    let limitations=vec![
        "Geometric preview only. No verified machining simulation, material removal, collision, gouge, holder or machine model.".into(),
        "Synthetic annotations, not automatic CAD feature recognition. Slot: 38 × 14 × 6 mm; pocket: 30 × 26 × 7 mm with schematic square corners.".into(),
        "NC is hand-authored illustration of top-face moves, not generated from the feature map. Side holes and complete pocket clearing are not programmed.".into(),
        "Bounded parser: G0–G3, G17–G19, G20/G21, G90/G91, G94, incremental IJK. Explicit transforms required for G54–G59. Defaults: mm, absolute, XY, origin [0,0,0].".into(),
        "Unsupported input rejects the entire program: canned cycles, CYCLE800, rotary axes, R arcs, compensation, macros and subprograms. This demo does not implement them.".into(),
        "Arc chord tolerance: 0.02 mm. Playback follows polyline distance, not feed or real machining time. Tool choices are illustrative, never production approval.".into(),
    ];
    program.limitations = limitations.clone();
    Ok(Demo {
        schema_version: 1,
        part: "Aurora / synthetic inspection plate".into(),
        units: "mm".into(),
        stock,
        features,
        groups,
        program,
        nc_text,
        limitations,
    })
}
fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
pub fn color(f: &Feature) -> &'static str {
    match f.kind {
        Kind::Slot => "#a99aff",
        Kind::Pocket => "#f0ba70",
        Kind::Hole => {
            if f.diameter_mm == Some(6.) {
                "#56ded0"
            } else {
                "#69adff"
            }
        }
    }
}
pub fn svg(d: &Demo) -> String {
    let mut s = String::from(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 440 180\" role=\"img\"><title>Aurora synthetic feature map, dimensions in mm</title><rect width=\"440\" height=\"180\" fill=\"#101c2a\"/>",
    );
    for (index, face) in [Face::ZMax, Face::YMin, Face::XMax].iter().enumerate() {
        let x = 12. + index as f64 * 146.;
        let [w, h] = face.size(&d.stock);
        write!(s,"<g transform=\"translate({x} 30)\"><text fill=\"#dce7ee\" font-size=\"7\">{} · {} / {} mm</text><g transform=\"translate(0 100) scale(1 -1)\"><rect width=\"{w}\" height=\"{h}\" fill=\"#1b2c3d\" stroke=\"#697e91\" stroke-width=\"0.5\"/>",face.name(),["X","Y","Z"][face.uv_axes().0],["X","Y","Z"][face.uv_axes().1]).unwrap();
        for f in d.features.iter().filter(|f| f.face == *face) {
            let points = f
                .outline_mm
                .iter()
                .map(|p| {
                    let uv = face.uv(*p, &d.stock);
                    format!("{},{}", uv[0], uv[1])
                })
                .collect::<Vec<_>>()
                .join(" ");
            let uv = face.uv(f.entry_mm, &d.stock);
            write!(s,"<polyline data-id=\"{}\" points=\"{points}\" fill=\"none\" stroke=\"{}\" stroke-width=\"0.8\"><title>{}</title></polyline><text transform=\"translate({} {}) scale(1 -1)\" fill=\"#dce7ee\" font-size=\"4\">{}</text>",xml(&f.id),color(f),xml(&f.source),uv[0]+4.,uv[1]+5.,xml(&f.id)).unwrap();
        }
        s.push_str("</g></g>");
    }
    s.push_str("<text x=\"12\" y=\"165\" fill=\"#aabbcc\" font-size=\"6\">Nominal outlines · three unmirrored face projections · synthetic data only · not a machining plan</text></svg>");
    s
}
pub fn csv(d: &Demo) -> String {
    let mut s = String::from(
        "id,kind,face,x_mm,y_mm,z_mm,axis_x,axis_y,axis_z,diameter_mm,depth_mm,tool,source\r\n",
    );
    for f in &d.features {
        let row = vec![
            f.id.clone(),
            serde_json::to_value(&f.kind)
                .unwrap()
                .as_str()
                .unwrap()
                .into(),
            f.face.name().into(),
            f.entry_mm[0].to_string(),
            f.entry_mm[1].to_string(),
            f.entry_mm[2].to_string(),
            f.axis_into_stock[0].to_string(),
            f.axis_into_stock[1].to_string(),
            f.axis_into_stock[2].to_string(),
            f.diameter_mm.map(|x| x.to_string()).unwrap_or_default(),
            f.depth_mm.to_string(),
            f.tool.clone(),
            f.source.clone(),
        ];
        s.push_str(
            &row.iter()
                .map(|v| format!("\"{}\"", v.replace('"', "\"\"")))
                .collect::<Vec<_>>()
                .join(","),
        );
        s.push_str("\r\n");
    }
    s
}
pub fn dxf(d: &Demo) -> String {
    let mut s = String::from(
        "0\nSECTION\n2\nHEADER\n9\n$ACADVER\n1\nAC1015\n9\n$INSUNITS\n70\n4\n0\nENDSEC\n0\nSECTION\n2\nENTITIES\n",
    );
    for f in &d.features {
        let uv = f.face.uv(f.entry_mm, &d.stock);
        let offset = match f.face {
            Face::ZMax => 0.,
            Face::YMin => 146.,
            _ => 292.,
        };
        let layer = format!("{}_{}", f.face.name(), f.id);
        if let Some(diameter) = f.diameter_mm {
            write!(
                s,
                "0\nCIRCLE\n8\n{layer}\n10\n{}\n20\n{}\n30\n0\n40\n{}\n",
                uv[0] + offset,
                uv[1],
                diameter / 2.
            )
            .unwrap();
        } else {
            for pair in f.outline_mm.windows(2) {
                let a = f.face.uv(pair[0], &d.stock);
                let b = f.face.uv(pair[1], &d.stock);
                write!(
                    s,
                    "0\nLINE\n8\n{layer}\n10\n{}\n20\n{}\n30\n0\n11\n{}\n21\n{}\n31\n0\n",
                    a[0] + offset,
                    a[1],
                    b[0] + offset,
                    b[1]
                )
                .unwrap();
            }
        }
        write!(
            s,
            "0\nTEXT\n8\n{layer}\n10\n{}\n20\n{}\n30\n0\n40\n2\n1\n{}\n",
            uv[0] + offset + 5.,
            uv[1] + 5.,
            f.id
        )
        .unwrap();
    }
    s.push_str("0\nENDSEC\n0\nEOF\n");
    s
}
pub fn build(path: &std::path::Path) -> Result<()> {
    let d = demo()?;
    let json = serde_json::to_string_pretty(&d).map_err(|e| e.to_string())?;
    let embedded = json
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026");
    let exports = serde_json::json!({"features.json":json,"features.csv":csv(&d),"feature-map.svg":svg(&d),"feature-map.dxf":dxf(&d)}).to_string()
        .replace('<', "\\u003c").replace('>', "\\u003e").replace('&', "\\u0026");
    let html = include_str!("../assets/index.html")
        .replace("__DATA__", &embedded)
        .replace("__EXPORTS__", &exports);
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    for (name, content) in [
        ("index.html", html),
        ("features.json", json),
        ("features.csv", csv(&d)),
        ("feature-map.svg", svg(&d)),
        ("feature-map.dxf", dxf(&d)),
        ("aurora.nc", d.nc_text),
    ] {
        std::fs::write(path.join(name), content).map_err(|e| e.to_string())?;
    }
    Ok(())
}
