use knoten_core::executor::ExecutionEngine;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

// Embedded at compile-time: absolute path to the knoten_core library source.
const KNOTEN_CORE_PATH: &str = env!("CARGO_MANIFEST_DIR");

fn main() {
    // Spawn with 8MB stack to support deep recursion in KnotenCore scripts
    let builder = std::thread::Builder::new().stack_size(8 * 1024 * 1024);
    let handler = builder
        .spawn(run)
        .expect("Failed to spawn KnotenCore runtime thread");
    handler.join().unwrap();
}

fn run() {
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

    let args: Vec<String> = env::args().collect();

    // ── Subcommand: build ─────────────────────────────────────────────
    // Usage: run_knc build <file.nod>
    if args.len() >= 2 && args[1] == "build" {
        if args.len() < 3 {
            eprintln!("Usage: run_knc build <path_to.nod>");
            std::process::exit(1);
        }
        build_standalone(&args[2]);
        return;
    }

    // ── Legacy flags ──────────────────────────────────────────────────
    let mut is_check = false;
    let mut no_opt = false;
    let mut transpile = false;
    let mut file_path = String::new();

    for arg in args.iter().skip(1) {
        if arg == "--check" {
            is_check = true;
        } else if arg == "--no-opt" {
            no_opt = true;
        } else if arg == "--transpile" {
            transpile = true;
        } else {
            file_path = arg.clone();
        }
    }

    if file_path.is_empty() {
        eprintln!("Usage: run_knc [--check] [--no-opt] [--transpile] <path_to.nod>");
        eprintln!("       run_knc build <path_to.nod>");
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

    if transpile {
        let rs_code = knoten_core::compiler::codegen::generate_rust_code(&ast);
        std::fs::write("output.rs", &rs_code).expect("Failed to write output.rs");
        println!("\nTranspiled successfully to output.rs:");
        println!("---------------------------------------");
        println!("{}", rs_code);
        return;
    }

    let result = engine.execute(&ast);

    println!("\nExecution Finished.\nResult: {}", result);
}

/// Full one-click build pipeline:
/// 1. Parse & optimise the .nod file
/// 2. Transpile to Rust source
/// 3. Scaffold a temporary Cargo project with knoten_core as a local dep
/// 4. `cargo build --release` with LTO enabled
/// 5. Copy the named binary back to the current working directory
fn build_standalone(nod_path: &str) {
    // ── Step 1: Parse & optimise ──────────────────────────────────────
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(" KnotenCore Build Pipeline");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("[1/5] Parsing  : {}", nod_path);

    let json_string = fs::read_to_string(nod_path).unwrap_or_else(|_| {
        eprintln!("Error: Cannot read '{}'", nod_path);
        std::process::exit(1);
    });
    let mut ast: knoten_core::ast::Node = serde_json::from_str(&json_string).unwrap_or_else(|e| {
        eprintln!("Error: Invalid AST JSON — {}", e);
        std::process::exit(1);
    });

    let before = knoten_core::optimizer::count_nodes(&ast);
    ast = knoten_core::optimizer::optimize(ast);
    let after = knoten_core::optimizer::count_nodes(&ast);
    println!("[2/5] Optimise : {} → {} nodes", before, after);

    // ── Step 2: Transpile ─────────────────────────────────────────────
    let rs_code = knoten_core::compiler::codegen::generate_rust_code(&ast);

    // Derive output binary name from the .nod filename stem
    let stem = Path::new(nod_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("knoten_app");

    println!("[3/5] Transpile: {} → {}.rs", nod_path, stem);

    // ── Step 3: Scaffold temp Cargo project ───────────────────────────
    let tmp_dir = std::env::temp_dir().join(format!("knoten_build_{}", stem));
    let src_dir = tmp_dir.join("src");
    fs::create_dir_all(&src_dir).expect("Cannot create temp build directory");

    // Cargo.toml — path dependency points to our library source
    let cargo_toml = format!(
        r#"[package]
name = "{stem}"
version = "0.1.0"
edition = "2021"

[dependencies]
knoten_core = {{ path = "{lib_path}" }}

[profile.release]
lto = "fat"
opt-level = 3
codegen-units = 1
strip = "symbols"
"#,
        stem = stem,
        lib_path = KNOTEN_CORE_PATH.replace('\\', "/"),
    );

    fs::write(tmp_dir.join("Cargo.toml"), &cargo_toml).expect("Cannot write temporary Cargo.toml");
    fs::write(src_dir.join("main.rs"), &rs_code).expect("Cannot write temporary main.rs");

    println!("[4/5] Compile  : cargo build --release (LTO + opt-level 3)");
    println!("      Build dir: {}", tmp_dir.display());

    // ── Step 4: Compile ───────────────────────────────────────────────
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&tmp_dir)
        .status()
        .expect("Failed to invoke cargo. Is it installed and in PATH?");

    if !status.success() {
        eprintln!("\n[Build FAILED] cargo exited with status {}", status);
        std::process::exit(1);
    }

    // ── Step 5: Copy binary to cwd ────────────────────────────────────
    let binary_name = if cfg!(windows) {
        format!("{}.exe", stem)
    } else {
        stem.to_string()
    };

    let built = tmp_dir.join("target").join("release").join(&binary_name);
    let dest = env::current_dir().unwrap().join(&binary_name);

    fs::copy(&built, &dest).unwrap_or_else(|e| {
        eprintln!("Could not copy binary: {}", e);
        std::process::exit(1);
    });

    println!(
        "[5/5] Done!    : {} ({} bytes)",
        dest.display(),
        fs::metadata(&dest).map(|m| m.len()).unwrap_or(0)
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(" Binary ready — run it anywhere:");
    println!("   .\\{}", binary_name);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
