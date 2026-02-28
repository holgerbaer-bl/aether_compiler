use knoten_core::executor::ExecutionEngine;
use std::env;
use std::fs;

fn main() {
    let mut engine = ExecutionEngine::new();

    // Check if we are bundled (Sprint 11)
    if let Some(bundled_json) = option_env!("KNOTEN_BUNDLE") {
        println!("Running embedded KnotenCore bundle...");
        let ast = serde_json::from_str(bundled_json)
            .expect("Failed to parse bundled KnotenCore JSON AST");
        let result = engine.execute(&ast);
        println!("\nExecution Finished.\nResult: {}", result);
        return;
    }

    // Normal JIT mode
    let args: Vec<String> = env::args().collect();

    let mut is_check = false;
    let mut no_opt = false;
    let mut file_path = String::new();

    for arg in args.iter().skip(1) {
        if arg == "--check" {
            is_check = true;
        } else if arg == "--no-opt" {
            no_opt = true;
        } else {
            file_path = arg.clone();
        }
    }

    if file_path.is_empty() {
        eprintln!("Usage: run_aec [--check] [--no-opt] <path_to.json>");
        std::process::exit(1);
    }

    println!("CWD: {:?}", env::current_dir().unwrap());
    println!("Loading KnotenCore Script: {}", file_path);

    let json_string = fs::read_to_string(&file_path).expect("Failed to read file");
    let mut ast = serde_json::from_str(&json_string).expect("Failed to parse KnotenCore JSON AST");

    let mut typer = knoten_core::optimizer::TypeChecker::new();
    let _ = typer.check(&ast);
    if !typer.errors.is_empty() {
        eprintln!("\n[TypeError] Static Type Inference Failed:");
        for err in typer.errors {
            eprintln!(" - {}", err);
        }
        std::process::exit(1);
    }

    if !no_opt {
        let before_nodes = knoten_core::optimizer::count_nodes(&ast);
        ast = knoten_core::optimizer::optimize(ast);
        let after_nodes = knoten_core::optimizer::count_nodes(&ast);
        println!(
            "Compiler Optimization: Reduced AST from {} to {} nodes.",
            before_nodes, after_nodes
        );
    }

    if is_check {
        use knoten_core::validator::Validator;
        let mut validator = Validator::new();
        match validator.validate(&ast) {
            Ok(_) => {
                println!("\nSyntax OK");
                std::process::exit(0);
            }
            Err(errors) => {
                eprintln!("\nValidation Failed:");
                for err in errors {
                    eprintln!(" - {}", err);
                }
                std::process::exit(1);
            }
        }
    }

    let result = engine.execute(&ast);

    println!("\nExecution Finished.\nResult: {}", result);
}
