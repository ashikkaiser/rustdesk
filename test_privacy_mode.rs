use std::env;
mod src {
    pub mod privacy_mode;
    pub mod privacy_mode {
        pub mod win_direct_overlay;
    }
    pub mod lang;
}

use src::privacy_mode::win_direct_overlay::{turn_on_privacy, turn_off_privacy};

fn main() {
    env_logger::init();
    
    println!("Testing privacy mode direct overlay...");
    
    // Test turning on privacy mode
    println!("Turning on privacy mode...");
    turn_on_privacy(0, 0, 1920, 1080);
    
    // Keep it on for a few seconds
    std::thread::sleep(std::time::Duration::from_secs(5));
    
    // Turn off privacy mode
    println!("Turning off privacy mode...");
    turn_off_privacy();
    
    println!("Test completed.");
}