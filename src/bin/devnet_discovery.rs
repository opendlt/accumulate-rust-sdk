#!/usr/bin/env cargo
//! Accumulate DevNet Discovery Tool
//!
//! Automatically discovers running DevNet instances and configures environment variables
//! for seamless integration with Rust SDK examples and applications.
//!
//! This tool:
//! 1. Detects local DevNet instances (Docker containers)
//! 2. Parses logs and configuration to find API endpoints
//! 3. Extracts faucet account information
//! 4. Generates environment configuration files
//! 5. Provides PowerShell/Bash commands for environment setup

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::process::Command;

const DEFAULT_DEVNET_DIR: &str = r"C:\Accumulate_Stuff\devnet-accumulate-instance";
const ENV_FILE: &str = ".env.local";

/// DevNet configuration discovered from environment
#[derive(Debug, Default)]
struct DevNetConfig {
    devnet_dir: String,
    rpc_url_v2: String,
    rpc_url_v3: String,
    faucet_account: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Accumulate DevNet Discovery");
    println!("==============================");

    let mut config = DevNetConfig::default();

    // Step 1: Determine DevNet directory
    config.devnet_dir =
        env::var("ACC_DEVNET_DIR").unwrap_or_else(|_| DEFAULT_DEVNET_DIR.to_string());

    println!("📁 DevNet Directory: {}", config.devnet_dir);

    // Step 2: Check if DevNet directory exists
    if !Path::new(&config.devnet_dir).exists() {
        eprintln!("❌ DevNet directory not found: {}", config.devnet_dir);
        eprintln!("   Please ensure DevNet is cloned and accessible");
        std::process::exit(1);
    }

    // Step 3: Discover RPC URLs (check env vars first, then defaults)
    config.rpc_url_v2 =
        env::var("ACC_RPC_URL_V2").unwrap_or_else(|_| discover_rpc_url_v2(&config.devnet_dir));

    config.rpc_url_v3 =
        env::var("ACC_RPC_URL_V3").unwrap_or_else(|_| discover_rpc_url_v3(&config.devnet_dir));

    // Step 4: Discover faucet account
    config.faucet_account = env::var("ACC_FAUCET_ACCOUNT")
        .unwrap_or_else(|_| discover_faucet_account(&config.devnet_dir));

    // Step 5: Test connectivity
    println!("\n🔗 Testing DevNet Connectivity");
    test_devnet_connectivity(&config)?;

    // Step 6: Write .env.local file
    write_env_file(&config)?;

    println!("\n✅ DevNet Discovery Complete!");
    println!("🚀 You can now run examples with: cargo run --example <example_name>");

    Ok(())
}

fn discover_rpc_url_v2(devnet_dir: &str) -> String {
    println!("🔍 Discovering V2 RPC URL...");

    // Try to find V2 port from Docker compose or logs
    if let Ok(port) = discover_port_from_docker(devnet_dir, "accumulate-core") {
        let url = format!("http://localhost:{}/v2", port);
        println!("   Found V2 URL: {}", url);
        return url;
    }

    // Default fallback
    let default_url = "http://localhost:26660/v2";
    println!("   Using default V2 URL: {}", default_url);
    default_url.to_string()
}

fn discover_rpc_url_v3(devnet_dir: &str) -> String {
    println!("🔍 Discovering V3 RPC URL...");

    // Try to find V3 port from Docker compose or logs
    if let Ok(port) = discover_port_from_docker(devnet_dir, "accumulate-core-v3") {
        let url = format!("http://localhost:{}/v3", port);
        println!("   Found V3 URL: {}", url);
        return url;
    }

    // Default fallback - often same port as V2 with different path
    let default_url = "http://localhost:26660/v3";
    println!("   Using default V3 URL: {}", default_url);
    default_url.to_string()
}

fn discover_faucet_account(devnet_dir: &str) -> String {
    println!("🔍 Discovering Faucet Account...");

    // Try to find faucet account from Docker logs
    if let Ok(account) = discover_faucet_from_logs(devnet_dir) {
        println!("   Found faucet account: {}", account);
        return account;
    }

    // Try to find from config files
    if let Ok(account) = discover_faucet_from_config(devnet_dir) {
        println!("   Found faucet account in config: {}", account);
        return account;
    }

    // Default fallback
    let default_account = "acc://faucet.acme/ACME";
    println!("   Using default faucet account: {}", default_account);
    default_account.to_string()
}

