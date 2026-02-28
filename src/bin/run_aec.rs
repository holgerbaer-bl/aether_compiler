use aether_compiler::executor::ExecutionEngine;
use std::env;
use std::fs;

fn main() {
    let mut engine = ExecutionEngine::new();

    // Check if we are bundled (Sprint 11)
    if let Some(bundled_json) = option_env!("AETHER_BUNDLE") {
        println!("Running embedded AetherCore bundle...");
        let ast = serde_json::from_str(bundled_json)
            .expect("Failed to parse bundled AetherCore JSON AST");
        let result = engine.execute(&ast);
        println!("\nExecution Finished.\nResult: {}", result);
        return;
    }

    // Normal JIT mode
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: run_aec <path_to.json>");
        std::process::exit(1);
    }

    let file_path = &args[1];
    println!("CWD: {:?}", env::current_dir().unwrap());
    println!("Loading AetherCore Script: {}", file_path);

    let json_string = fs::read_to_string(file_path).expect("Failed to read file");
    let ast = serde_json::from_str(&json_string).expect("Failed to parse AetherCore JSON AST");

    let result = engine.execute(&ast);

    println!("\nExecution Finished.\nResult: {}", result);
}
