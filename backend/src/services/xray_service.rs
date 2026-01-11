use crate::models::xray_config::{
    ApiConfig, InboundConfig, LogConfig, OutboundConfig, RoutingConfig, RoutingRule, XrayConfig,
};
use crate::services::system_service::{self, SharedMonitor};
use axum::async_trait;
use sqlx::SqlitePool;
use std::env;

#[async_trait]
pub trait XrayService {
    async fn generate_config(pool: &SqlitePool) -> crate::errors::ApiResult<XrayConfig>;
}

pub async fn apply_config(pool: &SqlitePool, monitor: SharedMonitor) -> crate::errors::ApiResult<()> {
    let inbounds = sqlx::query_as::<_, crate::models::inbound::Inbound>("SELECT * FROM inbounds")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            crate::errors::ApiError::InternalError(format!("Failed to fetch inbounds: {}", e))
        })?;

    let mut config = XrayConfig {
        log: LogConfig::default(),
        api: ApiConfig {
            tag: "api".to_string(),
            services: vec!["HandlerService".to_string(), "StatsService".to_string()],
        },
        dns: None,
        stats: None,
        policy: None,
        inbounds: vec![],
        outbounds: vec![],
        routing: None,
    };

    let mut inbound_configs = Vec::new();

    for inbound in inbounds {
        let clients_json = inbound
            .settings
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v.get("clients").cloned())
            .unwrap_or_else(|| serde_json::json!([]));

        let sniffing_json = inbound
            .sniffing
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .unwrap_or_else(|| serde_json::json!({ "enabled": false, "destOverride": ["tls", "http"] }));

        let settings = serde_json::json!({
            "clients": clients_json,
            "decryption": "none",
            "sniffing": sniffing_json
        });

        let mut stream_settings_json = inbound
            .stream_settings
            .as_ref()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok());

        if let Some(ref mut ss) = stream_settings_json {
            if let Some(ss_obj) = ss.as_object_mut() {
                if ss_obj.get("network").and_then(|n| n.as_str()) == Some("xhttp") {
                    ss_obj.insert("network".to_string(), serde_json::json!("tcp"));
                }

                if let Some(reality) = ss_obj.get_mut("realitySettings") {
                    if let Some(reality_obj) = reality.as_object_mut() {
                        if reality_obj.get("serverNames").is_none() {
                            let names = reality_obj.get("serverName")
                                .and_then(|v| v.as_str())
                                .map(|s| if s.is_empty() { vec![] } else { vec![s.to_string()] })
                                .unwrap_or_default();
                            reality_obj.insert("serverNames".to_string(), serde_json::json!(names));
                        }
                        if reality_obj.get("shortIds").is_none() {
                            let ids = reality_obj.get("shortId")
                                .and_then(|v| v.as_str())
                                .map(|s| if s.is_empty() { vec![] } else { vec![s.to_string()] })
                                .unwrap_or_default();
                            reality_obj.insert("shortIds".to_string(), serde_json::json!(ids));
                        }
                        if reality_obj.get("fingerprint").is_none() {
                            reality_obj.insert("fingerprint".to_string(), serde_json::json!("chrome"));
                        }
                        // Clean up
                        reality_obj.remove("serverName");
                        reality_obj.remove("shortId");
                    }
                }

                if let Some(xhttp) = ss_obj.get_mut("xhttpSettings") {
                    if let Some(xhttp_obj) = xhttp.as_object_mut() {
                        if xhttp_obj.get("mode").and_then(|m| m.as_str()) == Some("packet-up") {
                            xhttp_obj.insert("mode".to_string(), serde_json::json!("auto"));
                        }
                    }
                }
            }
        }

        let listen_addr = inbound.listen.as_ref()
            .map(|s| if s.is_empty() { "0.0.0.0".to_string() } else { s.clone() })
            .unwrap_or_else(|| "0.0.0.0".to_string());

        inbound_configs.push(InboundConfig {
            tag: inbound.tag.clone().unwrap_or_else(|| format!("inbound-{}", inbound.id)),
            port: inbound.port,
            protocol: inbound.protocol.clone(),
            listen: Some(listen_addr),
            allocate: None,
            settings: Some(settings),
            stream_settings: stream_settings_json,
            sniffing: None,
        });
    }

    config.inbounds = inbound_configs;

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

    config.routing = Some(RoutingConfig {
        domain_strategy: "IPIfNonMatch".to_string(),
        rules: vec![],
    });

    let config_json = serde_json::to_string_pretty(&config).map_err(|e| {
        crate::errors::ApiError::InternalError(format!("Failed to serialize config: {}", e))
    })?;

    let config_path = env::var("XRAY_CONFIG_PATH").unwrap_or_else(|_| "data/xray.json".to_string());

    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        if !parent.exists() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                crate::errors::ApiError::SystemError(format!("Failed to create config directory: {}", e))
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
