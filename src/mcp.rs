//! Minimal local stdio tools server; no filesystem, network, model mutation or machine commands.
use crate::{
    Result,
    core::{Face, Transform, Vec3, finite, norm, sub},
    nc, showcase,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{BufRead, Read, Write};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InspectArgs {
    face: Option<Face>,
    diameter_mm: Option<f64>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExplainArgs {
    nc: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TransformArgs {
    transform: Transform,
    point_mm: Vec3,
}
#[derive(Serialize)]
struct Inspection {
    schema_version: u32,
    units: &'static str,
    count: usize,
    features: Vec<showcase::Feature>,
    synthetic_only: bool,
}
#[derive(Serialize)]
struct Explanation {
    schema_version: u32,
    supported: bool,
    motion_count: usize,
    summary: String,
    diagnostic: Option<String>,
    machining_verified: bool,
}
#[derive(Serialize)]
struct Validation {
    schema_version: u32,
    valid: bool,
    transformed_point_mm: Option<Vec3>,
    round_trip_error_mm: Option<f64>,
    diagnostic: Option<String>,
}
fn decode<T: serde::de::DeserializeOwned>(v: Value) -> Result<T> {
    serde_json::from_value(v).map_err(|e| e.to_string())
}
fn encode<T: Serialize>(v: T) -> Result<Value> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}
pub fn call(name: &str, args: Value) -> Result<Value> {
    match name {
        "inspect_features" => {
            let args: InspectArgs = decode(args)?;
            if args.diameter_mm.is_some_and(|d| !d.is_finite() || d <= 0.) {
                return Err("diameter_mm must be finite and positive".into());
            }
            let features = showcase::demo()?
                .features
                .into_iter()
                .filter(|f| {
                    args.face.is_none_or(|face| f.face == face)
                        && args.diameter_mm.is_none_or(|d| f.diameter_mm == Some(d))
                })
                .collect::<Vec<_>>();
            encode(Inspection {
                schema_version: 1,
                units: "mm",
                count: features.len(),
                features,
                synthetic_only: true,
            })
        }
        "explain_nc_cycle" => {
            let args: ExplainArgs = decode(args)?;
            if args.nc.len() > 4096 || args.nc.lines().count() > 64 {
                return Err("NC input limit: 4096 bytes / 64 lines".into());
            }
            match nc::parse(&args.nc,&nc::NcConfig::default()){Ok(p)=>encode(Explanation{schema_version:1,supported:true,motion_count:p.motions.len(),summary:"Accepted by the bounded geometric parser; G0 rapid, G1 linear feed, G2/G3 incremental-IJK arcs. No canned-cycle expansion or machining verification.".into(),diagnostic:None,machining_verified:false}),Err(e)=>encode(Explanation{schema_version:1,supported:false,motion_count:0,summary:"Program rejected in full. Canned cycles, controller transforms, compensation and macros are unsupported; no motion is inferred.".into(),diagnostic:Some(e),machining_verified:false})}
        }
        "validate_coordinate_transform" => {
            let args: TransformArgs = decode(args)?;
            let result = args.transform.validate().and_then(|()| {
                if !finite(args.point_mm) {
                    return Err("Point must be finite".into());
                }
                let p = args.transform.point(args.point_mm);
                let error = norm(sub(args.transform.inverse_point(p), args.point_mm));
                if !finite(p) || !error.is_finite() {
                    return Err("Coordinate overflow".into());
                }
                Ok((p, error))
            });
            match result {
                Ok((point, error)) => encode(Validation {
                    schema_version: 1,
                    valid: true,
                    transformed_point_mm: Some(point),
                    round_trip_error_mm: Some(error),
                    diagnostic: None,
                }),
                Err(e) => encode(Validation {
                    schema_version: 1,
                    valid: false,
                    transformed_point_mm: None,
                    round_trip_error_mm: None,
                    diagnostic: Some(e),
                }),
            }
        }
        _ => Err("Unknown read-only tool".into()),
    }
}
pub fn tools() -> Value {
    let vec3 = json!({"type":"array","items":{"type":"number"},"minItems":3,"maxItems":3});
    let nullable_vec = json!({"anyOf":[vec3,{"type":"null"}]});
    let nullable_string = json!({"type":["string","null"]});
    let face = json!({"type":"string","enum":["X_MIN","X_MAX","Y_MIN","Y_MAX","Z_MIN","Z_MAX"]});
    let feature = json!({"type":"object","additionalProperties":false,"required":["id","kind","face","entry_mm","axis_into_stock","diameter_mm","depth_mm","tool","source","outline_mm"],"properties":{"id":{"type":"string"},"kind":{"enum":["hole","slot","pocket"]},"face":face,"entry_mm":vec3,"axis_into_stock":vec3,"diameter_mm":{"type":["number","null"]},"depth_mm":{"type":"number"},"tool":{"type":"string"},"source":{"type":"string"},"outline_mm":{"type":"array","items":vec3}}});
    let definitions = [
        (
            "inspect_features",
            "Inspect only the built-in synthetic Aurora features; filters never open files.",
            json!({"type":"object","additionalProperties":false,"properties":{"face":face,"diameter_mm":{"type":"number","exclusiveMinimum":0}}}),
            json!({"type":"object","additionalProperties":false,"required":["schema_version","units","count","features","synthetic_only"],"properties":{"schema_version":{"const":1},"units":{"const":"mm"},"count":{"type":"integer"},"features":{"type":"array","items":feature},"synthetic_only":{"const":true}}}),
        ),
        (
            "explain_nc_cycle",
            "Explain whether a short ISO-NC snippet is supported. Unsupported cycles are rejected, never expanded.",
            json!({"type":"object","additionalProperties":false,"required":["nc"],"properties":{"nc":{"type":"string","maxLength":4096}}}),
            json!({"type":"object","additionalProperties":false,"required":["schema_version","supported","motion_count","summary","diagnostic","machining_verified"],"properties":{"schema_version":{"const":1},"supported":{"type":"boolean"},"motion_count":{"type":"integer"},"summary":{"type":"string"},"diagnostic":nullable_string,"machining_verified":{"const":false}}}),
        ),
        (
            "validate_coordinate_transform",
            "Check a right-handed rigid transform; apply rotation then translation to a point in mm. No machine kinematics.",
            json!({"type":"object","additionalProperties":false,"required":["transform","point_mm"],"properties":{"point_mm":vec3,"transform":{"type":"object","additionalProperties":false,"required":["rotation","translation_mm"],"properties":{"rotation":{"type":"array","items":vec3,"minItems":3,"maxItems":3},"translation_mm":vec3}}}}),
            json!({"type":"object","additionalProperties":false,"required":["schema_version","valid","transformed_point_mm","round_trip_error_mm","diagnostic"],"properties":{"schema_version":{"const":1},"valid":{"type":"boolean"},"transformed_point_mm":nullable_vec,"round_trip_error_mm":{"type":["number","null"]},"diagnostic":nullable_string}}),
        ),
    ];
    json!({"tools":definitions.into_iter().map(|(name,description,input,output)|json!({"name":name,"description":description,"inputSchema":input,"outputSchema":output,"annotations":{"readOnlyHint":true,"destructiveHint":false,"idempotentHint":true,"openWorldHint":false}})).collect::<Vec<_>>()})
}
#[derive(Default)]
pub struct Server {
    initialized: bool,
    ready: bool,
}
impl Server {
    pub fn respond(&mut self, v: Value) -> Option<Value> {
        let id = v.get("id").cloned().unwrap_or(Value::Null);
        let error = |code, message| json!({"jsonrpc":"2.0","id":id,"error":{"code":code,"message":message}});
        if v["jsonrpc"] != "2.0"
            || !v["method"].is_string()
            || (!id.is_null() && !id.is_string() && !id.is_i64())
        {
            return Some(error(-32600, "Invalid request"));
        }
        let method = v["method"].as_str().unwrap();
        if v.get("id").is_none() {
            if method == "notifications/initialized" && self.initialized {
                self.ready = true;
            }
            return None;
        }
        let result = match method {
            "initialize" => {
                if self.initialized {
                    return Some(error(-32600, "Already initialized"));
                }
                if !v["params"]["protocolVersion"].is_string()
                    || !v["params"]["capabilities"].is_object()
                    || !v["params"]["clientInfo"].is_object()
                {
                    return Some(error(-32602, "Invalid initialize parameters"));
                }
                self.initialized = true;
                json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{"listChanged":false}},"serverInfo":{"name":"pixcam-showcase","version":"0.1.0"}})
            }
            "ping" => json!({}),
            _ if !self.ready => return Some(error(-32000, "Initialize the session first")),
            "tools/list" => tools(),
            "tools/call" => {
                let Some(name) = v["params"]["name"].as_str() else {
                    return Some(error(-32602, "Missing tool name"));
                };
                if ![
                    "inspect_features",
                    "explain_nc_cycle",
                    "validate_coordinate_transform",
                ]
                .contains(&name)
                {
                    return Some(error(-32602, "Unknown tool"));
                }
                match call(
                    name,
                    v["params"].get("arguments").cloned().unwrap_or(json!({})),
                ) {
                    Ok(value) => {
                        json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":false})
                    }
                    Err(e) => json!({"content":[{"type":"text","text":e}],"isError":true}),
                }
            }
            _ => return Some(error(-32601, "Method not found")),
        };
        Some(json!({"jsonrpc":"2.0","id":id,"result":result}))
    }
}
pub fn serve() -> Result<()> {
    let mut server = Server::default();
    let mut input = std::io::stdin().lock();
    let mut output = std::io::stdout().lock();
    loop {
        let mut line = Vec::new();
        let bytes = Read::by_ref(&mut input)
            .take(65537)
            .read_until(b'\n', &mut line)
            .map_err(|e| e.to_string())?;
        if bytes == 0 {
            break;
        }
        if bytes > 65536 {
            return Err("MCP message exceeds 64 KiB".into());
        }
        let response = match serde_json::from_slice(&line) {
            Ok(v) => server.respond(v),
            Err(_) => Some(
                json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Parse error"}}),
            ),
        };
        if let Some(response) = response {
            writeln!(output, "{response}").map_err(|e| e.to_string())?;
            output.flush().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
