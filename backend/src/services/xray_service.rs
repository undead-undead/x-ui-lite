use crate::errors::ApiResult;
use crate::models::inbound::Inbound;
use crate::models::xray_config::*;
use crate::services::system_service;
use crate::services::system_service::SharedMonitor;
use sqlx::SqlitePool;
use std::env;

pub async fn apply_config(pool: &SqlitePool, monitor: SharedMonitor) -> ApiResult<()> {
    let inbounds = sqlx::query_as::<_, Inbound>("SELECT * FROM inbounds WHERE enable = 1")
        .fetch_all(pool)
        .await?;

    let mut config = XrayConfig::default();

    // Simplified logging for xray-lite
    config.log.loglevel = "info".to_string();
    let cwd = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let log_dir = cwd.join("logs");
    if !log_dir.exists() {
        let _ = std::fs::create_dir_all(&log_dir);
    }

    config.log.access = Some(log_dir.join("access.log").to_string_lossy().to_string());
    config.log.error = Some(log_dir.join("error.log").to_string_lossy().to_string());

    // xray-lite doesn't need API configuration
    // Removed API inbound and services

    for inbound in inbounds {
        let tag = inbound
            .tag
            .clone()
            .unwrap_or_else(|| format!("inbound-{}", inbound.id));

        let allocate = inbound
            .allocate
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok());

        // xray-lite requires "listen" field to be present
        let listen = inbound.listen.clone().unwrap_or_else(|| "0.0.0.0".to_string());

        // Handle sniffing: move it into settings for xray-lite compliance
        // And merge existing settings
        let mut settings_json = inbound
            .settings
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .unwrap_or(serde_json::json!({}));
        
        let sniffing_json = inbound
            .sniffing
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());
            
        if let Some(obj) = settings_json.as_object_mut() {
            // Ensure clients field exists (required by xray-lite)
            if !obj.contains_key("clients") {
                obj.insert("clients".to_string(), serde_json::json!([]));
            }
            // Ensure decryption field exists
            if !obj.contains_key("decryption") {
                obj.insert("decryption".to_string(), serde_json::json!("none"));
            }

            if let Some(sniffing) = sniffing_json {
                obj.insert("sniffing".to_string(), sniffing);
            }
        }

        // Handle stream_settings: fix compatibility issues for xray-lite
        let mut stream_settings_json = inbound
            .stream_settings
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());

        if let Some(ref mut ss) = stream_settings_json {
            // Fix network: xhttp -> tcp (xray-lite doesn't have xhttp enum variant)
            if let Some(network) = ss.get("network").and_then(|n| n.as_str()) {
                if network == "xhttp" {
                    if let Some(ss_obj) = ss.as_object_mut() {
                        ss_obj.insert("network".to_string(), serde_json::json!("tcp"));
                    }
                }
            }

            // Fix realitySettings: serverName -> serverNames
            if let Some(reality) = ss.get_mut("realitySettings") {
                if let Some(reality_obj) = reality.as_object_mut() {
                    // Check if serverNames is missing but serverName exists
                    if reality_obj.get("serverNames").is_none() {
                        if let Some(server_name) = reality_obj.get("serverName") {
                            // Convert single serverName to array of serverNames
                            if let Some(name) = server_name.as_str() {
                                reality_obj.insert("serverNames".to_string(), serde_json::json!([name]));
                            }
                        }
                    }
                    
                    // Also check shortIds
                    if reality_obj.get("shortIds").is_none() {
                         if let Some(short_id) = reality_obj.get("shortId") {
                            if let Some(id) = short_id.as_str() {
                                reality_obj.insert("shortIds".to_string(), serde_json::json!([id]));
                            }
                        } else {
                            // Ensure shortIds exists as empty array if missing
                            reality_obj.insert("shortIds".to_string(), serde_json::json!([]));
                        }
                    }
                }
            }
        }

        let inbound_config = InboundConfig {
            tag,
            port: inbound.port,
            protocol: inbound.protocol.clone(),
            listen: Some(listen), // Ensure listen is set
            allocate,
            settings: Some(settings_json), // Settings now includes sniffing
            stream_settings: stream_settings_json,
            sniffing: None, // Remove top-level sniffing (it's now in settings)
        };


        config.inbounds.push(inbound_config);
    }

    // xray-lite doesn't need stats and policy
    // Removed stats and policy configuration

    config.outbounds.push(OutboundConfig {
        tag: "direct".to_string(),
        protocol: "freedom".to_string(),
        settings: None,
        stream_settings: None,
    });

    config.outbounds.push(OutboundConfig {
        tag: "blocked".to_string(),
        protocol: "blackhole".to_string(),
        settings: None,
        stream_settings: None,
    });

    // Simplified routing - no need for API routing
    config.routing = Some(RoutingConfig {
        domain_strategy: "IPIfNonMatch".to_string(),
        rules: vec![],
    });

    let config_json = serde_json::to_string_pretty(&config).map_err(|e| {
        crate::errors::ApiError::InternalError(format!("Failed to serialize config: {}", e))
    })?;

    let config_path =
        env::var("XRAY_CONFIG_PATH").unwrap_or_else(|_| "/etc/x-ui/xray.json".to_string());

    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                crate::errors::ApiError::SystemError(format!(
                    "Failed to create config directory: {}",
                    e
                ))
            })?;
        }
    }

    tokio::fs::write(&config_path, config_json)
        .await
        .map_err(|e| {
            crate::errors::ApiError::SystemError(format!("Failed to write config file: {}", e))
        })?;

    tracing::info!("xray-lite config generated at: {}", config_path);

    tokio::spawn(async move {
        if let Err(e) = system_service::restart_xray(monitor).await {
            tracing::error!("Background xray-lite restart failed: {:?}", e);
        } else {
            tracing::info!("Background xray-lite restart successful");
        }
    });

    Ok(())
}

impl Default for RoutingRule {
    fn default() -> Self {
        Self {
            rule_type: "field".to_string(),
            port: None,
            inbound_tag: None,
            outbound_tag: None,
            ip: None,
            domain: None,
            protocol: None,
        }
    }
}
