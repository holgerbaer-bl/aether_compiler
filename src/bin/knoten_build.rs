use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: knoten_build <path_to.json>");
        std::process::exit(1);
    }

    let input_path = &args[1];
    let path = PathBuf::from(input_path);

    if !path.exists() {
        eprintln!("Error: File '{}' not found.", input_path);
        std::process::exit(1);
    }

    let absolute_path = fs::canonicalize(&path).expect("Failed to get absolute path");
    let absolute_path_str = absolute_path.to_str().expect("Path is not valid UTF-8");

    // Windows backslash fix for include_str!()
    let safe_path_str = absolute_path_str.replace("\\", "/");

    let file_stem = path
        .file_stem()
        .expect("Invalid filename")
        .to_str()
        .expect("Invalid UTF-8 filename");

    #[cfg(windows)]
    let out_file_name = format!("{}.exe", file_stem);
    #[cfg(not(windows))]
    let out_file_name = file_stem.to_string();

    println!(
        "🎨 KnotenCore Bundler: Compiling '{}' into standalone executable '{}'",
        input_path, out_file_name
    );

    // Generate the standalone launcher source file dynamically
    let launcher_source = format!(
        r#"
use knoten_core::executor::ExecutionEngine;

fn main() {{
    println!("Running embedded KnotenCore bundle...");
    
    // Statically bake the JSON file content into the binary string section
    let bundled_json = include_str!("{}");
    
    let ast = serde_json::from_str(bundled_json)
        .expect("Failed to parse bundled KnotenCore JSON AST");
        
    let mut engine = ExecutionEngine::new();
    let result = engine.execute(&ast);
    
    println!("\nExecution Finished.\nResult: {{}}", result);
}}
"#,
        safe_path_str
    );

    let temp_launcher_path = "src/bin/_knoten_temp_launcher.rs";
    fs::write(temp_launcher_path, launcher_source)
        .expect("Failed to write temporary launcher source file");

    // Call Cargo Build on the temporary binary hook
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--bin")
        .arg("_knoten_temp_launcher")
        .status();

    // Clean up the temporary rust script immediately regardless of build success
    let _ = fs::remove_file(temp_launcher_path);

    if !status
        .expect("Failed to execute cargo build process")
        .success()
    {
        eprintln!("❌ Build failed.");
        std::process::exit(1);
    }

    // Locate the compiled binary
    #[cfg(windows)]
    let compiled_bin = PathBuf::from("target")
        .join("release")
        .join("_knoten_temp_launcher.exe");
    #[cfg(not(windows))]
    let compiled_bin = PathBuf::from("target")
        .join("release")
        .join("_knoten_temp_launcher");

    if !compiled_bin.exists() {
        eprintln!(
            "❌ Could not find the compiled executable at {:?}",
            compiled_bin
        );
        std::process::exit(1);
    }

    // Copy to the current directory matching the original json name
    let dest_path = PathBuf::from(&out_file_name);
    fs::copy(&compiled_bin, &dest_path).expect("Failed to copy executable to output directory");

    println!("✅ Bundle Successful!");
    println!("Standalone Application created: {}", dest_path.display());
}
