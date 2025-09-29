use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    println!("Starting basic krunvm test...");
    
    let start = Instant::now();
    
    // Test 1: Basic blocking command
    println!("Test 1: Direct krunvm call");
    let result1 = tokio::task::spawn_blocking(|| {
        std::process::Command::new("krunvm")
            .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
            .arg("--help")
            .output()
    }).await;
    
    match result1 {
        Ok(Ok(output)) => println!("✅ Direct call works: {} bytes", output.stdout.len()),
        Ok(Err(e)) => println!("❌ Command error: {}", e),
        Err(e) => println!("❌ Join error: {}", e),
    }
    
    // Test 2: List VMs
    println!("Test 2: List VMs");
    let result2 = tokio::task::spawn_blocking(|| {
        std::process::Command::new("krunvm")
            .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
            .arg("list")
            .output()
    }).await;
    
    match result2 {
        Ok(Ok(output)) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("✅ List works: {} lines", stdout.lines().count());
        },
        Ok(Err(e)) => println!("❌ Command error: {}", e),
        Err(e) => println!("❌ Join error: {}", e),
    }
    
    let elapsed = start.elapsed();
    println!("Total time: {:?}", elapsed);
    
    if elapsed > Duration::from_secs(5) {
        println!("⚠️ Taking too long, something is wrong");
    } else {
        println!("✅ All good, the issue is elsewhere");
    }
}