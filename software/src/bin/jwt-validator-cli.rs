// ABOUTME: Command-line JWT validator for cross-language TDD testing with Java and TypeScript
// ABOUTME: Minimal CLI implementation to make cross-language validation test pass

use std::env;
use std::fs;
use std::process;

use qollective::security::{SecurityConfig, SimpleJwtValidator};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: jwt-validator-cli <config_file> <token_file>");
        process::exit(1);
    }

    let config_file = &args[1];
    let token_file = &args[2];

    // Read the security config JSON
    let config_content = match fs::read_to_string(config_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("ERROR: Failed to read config file - {}", e);
            process::exit(1);
        }
    };

    let config: SecurityConfig = match serde_json::from_str(&config_content) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("ERROR: Failed to parse config JSON - {}", e);
            process::exit(1);
        }
    };

    // Read the JWT token
    let jwt_token = match fs::read_to_string(token_file) {
        Ok(content) => content.trim().to_string(),
        Err(e) => {
            eprintln!("ERROR: Failed to read token file - {}", e);
            process::exit(1);
        }
    };

    // Create validator with config settings
    let verify_signature = config.jwt_validation.verify_signature;
    let verify_expiry = config.jwt_validation.verify_expiry;
    let validator = SimpleJwtValidator::new(verify_signature, verify_expiry);

    // Validate the token
    match validator.validate(&jwt_token).await {
        Ok(validated_token) => {
            println!("SUCCESS: VALID JWT token");
            println!("Subject: {}", validated_token.subject);
            println!(
                "Claims: {}",
                serde_json::to_string(&validated_token.claims).unwrap_or_else(|_| "{}".to_string())
            );
        }
        Err(e) => {
            eprintln!("ERROR: JWT validation failed - {}", e);
            process::exit(1);
        }
    }
}
