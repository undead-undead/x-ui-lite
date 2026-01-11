use crate::models::xray_config::{
    ApiConfig, InboundConfig, LogConfig, OutboundConfig, RoutingConfig, RoutingRule, XrayConfig,
};
use crate::services::system_service::{self, SharedMonitor};
use axum::async_trait;
use sqlx::SqlitePool;
use std::env;
use serde_json::{json, Value};

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
        // --- 1. Clients & Settings ---
        let clients_raw = inbound.settings.as_ref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v.get("clients").cloned())
            .unwrap_or_else(|| json!([]));
        
        // Clean up clients (ensure only ID is present if it's VLESS)
        let mut clients = Vec::new();
        if let Some(arr) = clients_raw.as_array() {
            for c in arr {
                let mut client = json!({});
                if let Some(id) = c.get("id").or_else(|| c.get("password")) {
                    client["id"] = id.clone();
                }
                if let Some(email) = c.get("email") {
                    client["email"] = email.clone();
                }
                clients.push(client);
            }
        }

        let sniffing_json = inbound.sniffing.as_ref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .unwrap_or_else(|| json!({ "enabled": false, "destOverride": ["tls", "http"] }));

        let settings = json!({
            "clients": clients,
            "decryption": "none",
            "sniffing": sniffing_json
        });

        // --- 2. Stream Settings ---
        let stream_settings_raw = inbound.stream_settings.as_ref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .unwrap_or_else(|| json!({ "network": "tcp", "security": "none" }));
        
        let mut ss_obj = stream_settings_raw.as_object().cloned().unwrap_or_default();

        // Normalize network
        let network = ss_obj.get("network").and_then(|n| n.as_str()).unwrap_or("tcp");
        let safe_network = if network == "xhttp" { "tcp" } else { network };
        ss_obj.insert("network".to_string(), json!(safe_network));

        // Reality Normalization (Extremely strict for xray-lite)
        if ss_obj.get("security").and_then(|s| s.as_str()) == Some("reality") {
            if let Some(rs_val) = ss_obj.get("realitySettings") {
                let mut rs_new = json!({});
                
                // Only take what xray-lite supports
                rs_new["dest"] = rs_val.get("dest").cloned().unwrap_or(json!("www.microsoft.com:443"));
                rs_new["privateKey"] = rs_val.get("privateKey").cloned().or_else(|| rs_val.get("private_key").cloned()).unwrap_or(json!(""));
                rs_new["publicKey"] = rs_val.get("publicKey").cloned().or_else(|| rs_val.get("public_key").cloned()).unwrap_or(Value::Null);
                rs_new["fingerprint"] = rs_val.get("fingerprint").cloned().unwrap_or(json!("chrome"));

                // serverNames (Array required)
                let sn = rs_val.get("serverNames").or_else(|| rs_val.get("serverName"));
                rs_new["serverNames"] = if let Some(sn_val) = sn {
                    if sn_val.is_array() { sn_val.clone() }
                    else if let Some(s) = sn_val.as_str() { if s.is_empty() { json!([]) } else { json!([s]) } }
                    else { json!([]) }
                } else { json!([]) };

                // shortIds (Array required)
                let si = rs_val.get("shortIds").or_else(|| rs_val.get("shortId"));
                rs_new["shortIds"] = if let Some(si_val) = si {
                    if si_val.is_array() { si_val.clone() }
                    else if let Some(s) = si_val.as_str() { if s.is_empty() { json!([]) } else { json!([s]) } }
                    else { json!([]) }
                } else { json!([]) };

                ss_obj.insert("realitySettings".to_string(), rs_new);
            }
        }

        // XHTTP Normalization
        if let Some(xh_val) = ss_obj.get("xhttpSettings") {
            let mut xh_new = json!({});
            let mode = xh_val.get("mode").and_then(|m| m.as_str()).unwrap_or("auto");
            xh_new["mode"] = if mode == "packet-up" { json!("auto") } else { json!(mode) };
            xh_new["path"] = xh_val.get("path").cloned().unwrap_or(json!("/"));
            xh_new["host"] = xh_val.get("host").cloned().unwrap_or(json!(""));
            ss_obj.insert("xhttpSettings".to_string(), xh_new);
        }

        // Sockopt Normalization
        let mut sockopt = json!({
            "tcpFastOpen": true,
            "tcpNoDelay": true,
            "acceptProxyProtocol": false
        });
        if let Some(so_val) = ss_obj.get("sockopt") {
            if let Some(b) = so_val.get("tcpFastOpen").and_then(|v| v.as_bool()) { sockopt["tcpFastOpen"] = json!(b); }
            if let Some(b) = so_val.get("tcpNoDelay").and_then(|v| v.as_bool()) { sockopt["tcpNoDelay"] = json!(b); }
            if let Some(b) = so_val.get("acceptProxyProtocol").and_then(|v| v.as_bool()) { sockopt["acceptProxyProtocol"] = json!(b); }
        }
        ss_obj.insert("sockopt".to_string(), sockopt);

        // --- 3. Assemble Inbound ---
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
            stream_settings: Some(json!(ss_obj)),
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

    let config_path = env::var("XRAY_CONFIG_PATH").unwrap_or_else(|_| "/usr/local/x-ui/data/xray.json".to_string());

    if let Some(parent) = std::path::Path::new(&config_path).parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    tokio::fs::write(&config_path, config_json).await.map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Failed to write config file: {}", e))
    })?;

    tracing::info!("xray-lite config generated at: {}", config_path);
    tokio::spawn(async move {
        let _ = system_service::restart_xray(monitor).await;
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
