use crate::errors::ApiResult;
use crate::models::inbound::Inbound;
use crate::services::system_service::SharedMonitor;
use crate::services::xray_service;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::process::Command;
use tokio::time::{interval, Duration};

pub fn start_traffic_stats_task(pool: SqlitePool, monitor: SharedMonitor) {
    tracing::info!("Starting traffic stats collector for xray-lite (Iptables Kernel Mode)");
    
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        let mut last_counters: HashMap<String, (u64, u64)> = HashMap::new();

        loop {
            interval.tick().await;
            
            if let Err(e) = process_iptables_traffic(&pool, monitor.clone(), &mut last_counters).await {
                tracing::error!("Error processing iptables traffic: {}", e);
            }
        }
    });
}

async fn process_iptables_traffic(
    pool: &SqlitePool,
    monitor: SharedMonitor,
    last_counters: &mut HashMap<String, (u64, u64)>,
) -> ApiResult<()> {
    // 1. Sync Rules
    sync_iptables_rules(pool).await?;

    // 2. Read Stats
    let current_stats = read_iptables_stats()?;
    
    let mut needs_reapply = false;

    // 3. Update DB with deltas
    for (tag, (current_in, current_out)) in current_stats {
        let (last_in, last_out) = last_counters.get(&tag).cloned().unwrap_or((0, 0));
        
        // Calculate deltas (handle counter resets)
        let delta_in = if current_in >= last_in { current_in - last_in } else { current_in };
        let delta_out = if current_out >= last_out { current_out - last_out } else { current_out };
        
        last_counters.insert(tag.clone(), (current_in, current_out));

        if delta_in > 0 || delta_out > 0 {
            let traffic_data = TrafficData {
                tag,
                up: delta_in as i64,   // Client UP = Server IN
                down: delta_out as i64, // Client DOWN = Server OUT
            };
            
            if let Err(e) = update_db_traffic(pool, &traffic_data, &mut needs_reapply).await {
                tracing::error!("Failed to update traffic for tag {}: {}", traffic_data.tag, e);
            }
        }
    }

    if needs_reapply {
        tracing::info!("Traffic limit reached for some nodes, reapplying config...");
        if let Err(e) = xray_service::apply_config(pool, monitor).await {
            tracing::error!("Failed to reapply config after quota reached: {}", e);
        }
    }

    Ok(())
}

struct TrafficData {
    tag: String,
    up: i64,
    down: i64,
}

async fn sync_iptables_rules(pool: &SqlitePool) -> ApiResult<()> {
    // Get enabled inbounds
    let inbounds = sqlx::query_as::<_, Inbound>("SELECT * FROM inbounds WHERE enable = 1")
        .fetch_all(pool)
        .await
        .map_err(|e| crate::errors::ApiError::InternalError(format!("DB error: {}", e)))?;

    // Create chains if they don't exist
    let _ = Command::new("iptables").args(["-N", "XUI_IN"]).output();
    let _ = Command::new("iptables").args(["-N", "XUI_OUT"]).output();

    // Ensure jump rules exist (insert at top only once)
    ensure_jump_rule("INPUT", "XUI_IN")?;
    ensure_jump_rule("OUTPUT", "XUI_OUT")?;

    // Get current rules to avoid duplicates
    let current_rules_in = get_chain_rules("XUI_IN")?;
    let current_rules_out = get_chain_rules("XUI_OUT")?;

    for inbound in inbounds {
        let tag = inbound.tag.as_ref().filter(|s| !s.is_empty()).cloned().unwrap_or_else(|| format!("inbound-{}", inbound.id));
        let port = inbound.port;
        let comment = format!("xui-{}", tag);

        // INPUT Rules (Downlink for server)
        if !current_rules_in.contains(&comment) {
            let _ = Command::new("iptables").args([
                "-A", "XUI_IN", "-p", "tcp", "--dport", &port.to_string(),
                "-j", "RETURN", "-m", "comment", "--comment", &comment
            ]).status();
            let _ = Command::new("iptables").args([
                "-A", "XUI_IN", "-p", "udp", "--dport", &port.to_string(),
                "-j", "RETURN", "-m", "comment", "--comment", &comment
            ]).status();
        }

        // OUTPUT Rules (Uplink for server)
        if !current_rules_out.contains(&comment) {
            let _ = Command::new("iptables").args([
                "-A", "XUI_OUT", "-p", "tcp", "--sport", &port.to_string(),
                "-j", "RETURN", "-m", "comment", "--comment", &comment
            ]).status();
            let _ = Command::new("iptables").args([
                "-A", "XUI_OUT", "-p", "udp", "--sport", &port.to_string(),
                "-j", "RETURN", "-m", "comment", "--comment", &comment
            ]).status();
        }
    }

    Ok(())
}

