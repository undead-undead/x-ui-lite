use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RealityKeysResponse {
    pub private_key: String,
    pub public_key: String,
}

pub async fn generate_reality_keys() -> Result<Json<RealityKeysResponse>, StatusCode> {
    // Use xray-lite's keygen binary
    let xray_bin =
        std::env::var("XRAY_BIN_PATH").unwrap_or_else(|_| "/usr/local/x-ui/bin/xray".to_string());
    
    // xray-lite has a separate keygen binary
    // Try to find it in the same directory as the xray binary
    let xray_path = std::path::Path::new(&xray_bin);
    let keygen_bin = if let Some(parent) = xray_path.parent() {
        parent.join("keygen")
    } else {
        std::path::PathBuf::from("/usr/local/x-ui/bin/keygen")
    };
    
    // If keygen doesn't exist, fall back to using the xray binary itself
    // (in case it's actually xray-core which has x25519 command)
    let (command, args): (String, Vec<&str>) = if keygen_bin.exists() {
        (keygen_bin.to_string_lossy().to_string(), vec![])
    } else {
        (xray_bin.clone(), vec!["x25519"])
    };

    let output = std::process::Command::new(&command)
        .args(&args)
        .output()
        .map_err(|e| {
            tracing::error!("Failed to execute key generation: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if !output.status.success() {
        tracing::error!(
            "Key generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    tracing::info!("Key generation output: {}", stdout);

    let mut private_key = String::new();
    let mut public_key = String::new();

    // Parse xray-lite keygen format:
    // Private key: xxxxx
    // Public key: xxxxx
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Private key:") {
            private_key = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if trimmed.starts_with("Public key:") {
            public_key = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
        }
        // Also handle xray-core format for backward compatibility
        else if trimmed.starts_with("PrivateKey:") {
            private_key = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if trimmed.starts_with("Password:") {
            public_key = trimmed.split(':').nth(1).unwrap_or("").trim().to_string();
        }
    }

    if private_key.is_empty() || public_key.is_empty() {
        tracing::error!(
            "Failed to parse key generation output. Private: '{}', Public: '{}'",
            private_key,
            public_key
        );
        tracing::error!("Raw output was: {}", stdout);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(RealityKeysResponse {
        private_key,
        public_key,
    }))
}
