use aether_compiler::ast::Node;
use std::env;
use std::fs;
use std::path::Path;

// Simple Rust function parser (Sprint 27)
fn parse_rust_file(file_content: &str, module_name: &str) -> Node {
    let mut functions = Vec::new();

    for line in file_content.lines() {
        let line = line.trim();
        if line.starts_with("pub fn ") {
            // Extract the function signature
            let sig_start = line.find("pub fn ").unwrap() + 7;
            let sig_end = line.find('{').unwrap_or(line.len());
            let sig = line[sig_start..sig_end].trim();

            if let Some(paren_start) = sig.find('(') {
                if let Some(paren_end) = sig.find(')') {
                    let fn_name = sig[0..paren_start].trim();
                    let args_str = &sig[paren_start + 1..paren_end];

                    let mut arg_names = Vec::new();
                    if !args_str.trim().is_empty() {
                        for arg_def in args_str.split(',') {
                            let parts: Vec<&str> = arg_def.split(':').collect();
                            if !parts.is_empty() {
                                arg_names.push(parts[0].trim().to_string());
                            }
                        }
                    }

                    // Build the ExternCall node mapped to those arguments
                    let mut call_args = Vec::new();
                    for arg in &arg_names {
                        call_args.push(Node::Identifier(arg.clone()));
                    }

                    let extern_call = Node::ExternCall {
                        module: module_name.to_string(),
                        function: fn_name.to_string(),
                        args: call_args,
                    };

                    let fn_def = Node::FnDef(
                        fn_name.to_string(),
                        arg_names,
                        Box::new(Node::Block(vec![Node::Return(Box::new(extern_call))])),
                    );

                    functions.push(fn_def);
                }
            }
        }
    }

    Node::Block(functions)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rust_ingest <path_to.rs>");
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    let module_name = input_path.file_stem().unwrap().to_str().unwrap();

    let content = fs::read_to_string(input_path).expect("Failed to read input rust file");

    let aether_ast = parse_rust_file(&content, module_name);

    let json_output = serde_json::to_string_pretty(&aether_ast).expect("Failed to serialize AST");

    let output_filename = format!("{}.aec", module_name);
    // Placed directly alongside the demos for integration evaluations
    let output_path = Path::new("examples/core").join(&output_filename);

    fs::write(&output_path, json_output).expect("Failed to write FFI interface");

    println!(
        "[Rust-Ingestor] Successfully generated FFI AetherCore binary: {:?}",
        output_path
    );
}