fn ensure_jump_rule(base_chain: &str, target_chain: &str) -> ApiResult<()> {
    let output = Command::new("iptables").args(["-C", base_chain, "-j", target_chain]).output().ok();
    let exists = output.map(|o| o.status.success()).unwrap_or(false);
    
    if !exists {
        let _ = Command::new("iptables").args(["-I", base_chain, "1", "-j", target_chain]).status();
    }
    Ok(())
}

fn get_chain_rules(chain: &str) -> ApiResult<Vec<String>> {
    let output = Command::new("iptables").args(["-S", chain]).output().map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Iptables -S failed: {}", e))
    })?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut comments = Vec::new();
    for line in stdout.lines() {
        if let Some(pos) = line.find("--comment ") {
            let comment = line[pos + 10..].trim_matches('\"').replace("\"", "");
            comments.push(comment);
        }
    }
    Ok(comments)
}

fn read_iptables_stats() -> ApiResult<HashMap<String, (u64, u64)>> {
    let mut stats: HashMap<String, (u64, u64)> = HashMap::new();

    // Read INPUT (Down)
    parse_chain_stats("XUI_IN", &mut stats, true)?;
    // Read OUTPUT (Up)
    parse_chain_stats("XUI_OUT", &mut stats, false)?;

    Ok(stats)
}

fn parse_chain_stats(chain: &str, stats: &mut HashMap<String, (u64, u64)>, is_in: bool) -> ApiResult<()> {
    let output = Command::new("iptables").args(["-L", chain, "-v", "-n", "-x"]).output().map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Iptables -L failed: {}", e))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("/* xui-") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 { continue; }
            
            let bytes = parts[1].parse::<u64>().unwrap_or(0);
            if let Some(comment_pos) = line.find("/* xui-") {
                let tag = line[comment_pos + 7..].trim_end_matches(" */").trim().to_string();
                let entry = stats.entry(tag).or_insert((0, 0));
                if is_in {
                    entry.0 += bytes;
                } else {
                    entry.1 += bytes;
                }
            }
        }
    }
    Ok(())
}

async fn update_db_traffic(
    pool: &SqlitePool,
    data: &TrafficData,
    needs_reapply: &mut bool,
) -> ApiResult<()> {
    // 1. Update traffic atomically and check quota
    sqlx::query(
        r#"
        UPDATE inbounds 
        SET up = up + ?, 
            down = down + ?,
            enable = CASE 
                WHEN total > 0 AND (up + down + ? + ?) >= total THEN 0 
                ELSE enable 
            END
        WHERE tag = ?
        "#
    )
    .bind(data.up)
    .bind(data.down)
    .bind(data.up)
    .bind(data.down)
    .bind(&data.tag)
    .execute(pool)
    .await.map_err(|e| crate::errors::ApiError::InternalError(format!("Update DB failed: {}", e)))?;

    // 2. Check if we just disabled any node to trigger config reapply
    let inbound = sqlx::query_as::<_, Inbound>("SELECT * FROM inbounds WHERE tag = ?")
        .bind(&data.tag)
        .fetch_optional(pool)
        .await.map_err(|e| crate::errors::ApiError::InternalError(format!("Fetch DB failed: {}", e)))?
        .ok_or_else(|| crate::errors::ApiError::InternalError(format!("Inbound tag {} not found", data.tag)))?;

    if !inbound.enable {
        *needs_reapply = true;
    }

    Ok(())
}
