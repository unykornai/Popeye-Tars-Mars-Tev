//! Unykorn L1 Node Binary
//!
//! The main entrypoint for running an Unykorn L1 blockchain node.

use node::{Node, NodeConfig};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    println!("╔═══════════════════════════════════════════╗");
    println!("║         UNYKORN L1 BLOCKCHAIN             ║");
    println!("║     MARS · POPEYE · TEV · TAR             ║");
    println!("╚═══════════════════════════════════════════╝");
    println!();

    // Parse arguments
    let args: Vec<String> = std::env::args().collect();
    
    let config = if args.len() > 2 && args[1] == "--config" {
        let config_path = PathBuf::from(&args[2]);
        match NodeConfig::load(&config_path) {
            Ok(cfg) => {
                println!("Loaded config from: {:?}", config_path);
                cfg
            }
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                eprintln!("Using default configuration...");
                NodeConfig::default()
            }
        }
    } else if args.contains(&"--dev".to_string()) {
        println!("Running in development mode...");
        NodeConfig::dev()
    } else {
        println!("Using default configuration...");
        NodeConfig::default()
    };

    // Create and run node
    match Node::new(config) {
        Ok(mut node) => {
            println!();
            println!("Node initialized at height {}", node.height());
            println!("Press Ctrl+C to shutdown");
            println!();

            // Handle Ctrl+C
            let shutdown_handle = tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                println!("\nReceived shutdown signal...");
            });

            tokio::select! {
                result = node.run() => {
                    if let Err(e) = result {
                        eprintln!("Node error: {}", e);
                    }
                }
                _ = shutdown_handle => {
                    node.shutdown().await;
                }
            }

            println!("Node shutdown complete.");
        }
        Err(e) => {
            eprintln!("Failed to initialize node: {}", e);
            std::process::exit(1);
        }
    }
}
