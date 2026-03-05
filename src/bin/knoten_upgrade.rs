use std::env;
use std::fs;
use std::path::Path;

use knoten_core::ast::Node;
use knoten_core::dsl_emitter::emit_dsl;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: knoten_upgrade <file.nod|file.json>");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let path = Path::new(input_path);
    if !path.exists() {
        eprintln!("Error: File not found: {}", input_path);
        std::process::exit(1);
    }

    let content = fs::read_to_string(path).expect("Failed to read file");

    // Parse old JSON AST
    let node: Node = serde_json::from_str(&content)
        .expect("Failed to parse JSON AST. Is this a valid KnotenCore file?");

    // Transpile to DSL
    let dsl = format!("// Auto-Upgraded to Knoten-DSL\n{}", emit_dsl(&node, 0));

    let output_path = path.with_extension("knoten");
    fs::write(&output_path, dsl).expect("Failed to write upgraded file");

    println!(
        "Successfully upgraded {} -> {}",
        input_path,
        output_path.display()
    );
}
