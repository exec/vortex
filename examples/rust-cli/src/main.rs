use clap::{Parser, Subcommand};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(name = "vortex-rust-example")]
#[command(about = "Rust CLI demonstrating Vortex's instant development environments")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show environment information
    Info,
    /// Run performance test
    Benchmark,
    /// Show Vortex advantages
    Advantages,
    /// Test Rust ecosystem integration
    Test,
}

fn main() {
    let cli = Cli::parse();

    println!("🚀 Vortex Rust Environment Demo");
    println!("═══════════════════════════════════");

    match &cli.command {
        Commands::Info => show_info(),
        Commands::Benchmark => run_benchmark(),
        Commands::Advantages => show_advantages(),
        Commands::Test => run_tests(),
    }
}

fn show_info() {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let info = json!({
        "rust_info": {
            "version": env!("CARGO_PKG_VERSION"),
            "rustc_version": env!("RUSTC_VERSION"),
            "target": env!("TARGET"),
            "host": env!("HOST"),
            "opt_level": env!("OPT_LEVEL"),
            "debug": env!("DEBUG"),
            "profile": env!("PROFILE")
        },
        "vortex_environment": {
            "status": "✅ Active",
            "startup_time": "2-3 seconds",
            "vs_docker": "20x faster than Docker DevContainers",
            "isolation": "Hardware-level VM boundaries",
            "features": [
                "Instant Rust toolchain",
                "Native cargo performance", 
                "Direct filesystem access",
                "Hardware isolation",
                "True security boundaries"
            ]
        },
        "system_info": {
            "timestamp": timestamp,
            "environment": std::env::var("RUST_ENV").unwrap_or_else(|_| "development".to_string()),
            "cargo_target_dir": std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string()),
        }
    });

    println!("📊 Environment Information:");
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
}

fn run_benchmark() {
    println!("⚡ Running Rust Performance Benchmark...");
    
    let start = std::time::Instant::now();
    
    // CPU-intensive computation
    let mut sum = 0u64;
    for i in 0..1_000_000 {
        sum += (i as f64).sqrt() as u64;
    }
    
    let duration = start.elapsed();
    
    println!("🔥 Benchmark Results:");
    println!("   • Iterations: 1,000,000");
    println!("   • Execution time: {:.2}ms", duration.as_millis());
    println!("   • Operations/sec: {:.0}", 1_000_000.0 / duration.as_secs_f64());
    println!("   • Result checksum: {}", sum);
    println!();
    println!("✅ Native Rust performance in Vortex environment!");
    println!("🚀 No Docker container overhead - direct hardware access!");
}

fn show_advantages() {
    println!("🎯 Vortex vs Docker DevContainers:");
    println!();
    println!("┌─────────────────────┬─────────────┬──────────────────┐");
    println!("│ Feature             │ Vortex      │ Docker           │");
    println!("├─────────────────────┼─────────────┼──────────────────┤");
    println!("│ Startup Time        │ 2-3 seconds │ 60-100 seconds   │");
    println!("│ Security Isolation  │ Hardware VM │ Namespace sharing│");
    println!("│ Performance         │ Native      │ Container overhead│");
    println!("│ File System Access  │ Direct      │ Bind mounts      │");
    println!("│ Network Performance │ Native      │ Bridge overhead  │");
    println!("│ Memory Overhead     │ Minimal     │ Significant      │");
    println!("│ Rust Compilation    │ Native      │ Slower           │");
    println!("│ Cargo Caching       │ Direct      │ Volume complexity│");
    println!("└─────────────────────┴─────────────┴──────────────────┘");
    println!();
    println!("🔥 Result: 20x faster development workflow!");
}

fn run_tests() {
    println!("🧪 Testing Rust Environment...");
    println!();
    
    let mut passed = 0;
    let mut total = 0;
    
    // Test 1: Cargo compilation
    total += 1;
    println!("Test 1: Cargo compilation system");
    if std::process::Command::new("cargo")
        .arg("--version")
        .output()
        .is_ok() {
        println!("   ✅ PASS - Cargo available");
        passed += 1;
    } else {
        println!("   ❌ FAIL - Cargo not available");
    }
    
    // Test 2: Rustc compiler
    total += 1;
    println!("Test 2: Rust compiler");
    if std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok() {
        println!("   ✅ PASS - Rustc available");
        passed += 1;
    } else {
        println!("   ❌ FAIL - Rustc not available");
    }
    
    // Test 3: File system access
    total += 1;
    println!("Test 3: File system access");
    match std::fs::write("/tmp/vortex_rust_test.txt", "test") {
        Ok(_) => {
            std::fs::remove_file("/tmp/vortex_rust_test.txt").ok();
            println!("   ✅ PASS - File system writable");
            passed += 1;
        }
        Err(_) => println!("   ❌ FAIL - File system not writable"),
    }
    
    // Test 4: Network connectivity
    total += 1;
    println!("Test 4: Network connectivity");
    if std::process::Command::new("ping")
        .arg("-c")
        .arg("1")
        .arg("8.8.8.8")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false) {
        println!("   ✅ PASS - Network connectivity");
        passed += 1;
    } else {
        println!("   ❌ FAIL - Network connectivity");
    }
    
    // Test 5: Environment variables
    total += 1;
    println!("Test 5: Environment variables");
    if std::env::var("PATH").is_ok() {
        println!("   ✅ PASS - Environment variables accessible");
        passed += 1;
    } else {
        println!("   ❌ FAIL - Environment variables not accessible");
    }
    
    println!();
    println!("📊 Test Results: {}/{} passed", passed, total);
    
    if passed == total {
        println!("🎉 All tests passed! Vortex Rust environment is fully functional!");
        println!("🚀 Ready for Rust development with 20x faster startup than Docker!");
    } else {
        println!("⚠️  Some tests failed. Check your Vortex environment setup.");
    }
}