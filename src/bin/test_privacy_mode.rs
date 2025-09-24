use std::thread;
use std::time::Duration;
use hbb_common::tokio;

#[tokio::main]
async fn main() {
    // Initialize logging with INFO level to see our messages
    hbb_common::init_log(false, "test_privacy_mode");

    println!("Testing Privacy Mode Activation");
    println!("==============================");

    // Activate privacy mode with direct overlay implementation  
    let impl_key = "privacy_mode_impl_direct_overlay";
    let conn_id = 436033839;

    println!("Activating privacy mode with impl_key: '{}', conn_id: {}", impl_key, conn_id);

    match libcloudydesk::privacy_mode::turn_on_privacy(impl_key, conn_id).await {
        Some(Ok(success)) => {
            if success {
                println!("✅ Privacy mode activated successfully!");
                println!("The black overlay should now be visible with red corners");
                println!("Input hooks should be installed and capturing events");
                
                // Keep running to capture input events
                println!("\nWaiting 10 seconds then testing agent input...");
                thread::sleep(Duration::from_secs(3));
                
                // Simulate some agent input to test the hooks
                println!("Simulating agent input to test hook filtering...");
                
                // Run the Python script from within the test
                let output = std::process::Command::new("python")
                    .arg("test_agent_input.py")
                    .current_dir(".")
                    .output();
                
                match output {
                    Ok(result) => {
                        println!("Agent input test output:");
                        println!("{}", String::from_utf8_lossy(&result.stdout));
                        if !result.stderr.is_empty() {
                            println!("Errors: {}", String::from_utf8_lossy(&result.stderr));
                        }
                    }
                    Err(e) => {
                        println!("Failed to run agent input test: {}", e);
                    }
                }
                
                println!("Wait 7 more seconds to see logs...");
                thread::sleep(Duration::from_secs(7));
            } else {
                println!("❌ Privacy mode activation returned false");
            }
        }
        Some(Err(e)) => {
            println!("❌ Privacy mode activation failed: {}", e);
        }
        None => {
            println!("❌ Privacy mode activation returned None");
        }
    }

    println!("Test completed. Shutting down...");
}