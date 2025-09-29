use std::time::{Duration, Instant};
use std::process::Command;

fn main() {
    println!("Testing krunvm execution...");
    
    let start = Instant::now();
    
    // Test 1: krunvm --help
    println!("Test 1: krunvm --help");
    match Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .arg("--help")
        .output() 
    {
        Ok(output) => println!("✅ Help: {} bytes", output.stdout.len()),
        Err(e) => println!("❌ Help error: {}", e),
    }
    
    // Test 2: krunvm list  
    println!("Test 2: krunvm list");
    match Command::new("krunvm")
        .env("DYLD_LIBRARY_PATH", "/opt/homebrew/lib")
        .arg("list")
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("✅ List: {} lines", stdout.lines().count());
            
            // Parse VM names
            let vm_names: Vec<_> = stdout
                .lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if !line.is_empty()
                        && !line.starts_with(" ")
                        && !line.contains("CPUs:")
                        && !line.contains("RAM")
                        && !line.contains("DNS")
                        && !line.contains("Buildah")
                        && !line.contains("Workdir")
                        && !line.contains("Mapped")
                    {
                        Some(line.to_string())
                    } else {
                        None
                    }
                })
                .collect();
            
            println!("Parsed VMs: {:?}", vm_names);
            
            let vortex_vms: Vec<_> = vm_names.iter()
                .filter(|name| name.starts_with("vortex-"))
                .collect();
            
            println!("Vortex VMs: {:?}", vortex_vms);
        },
        Err(e) => println!("❌ List error: {}", e),
    }
    
    let elapsed = start.elapsed();
    println!("Total time: {:?}", elapsed);
    
    if elapsed > Duration::from_secs(2) {
        println!("⚠️ Commands are slow");
    } else {
        println!("✅ Commands are fast, issue is in Vortex init");
    }
}