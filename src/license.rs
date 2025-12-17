use serde::{Deserialize, Serialize};
use hbb_common::{config, log};

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseResponse {
    pub valid: bool,
    pub company: Option<Company>,
    pub limits: Option<Limits>,
    pub subscription: Option<Subscription>,
    #[serde(rename = "relayServers")]
    pub relay_servers: Option<Vec<RelayServer>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Company {
    pub id: String,
    pub name: String,
    #[serde(rename = "licenseKey")]
    pub license_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Limits {
    #[serde(rename = "maxAgents")]
    pub max_agents: i32,
    #[serde(rename = "maxClients")]
    pub max_clients: i32,
    #[serde(rename = "maxSessions")]
    pub max_sessions: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub status: String,
    pub start: String,
    pub end: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelayServer {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub endpoint: String,
}

/// Validate license and configure servers from license response
pub fn validate_and_configure_license(license_key: &str) -> Result<LicenseResponse, String> {
    log::info!("Validating license key: {}...", &license_key[..license_key.len().min(10)]);
    
    let license_url = "https://manager.cloudydesk.us/api/public/license";
    
    // Prepare request body
    let body = serde_json::json!({
        "licenseKey": license_key
    });
    
    log::info!("Sending license validation request to: {}", license_url);
    
    // Make POST request to validate license
    match crate::post_request_sync(license_url.to_string(), body.to_string(), "Content-Type: application/json") {
        Ok(response) => {
            log::info!("License validation response received");
            
            // Parse the response
            match serde_json::from_str::<LicenseResponse>(&response) {
                Ok(license_data) => {
                    if license_data.valid {
                        log::info!("✓ License is VALID!");
                        
                        // Configure servers from license response
                        if let Some(relay_servers) = &license_data.relay_servers {
                            if let Some(first_server) = relay_servers.first() {
                                configure_servers_from_license(first_server, license_key);
                            } else {
                                log::warn!("No relay servers found in license response");
                            }
                        } else {
                            log::warn!("No relay servers in license response");
                        }
                        
                        // Log company and subscription info
                        if let Some(company) = &license_data.company {
                            log::info!("Licensed to: {}", company.name);
                        }
                        
                        if let Some(subscription) = &license_data.subscription {
                            log::info!("Subscription status: {} (expires: {})", 
                                subscription.status, subscription.end);
                        }
                        
                        if let Some(limits) = &license_data.limits {
                            log::info!("License limits - Agents: {}, Clients: {}, Sessions: {}",
                                limits.max_agents, limits.max_clients, limits.max_sessions);
                        }
                        
                        Ok(license_data)
                    } else {
                        let err = "License is invalid or expired".to_string();
                        log::error!("✗ {}", err);
                        Err(err)
                    }
                }
                Err(e) => {
                    let err = format!("Failed to parse license response: {}", e);
                    log::error!("✗ {}", err);
                    log::debug!("Response was: {}", response);
                    Err(err)
                }
            }
        }
        Err(e) => {
            let err = format!("License validation request failed: {}", e);
            log::error!("✗ {}", err);
            Err(err)
        }
    }
}

/// Configure server settings from license data
fn configure_servers_from_license(relay_server: &RelayServer, license_key: &str) {
    log::info!("Configuring servers from license data:");
    log::info!("  Relay Server: {} ({})", relay_server.name, relay_server.endpoint);
    
    // Extract host from the relay server
    let relay_host = &relay_server.host;
    
    // Set rendezvous server (typically port 21116)
    let rendezvous_server = format!("{}:21116", relay_host);
    config::Config::set_option("custom-rendezvous-server".to_string(), rendezvous_server.clone());
    log::info!("  ✓ Rendezvous: {}", rendezvous_server);
    
    // Set relay server (typically port 21117)
    let relay_endpoint = format!("{}:21117", relay_host);
    config::Config::set_option("relay-server".to_string(), relay_endpoint.clone());
    log::info!("  ✓ Relay: {}", relay_endpoint);
    
    // Set API server (HTTP, typically port 21114)
    let api_server = format!("http://{}:21114", relay_host);
    config::Config::set_option("api-server".to_string(), api_server.clone());
    log::info!("  ✓ API Server: {}", api_server);
    
    // Set the license key as the encryption key
    config::Config::set_option("key".to_string(), license_key.to_string());
    log::info!("  ✓ Encryption key set from license");
    
    // Set connection type to incoming (for agents)
    config::Config::set_conn_type("incoming");
    log::info!("  ✓ Connection type: incoming");
    
    // Store the license key in config for future use
    config::Config::set_option("license-key".to_string(), license_key.to_string());
    
    log::info!("✓ Server configuration completed successfully");
}

/// Get license key - ONLY from build-time injection
/// The license key is embedded during build and cannot be changed after
pub fn get_license_key() -> Option<String> {
    // FIRST PRIORITY: Check for build-time injected license key via environment variable
    // Set during build with: CLOUDYDESK_LICENSE_KEY=your_key cargo build
    if let Some(key) = option_env!("CLOUDYDESK_LICENSE_KEY") {
        if !key.is_empty() {
            log::info!("License key found: Build-time injected (compile-time env)");
            return Some(key.to_string());
        }
    }
    
    // SECOND PRIORITY: Check for license.conf file in the same directory as executable
    // This file should be bundled during build/installer creation
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let license_file = exe_dir.join("license.conf");
            log::debug!("Checking for license file: {}", license_file.display());
            if license_file.exists() {
                log::info!("Found license.conf file");
                if let Ok(content) = std::fs::read_to_string(&license_file) {
                    // Parse the file, looking for LicenseKey= line
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("LicenseKey=") {
                            if let Some(key) = line.strip_prefix("LicenseKey=") {
                                let key = key.trim().to_string();
                                if !key.is_empty() {
                                    log::info!("License key found: license.conf file");
                                    return Some(key);
                                }
                            }
                        } else if !line.is_empty() && !line.starts_with("#") {
                            // If it's not a comment and not empty, treat the whole line as the key
                            let key = line.to_string();
                            if !key.is_empty() {
                                log::info!("License key found: license.conf file (direct)");
                                return Some(key);
                            }
                        }
                    }
                }
            }
        }
    }
    
    log::error!("No license key found!");
    log::error!("License key must be injected during build time.");
    log::error!("Build with: CLOUDYDESK_LICENSE_KEY=your_key cargo build");
    log::error!("Or include license.conf file in the build directory");
    None
}

/// Initialize license validation on startup
pub fn init_license_validation() -> bool {
    log::info!("========================================");
    log::info!("Initializing License Validation");
    log::info!("========================================");
    
    // Get license key
    let license_key = match get_license_key() {
        Some(key) => key,
        None => {
            log::error!("✗ No license key provided!");
            log::error!("Options:");
            log::error!("  1. Use command line: --license-key YOUR_KEY");
            log::error!("  2. Create license.conf file with: LicenseKey=YOUR_KEY");
            log::error!("  3. Set via UI (stored in config)");
            return false;
        }
    };
    
    // Validate license and configure servers
    match validate_and_configure_license(&license_key) {
        Ok(_) => {
            log::info!("========================================");
            log::info!("✓ License Validation SUCCESS");
            log::info!("========================================");
            true
        }
        Err(e) => {
            log::error!("========================================");
            log::error!("✗ License Validation FAILED");
            log::error!("Error: {}", e);
            log::error!("========================================");
            false
        }
    }
}
