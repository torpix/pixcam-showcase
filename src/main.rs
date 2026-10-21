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
        _ => Err("Usage: pixcam-showcase build [output-directory] | mcp | tool-schemas".into()),
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
