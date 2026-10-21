use pixcam_showcase::{Result, showcase};
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("capture-solidcraft") => {
            let config = pixcam_showcase::bridge::config(&args[1..])?;
            println!("{}", serde_json::to_string_pretty(&pixcam_showcase::bridge::Client::capture(&config)?).map_err(|_| "Snapshot serialization failed")?);
            Ok(())
        }
        Some("analyze-solidcraft") if args.len() == 2 => {
            let text = std::fs::read_to_string(&args[1]).map_err(|e| e.to_string())?;
            let report = pixcam_showcase::recognition::analyze(&text)?;
            println!("{}", serde_json::to_string_pretty(&report).map_err(|e|e.to_string())?);
            Ok(())
        }
        Some("bridge") => pixcam_showcase::local_server::serve(&args[1..]),
        Some("benchmark-solidcraft") if args.len()==2 => {
            let text=std::fs::read_to_string(&args[1]).map_err(|e|e.to_string())?;
            for _ in 0..20 {std::hint::black_box(pixcam_showcase::recognition::analyze(&text)?);}
            let mut samples=Vec::new();
            for _ in 0..1000 {
                let start=std::time::Instant::now();
                std::hint::black_box(pixcam_showcase::recognition::analyze(&text)?);
                samples.push(start.elapsed().as_secs_f64()*1000.);
            }
            samples.sort_by(f64::total_cmp);
            println!("{}",serde_json::json!({"runs":1000,"warmup":20,"scope":"JSON parse + validation + recognition + report construction; no process startup or CAD evaluation","input_bytes":text.len(),"median_ms":samples[499],"p95_ms":samples[949],"min_ms":samples[0],"max_ms":samples[999]}));Ok(())
        }
        Some("mcp") if args.len() == 1 => pixcam_showcase::mcp::serve(),
        Some("tool-schemas") if args.len() == 1 => {
            println!("{}", pixcam_showcase::mcp::tools());
            Ok(())
        }
        Some("build") if args.len() <= 2 => {
            let path = args.get(1).map(String::as_str).unwrap_or("dist");
            showcase::build(std::path::Path::new(path))?;
            println!("Open {path}/index.html in your browser. Synthetic data only.");
            Ok(())
        }
        Some("import-solidcraft-snapshot") if args.len() == 2 => {
            let text = std::fs::read_to_string(&args[1]).map_err(|e| e.to_string())?;
            let imported = pixcam_showcase::solidcraft::import_snapshot(&text)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&imported).map_err(|e| e.to_string())?
            );
            Ok(())
        }
        _ => Err("Usage: pixcam-showcase build [DIR] | bridge (--in FILE | --connect 127.0.0.1:PORT) [--timeout-ms N] | capture-solidcraft [--in FILE | --connect 127.0.0.1:PORT] [--timeout-ms N] | analyze-solidcraft SNAPSHOT.json | benchmark-solidcraft SNAPSHOT.json | mcp | tool-schemas | import-solidcraft-snapshot SNAPSHOT.json".into()),
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
