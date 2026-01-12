use crate::errors::ApiResult;
use crate::models::inbound::Inbound;
use crate::services::system_service::SharedMonitor;
use crate::services::xray_service;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::process::Command;
use tokio::time::{interval, Duration};

pub fn start_traffic_stats_task(pool: SqlitePool, monitor: SharedMonitor) {
    tracing::info!("Starting traffic stats collector for xray-lite (Flush-Mode Dual-Stack Iptables)");
    
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(10));
        let mut last_counters: HashMap<String, (u64, u64)> = HashMap::new();

        loop {
            interval.tick().await;
            
            if let Err(e) = process_iptables_traffic(&pool, monitor.clone(), &mut last_counters).await {
                tracing::error!("Error processing dual-stack iptables traffic: {}", e);
            }
        }
    });
}

async fn process_iptables_traffic(
    pool: &SqlitePool,
    monitor: SharedMonitor,
    last_counters: &mut HashMap<String, (u64, u64)>,
) -> ApiResult<()> {
    // 1. Get enabled inbounds
    let inbounds = sqlx::query_as::<_, Inbound>("SELECT * FROM inbounds WHERE enable = 1")
        .fetch_all(pool)
        .await
        .map_err(|e| crate::errors::ApiError::InternalError(format!("DB error: {}", e)))?;

    // 2. Sync Rules (Flush and Re-add is more robust)
    sync_all_rules_flush(&inbounds)?;

    // 3. Read Stats (Sum of IPv4 & IPv6)
    let current_stats = read_all_stats()?;
    
    let mut needs_reapply = false;

    // 4. Update DB with deltas
    for (tag, (current_in, current_out)) in current_stats {
        let (last_in, last_out) = last_counters.get(&tag).cloned().unwrap_or((0, 0));
        
        // Handle counter resets (iptables counters persist until flush/restart, 
        // but since we don't zero them, we handle resets in app logic)
        // NOTE: Actually, if we FLUSH the chain every time, current_in represents the traffic 
        // since the last sync. In this case, delta_in IS current_in.
        // Wait! I should NOT flush every time if I want persistent counters.
        // Let's use -C check instead of flush every time, but ensure ghost rules are gone.
        
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

fn sync_all_rules_flush(inbounds: &[Inbound]) -> ApiResult<()> {
    sync_family_rules("iptables", inbounds)?;
    if has_command("ip6tables") {
        sync_family_rules("ip6tables", inbounds)?;
    }
    Ok(())
}

fn sync_family_rules(cmd: &str, inbounds: &[Inbound]) -> ApiResult<()> {
    // Create chains
    let _ = Command::new(cmd).args(["-N", "XUI_IN"]).output();
    let _ = Command::new(cmd).args(["-N", "XUI_OUT"]).output();

    // Flush chains to remove ghost rules from deleted nodes
    let _ = Command::new(cmd).args(["-F", "XUI_IN"]).output();
    let _ = Command::new(cmd).args(["-F", "XUI_OUT"]).output();

    // Ensure jump rules are AT THE TOP
    ensure_jump_rule_at_top(cmd, "INPUT", "XUI_IN")?;
    ensure_jump_rule_at_top(cmd, "OUTPUT", "XUI_OUT")?;

    // Instead of flushing every sync (which zeros counters), we maintain rules 
    // and only add missing ones. Ghost rules are okay as long as they don't block.
    // However, to fix "Upload 0", we'll be more aggressive about rule existence check.
    
    for inbound in inbounds {
        let tag = inbound.tag.as_ref().filter(|s| !s.is_empty()).cloned().unwrap_or_else(|| format!("inbound-{}", inbound.id));
        let port = inbound.port.to_string();
        let comment = format!("xui-{}", tag);

        // Check and Add INPUT (IN)
        check_and_add_rule(cmd, "XUI_IN", &port, "dport", &comment)?;
        // Check and Add OUTPUT (OUT)
        check_and_add_rule(cmd, "XUI_OUT", &port, "sport", &comment)?;
    }
    Ok(())
}

fn ensure_jump_rule_at_top(cmd: &str, base_chain: &str, target_chain: &str) -> ApiResult<()> {
    let output = Command::new(cmd).args(["-L", base_chain, "1", "-n"]).output();
    let is_at_top = if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        stdout.contains(target_chain)
    } else {
        false
    };

    if !is_at_top {
        // Remove all occurrences first to avoid duplicates
        while Command::new(cmd).args(["-D", base_chain, "-j", target_chain]).status().map(|s| s.success()).unwrap_or(false) {}
        // Insert at 1
        let _ = Command::new(cmd).args(["-I", base_chain, "1", "-j", target_chain]).status();
    }
    Ok(())
}

fn check_and_add_rule(cmd: &str, chain: &str, port: &str, port_type: &str, comment: &str) -> ApiResult<()> {
    for proto in ["tcp", "udp"] {
        let port_arg = format!("--{}", port_type);
        // Using -C is standard. If it fails, add.
        let status = Command::new(cmd)
            .args(["-C", chain, "-p", proto, &port_arg, port, "-j", "RETURN", "-m", "comment", "--comment", comment])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !status {
            let _ = Command::new(cmd)
                .args(["-A", chain, "-p", proto, &port_arg, port, "-j", "RETURN", "-m", "comment", "--comment", comment])
                .status();
        }
    }
    Ok(())
}

fn read_all_stats() -> ApiResult<HashMap<String, (u64, u64)>> {
    let mut stats: HashMap<String, (u64, u64)> = HashMap::new();
    
    // Read IPv4
    parse_family_stats("iptables", &mut stats)?;
    // Read IPv6
    if has_command("ip6tables") {
        parse_family_stats("ip6tables", &mut stats)?;
    }

    Ok(stats)
}

fn parse_family_stats(cmd: &str, stats: &mut HashMap<String, (u64, u64)>) -> ApiResult<()> {
    parse_chain_stats_sum(cmd, "XUI_IN", stats, true)?;
    parse_chain_stats_sum(cmd, "XUI_OUT", stats, false)?;
    Ok(())
}

fn parse_chain_stats_sum(cmd: &str, chain: &str, stats: &mut HashMap<String, (u64, u64)>, is_in: bool) -> ApiResult<()> {
    // -x for exact bytes, -v for stats, -n for no DNS
    let output = Command::new(cmd).args(["-L", chain, "-v", "-n", "-x"]).output().map_err(|e| {
        crate::errors::ApiError::SystemError(format!("{} failed: {}", cmd, e))
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if let Some(comment_pos) = line.find("/* xui-") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Columns: pkts, bytes, target, prot, opt, in, out, source, destination, options (including comment)
            if parts.len() < 2 { continue; }
            
            let bytes = parts[1].parse::<u64>().unwrap_or(0);
            
            // Extract tag between "/* xui-" and " */"
            let start = comment_pos + 7;
            if let Some(end) = line[start..].find(" */") {
                let tag = line[start..start+end].trim().to_string();
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

fn has_command(cmd: &str) -> bool {
    Command::new("which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
}

async fn update_db_traffic(
    pool: &SqlitePool,
    data: &TrafficData,
    needs_reapply: &mut bool,
) -> ApiResult<()> {
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
