use std::env;
use std::path::Path;
use std::process::{Command, exit};

fn main() {
    // Collect arguments excluding the binary name
    let args: Vec<String> = env::args().skip(1).collect();
    
    // Route commands
    match args.get(0).map(|s| s.as_str()) {
        Some("build") => {
            let target = args.get(1).map(|s| s.as_str());
            run_build(target);
        }
        Some("help") | Some("--help") | None => {
            print_usage();
        }
        Some(cmd) => {
            eprintln!("Unknown command: {}", cmd);
            print_usage();
            exit(1);
        }
    }
}

fn get_host_target() -> String {
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("Failed to run rustc to detect host target");
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.starts_with("host: ") {
            return line.replace("host: ", "").trim().to_string();
        }
    }
    panic!("Could not detect host target triple");
}

fn print_usage() {
    println!("Usage: cargo xtask <command> [args]");
    println!("\nCommands:");
    println!("  build [target]    Builds the project (default: host target)");
    println!("\nExample:");
    println!("  cargo xtask build");
    println!("  cargo xtask build aarch64-unknown-linux-gnu");
}

fn run_build(target_opt: Option<&str>) {
    let final_target = target_opt.unwrap_or_else(|| {
        let host = get_host_target();
        println!("--- No target specified: Defaulting to {} ---", host);
        "" // Placeholder, logic below uses get_host_target()
    });

    let target = if final_target.is_empty() { get_host_target() } else { final_target.to_string() };

    // 1. Detect external drive
    let external_root = Path::new("/Volumes/DeepSpaceExtSDD");
    
    // 2. Determine target directory path
    let target_dir = if external_root.exists() {
        let path = external_root.join("target").join(&target);
        println!("--- External volume detected: Using {} ---", path.display());
        path.to_string_lossy().into_owned()
    } else {
        format!("target/{}", target)
    };
    
    println!("--- Building for: {} ---", target);

    // 3. Execute the build command
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
       .arg("--target")
       .arg(&target)
       .env("CARGO_TARGET_DIR", &target_dir);

    let status = cmd.status().expect("Failed to execute cargo build");

    if !status.success() {
        eprintln!("Build failed for target: {}", target);
        exit(1);
    } else {
        println!("--- Build successful! Artifacts located in: {} ---", target_dir);
    }
}
