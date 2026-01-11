use crate::errors::ApiResult;
use crate::models::inbound::Inbound;
use crate::services::system_service::SharedMonitor;
use crate::services::xray_service;
use sqlx::SqlitePool;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;
use tokio::time::{interval, Duration};

pub fn start_traffic_stats_task(pool: SqlitePool, monitor: SharedMonitor) {
    tracing::info!("Starting traffic stats collector for xray-lite (Log Parsing Mode)");
    
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(5));
        let mut last_pos: u64 = 0;
        let log_path = "/usr/local/x-ui/logs/access.log";

        loop {
            interval.tick().await;
            
            if let Err(e) = process_logs(&pool, monitor.clone(), &log_path, &mut last_pos).await {
                tracing::error!("Error processing traffic logs: {}", e);
            }
        }
    });
}

async fn process_logs(
    pool: &SqlitePool,
    monitor: SharedMonitor,
    log_path: &str,
    last_pos: &mut u64,
) -> ApiResult<()> {
    let path = Path::new(log_path);
    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path).map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Failed to open log file: {}", e))
    })?;
    
    let metadata = file.metadata().map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Failed to get log metadata: {}", e))
    })?;
    
    let current_size = metadata.len();

    // Check if file was rotated or cleared
    if current_size < *last_pos {
        *last_pos = 0;
    }

    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(*last_pos)).map_err(|e| {
        crate::errors::ApiError::SystemError(format!("Failed to seek log file: {}", e))
    })?;

    let mut lines_processed = 0;
    let mut needs_reapply = false;

    for line_result in reader.lines() {
        let line = match line_result {
            Ok(l) => l,
            Err(_) => break,
        };
        
        lines_processed += 1;

        // Pattern: [TRAFFIC] tag=inbound-45d1a3df up=123 down=456
        if let Some(traffic_data) = parse_traffic_line(&line) {
            if let Err(e) = update_db_traffic(pool, &traffic_data, &mut needs_reapply).await {
                tracing::error!("Failed to update traffic for tag {}: {}", traffic_data.tag, e);
            }
        }
    }

    *last_pos = current_size;
    
    if lines_processed > 0 {
        tracing::debug!("Processed {} lines from traffic log", lines_processed);
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

fn parse_traffic_line(line: &str) -> Option<TrafficData> {
    if !line.contains("[TRAFFIC]") {
        return None;
    }

    // Example: 2026-01-11T04:25:31.123Z INFO [TRAFFIC] tag=inbound-864ee8e5 up=1024 down=2048
    let tag = line.split("tag=").nth(1)?.split_whitespace().next()?;
    let up_str = line.split("up=").nth(1)?.split_whitespace().next()?;
    let down_str = line.split("down=").nth(1)?.split_whitespace().next()?;

    let up = up_str.parse::<i64>().ok()?;
    let down = down_str.parse::<i64>().ok()?;

    Some(TrafficData {
        tag: tag.to_string(),
        up,
        down,
    })
}

async fn update_db_traffic(
    pool: &SqlitePool,
    data: &TrafficData,
    needs_reapply: &mut bool,
) -> ApiResult<()> {
    // 1. Get current stats and limit
    let inbound = sqlx::query_as::<_, Inbound>("SELECT * FROM inbounds WHERE tag = ?")
        .bind(&data.tag)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| crate::errors::ApiError::InternalError(format!("Inbound tag {} not found", data.tag)))?;

    let new_up = inbound.up + data.up;
    let new_down = inbound.down + data.down;
    let mut enable = 1;

    // 2. Check quota
    if inbound.total > 0 && (new_up + new_down) >= inbound.total {
        enable = 0;
        *needs_reapply = true;
        tracing::info!("Node {} reached traffic quota ({} >= {}), disabling.", inbound.remark, new_up + new_down, inbound.total);
    }

    // 3. Update DB
    sqlx::query("UPDATE inbounds SET up = ?, down = ?, enable = ? WHERE tag = ?")
        .bind(new_up)
        .bind(new_down)
        .bind(enable)
        .bind(&data.tag)
        .execute(pool)
        .await?;

    Ok(())
}
