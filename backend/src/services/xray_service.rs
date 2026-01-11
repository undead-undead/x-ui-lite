use crate::services::system_service::{self, SharedMonitor};
use axum::async_trait;
use sqlx::SqlitePool;
use std::env;
use serde_json::{json, Value, Map};

#[async_trait]
pub trait XrayService {
    async fn apply_config(pool: &SqlitePool, monitor: SharedMonitor) -> crate::errors::ApiResult<()>;
}

pub async fn apply_config(pool: &SqlitePool, monitor: SharedMonitor) -> crate::errors::ApiResult<()> {
    let inbounds = sqlx::query_as::<_, crate::models::inbound::Inbound>("SELECT * FROM inbounds")
        .fetch_all(pool)
        .await
        .map_err(|e| {
            crate::errors::ApiError::InternalError(format!("Failed to fetch inbounds: {}", e))
        })?;

    // --- Create a pure JSON map for xray-lite ---
    // xray-lite ONLY supports: inbounds, outbounds, routing
    let mut root = Map::new();
    let mut inbound_configs = Vec::new();

    for inbound in inbounds {
        // 1. Prepare Settings
        let clients_raw = inbound.settings.as_ref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .and_then(|v| v.get("clients").cloned())
            .unwrap_or_else(|| json!([]));
        
        let mut clients = Vec::new();
        if let Some(arr) = clients_raw.as_array() {
            for c in arr {
                let mut client = Map::new();
                if let Some(id) = c.get("id").or_else(|| c.get("password")) {
                    client.insert("id".to_string(), id.clone());
                }
                if let Some(email) = c.get("email") {
                    client.insert("email".to_string(), email.clone());
                }
                clients.push(Value::Object(client));
            }
        }

        // Force sniffing configuration (Mandatory for Reality SNI)
        let sniffing = json!({
            "enabled": true,
            "destOverride": ["tls", "http"]
        });

        let settings = json!({
            "clients": clients,
            "decryption": "none",
            "sniffing": sniffing
        });

        // 2. Prepare Stream Settings
        let stream_settings_raw = inbound.stream_settings.as_ref()
            .and_then(|s| serde_json::from_str::<Value>(s).ok())
            .unwrap_or_else(|| json!({ "network": "tcp", "security": "none" }));
        
        let mut ss_obj = stream_settings_raw.as_object().cloned().unwrap_or_default();

        // Normalize network
        let network = ss_obj.get("network").and_then(|n| n.as_str()).unwrap_or("tcp");
        let safe_network = if network == "xhttp" { "tcp" } else { network };
        ss_obj.insert("network".to_string(), json!(safe_network));

        // Reality Strict Normalization
        if ss_obj.get("security").and_then(|s| s.as_str()) == Some("reality") {
            if let Some(rs_val) = ss_obj.get("realitySettings") {
                let mut rs_new = Map::new();
                
                // Duplication for absolute compatibility with xray-lite's lack of serde rename attributes
                let dest = rs_val.get("dest").cloned().unwrap_or(json!("www.microsoft.com:443"));
                let priv_key = rs_val.get("privateKey").cloned().or_else(|| rs_val.get("private_key").cloned()).unwrap_or(json!(""));
                let pub_key = rs_val.get("publicKey").cloned().or_else(|| rs_val.get("public_key").cloned()).unwrap_or(Value::Null);
                let fp = rs_val.get("fingerprint").cloned().unwrap_or(json!("chrome"));

                rs_new.insert("dest".to_string(), dest);
                rs_new.insert("privateKey".to_string(), priv_key.clone());
                rs_new.insert("private_key".to_string(), priv_key); // Standard snake_case
                rs_new.insert("publicKey".to_string(), pub_key.clone());
                rs_new.insert("public_key".to_string(), pub_key); // Standard snake_case
                rs_new.insert("fingerprint".to_string(), fp);

                // serverNames
                let sn = rs_val.get("serverNames").or_else(|| rs_val.get("serverName")).or_else(|| rs_val.get("server_names"));
                let server_names = if let Some(sn_val) = sn {
                    if sn_val.is_array() { sn_val.clone() }
                    else if let Some(s) = sn_val.as_str() { if s.is_empty() { json!([]) } else { json!([s]) } }
                    else { json!([]) }
                } else { json!([]) };
                rs_new.insert("serverNames".to_string(), server_names.clone());
                rs_new.insert("server_names".to_string(), server_names); // Standard snake_case

                // shortIds
                let si = rs_val.get("shortIds").or_else(|| rs_val.get("shortId")).or_else(|| rs_val.get("short_ids"));
                let short_ids = if let Some(si_val) = si {
                    if si_val.is_array() { si_val.clone() }
                    else if let Some(s) = si_val.as_str() { if s.is_empty() { json!([]) } else { json!([s]) } }
                    else { json!([]) }
                } else { json!([]) };
                rs_new.insert("shortIds".to_string(), short_ids.clone());
                rs_new.insert("short_ids".to_string(), short_ids); // Standard snake_case

                ss_obj.insert("realitySettings".to_string(), Value::Object(rs_new));
            }
        }

        // XHTTP Strict Normalization
        if let Some(xh_val) = ss_obj.get("xhttpSettings") {
            let mut xh_new = Map::new();
            let mode = xh_val.get("mode").and_then(|m| m.as_str()).unwrap_or("auto");
            xh_new.insert("mode".to_string(), if mode == "packet-up" { json!("auto") } else { json!(mode) });
            xh_new.insert("path".to_string(), xh_val.get("path").cloned().unwrap_or(json!("/")));
            xh_new.insert("host".to_string(), xh_val.get("host").cloned().unwrap_or(json!("")));
            ss_obj.insert("xhttpSettings".to_string(), Value::Object(xh_new));
        }

        // Sockopt
        let sockopt = json!({
            "tcpFastOpen": true,
            "tcpNoDelay": true,
            "acceptProxyProtocol": false
        });
        ss_obj.insert("sockopt".to_string(), sockopt);

        // 3. Build Inbound (ONLY fields supported by xray-lite)
        let mut inbound_map = Map::new();
        inbound_map.insert("port".to_string(), json!(inbound.port));
        inbound_map.insert("protocol".to_string(), json!(inbound.protocol.to_lowercase()));
        inbound_map.insert("listen".to_string(), json!(inbound.listen.as_ref().filter(|s| !s.is_empty()).unwrap_or(&"0.0.0.0".to_string())));
        inbound_map.insert("settings".to_string(), settings);
        inbound_map.insert("streamSettings".to_string(), Value::Object(ss_obj));

        inbound_configs.push(Value::Object(inbound_map));
    }

    root.insert("inbounds".to_string(), Value::Array(inbound_configs));
    root.insert("outbounds".to_string(), json!([
        { "tag": "direct", "protocol": "freedom" },
        { "tag": "blocked", "protocol": "blackhole" }
    ]));
    root.insert("routing".to_string(), json!({
        "rules": []
    }));

    let config_json = serde_json::to_string_pretty(&Value::Object(root)).map_err(|e| {
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