fn discover_port_from_docker(
    devnet_dir: &str,
    service_name: &str,
) -> Result<u16, Box<dyn std::error::Error>> {
    // Try docker-compose.yml first
    let compose_path = Path::new(devnet_dir).join("docker-compose.yml");
    if compose_path.exists() {
        if let Ok(content) = fs::read_to_string(&compose_path) {
            // Look for port mappings like "26660:26660"
            for line in content.lines() {
                if line.contains(service_name) || line.trim().starts_with("ports:") {
                    // Simple port extraction - look for patterns like "26660:26660"
                    if let Some(port_str) = extract_port_from_line(line) {
                        if let Ok(port) = port_str.parse::<u16>() {
                            return Ok(port);
                        }
                    }
                }
            }
        }
    }

    // Try docker ps to find running containers
    let output = Command::new("docker")
        .args(&["ps", "--format", "table {{.Names}}\\t{{.Ports}}"])
        .output();

    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("accumulate") && line.contains("26660") {
                if let Some(port_str) = extract_port_from_line(line) {
                    if let Ok(port) = port_str.parse::<u16>() {
                        return Ok(port);
                    }
                }
            }
        }
    }

    Err("Could not discover port from Docker".into())
}

fn extract_port_from_line(line: &str) -> Option<&str> {
    // Look for patterns like "26660:26660" or "0.0.0.0:26660"
    if let Some(start) = line.find("26660") {
        return Some("26660");
    }
    if let Some(start) = line.find("26661") {
        return Some("26661");
    }
    None
}

fn discover_faucet_from_logs(devnet_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try docker logs
    let containers = ["accumulate-core", "accumulate-devnet", "devnet"];

    for container in &containers {
        let output = Command::new("docker").args(&["logs", container]).output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            for content in &[stdout.as_ref(), stderr.as_ref()] {
                for line in content.lines() {
                    if line.contains("Faucet account=") || line.contains("faucet") {
                        if let Some(account) = extract_faucet_account(line) {
                            return Ok(account);
                        }
                    }
                }
            }
        }
    }

    Err("Could not find faucet account in logs".into())
}

fn discover_faucet_from_config(devnet_dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Look for config files that might contain faucet account
    let config_paths = [
        "config/genesis.json",
        "config/devnet.json",
        "genesis.json",
        "devnet.json",
        ".env",
        "docker-compose.yml",
    ];

    for config_file in &config_paths {
        let config_path = Path::new(devnet_dir).join(config_file);
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                for line in content.lines() {
                    if line.contains("faucet") || line.contains("ACME") {
                        if let Some(account) = extract_faucet_account(line) {
                            return Ok(account);
                        }
                    }
                }
            }
        }
    }

    Err("Could not find faucet account in config files".into())
}

fn extract_faucet_account(line: &str) -> Option<String> {
    // Look for patterns like "acc://faucet.acme/ACME" or "Faucet account=acc://..."
    if let Some(start) = line.find("acc://") {
        let remainder = &line[start..];
        if let Some(end) = remainder.find(char::is_whitespace) {
            return Some(remainder[..end].to_string());
        } else {
            // Take the rest of the line if no whitespace found
            return Some(remainder.to_string());
        }
    }
    None
}

fn test_devnet_connectivity(config: &DevNetConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing V2 API: {}", config.rpc_url_v2);

    // Simple HTTP test - just check if we can connect
    let client = std::sync::Arc::new(
        reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()?,
    );

    // Test V2 endpoint
    let v2_status_url = format!("{}/status", config.rpc_url_v2);
    match client.get(&v2_status_url).send() {
        Ok(response) => {
            println!("   ✅ V2 API responding (status: {})", response.status());
        }
        Err(e) => {
            println!("   ⚠️  V2 API not responding: {}", e);
            println!("   💡 Make sure DevNet is running: docker-compose up -d");
        }
    }

    // Test V3 endpoint
    println!("🔗 Testing V3 API: {}", config.rpc_url_v3);
    let v3_status_url = format!("{}/status", config.rpc_url_v3);
    match client.get(&v3_status_url).send() {
        Ok(response) => {
            println!("   ✅ V3 API responding (status: {})", response.status());
        }
        Err(e) => {
            println!("   ⚠️  V3 API not responding: {}", e);
        }
    }

    Ok(())
}

fn write_env_file(config: &DevNetConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Writing environment configuration to {}", ENV_FILE);

    let env_content = format!(
        "# Accumulate DevNet Configuration\n\
         # Generated by devnet_discovery on {}\n\n\
         ACC_DEVNET_DIR={}\n\
         ACC_RPC_URL_V2={}\n\
         ACC_RPC_URL_V3={}\n\
         ACC_FAUCET_ACCOUNT={}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        config.devnet_dir,
        config.rpc_url_v2,
        config.rpc_url_v3,
        config.faucet_account
    );

    fs::write(ENV_FILE, env_content)?;

    println!("   ✅ Environment file written successfully");
    println!("\n📋 Discovered Configuration:");
    println!("   ACC_DEVNET_DIR={}", config.devnet_dir);
    println!("   ACC_RPC_URL_V2={}", config.rpc_url_v2);
    println!("   ACC_RPC_URL_V3={}", config.rpc_url_v3);
    println!("   ACC_FAUCET_ACCOUNT={}", config.faucet_account);

    Ok(())
}
