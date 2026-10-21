use pixcam_showcase::{Result, showcase};
fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
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
        _ => Err("Usage: pixcam-showcase build [output-directory] | mcp | tool-schemas | import-solidcraft-snapshot SNAPSHOT.json".into()),
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
