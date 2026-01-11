use crate::errors::ApiResult;
use crate::models::inbound::{CreateInboundRequest, Inbound, UpdateInboundRequest};
use sqlx::SqlitePool;

pub async fn get_all_inbounds(pool: &SqlitePool) -> ApiResult<Vec<Inbound>> {
    let inbounds = sqlx::query_as::<_, Inbound>("SELECT * FROM inbounds ORDER BY id DESC")
        .fetch_all(pool)
        .await?;
    Ok(inbounds)
}

pub async fn add_inbound(pool: &SqlitePool, req: CreateInboundRequest) -> ApiResult<Inbound> {
    let now = chrono::Local::now().naive_local();

    let settings_json = req
        .settings
        .map(|v| v.to_string())
        .unwrap_or("{}".to_string());
    let stream_settings_json = req
        .stream_settings
        .map(|v| v.to_string())
        .unwrap_or("{}".to_string());
    let sniffing_json = req
        .sniffing
        .map(|v| v.to_string())
        .unwrap_or("{}".to_string());
    let allocate_json = req.allocate.map(|v| v.to_string());

    let enable = req.enable.unwrap_or(true);

    let tag = req.tag.or_else(|| {
        Some(format!(
            "inbound-{}",
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        ))
    });

    let id = req.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let inbound = sqlx::query_as::<_, Inbound>(
        r#"
        INSERT INTO inbounds (id, remark, protocol, port, enable, tag, listen, allocate, settings, stream_settings, sniffing, total, expiry, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#
    )
    .bind(id)
    .bind(req.remark)
    .bind(req.protocol)
    .bind(req.port)
    .bind(enable)
    .bind(tag)
    .bind(req.listen)
    .bind(allocate_json)
    .bind(settings_json)
    .bind(stream_settings_json)
    .bind(sniffing_json)
    .bind(req.total.unwrap_or(0))
    .bind(req.expiry.unwrap_or(0))
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(inbound)
}

pub async fn update_inbound(pool: &SqlitePool, req: UpdateInboundRequest) -> ApiResult<Inbound> {
    let now = chrono::Local::now().naive_local();

    let settings_str = req.settings.map(|v| v.to_string());
    let stream_settings_str = req.stream_settings.map(|v| v.to_string());
    let sniffing_str = req.sniffing.map(|v| v.to_string());
    let allocate_str = req.allocate.map(|v| v.to_string());

    let inbound = sqlx::query_as::<_, Inbound>(
        r#"
        UPDATE inbounds 
        SET 
            remark = COALESCE(?, remark),
            protocol = COALESCE(?, protocol),
            port = COALESCE(?, port),
            enable = COALESCE(?, enable),
            tag = COALESCE(?, tag),
            listen = COALESCE(?, listen),
            allocate = COALESCE(?, allocate),
            settings = COALESCE(?, settings),
            stream_settings = COALESCE(?, stream_settings),
            sniffing = COALESCE(?, sniffing),
            total = COALESCE(?, total),
            expiry = COALESCE(?, expiry),
            updated_at = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(req.remark)
    .bind(req.protocol)
    .bind(req.port)
    .bind(req.enable)
    .bind(req.tag)
    .bind(req.listen)
    .bind(allocate_str)
    .bind(settings_str)
    .bind(stream_settings_str)
    .bind(sniffing_str)
    .bind(req.total)
    .bind(req.expiry)
    .bind(now)
    .bind(req.id)
    .fetch_one(pool)
    .await?;

    Ok(inbound)
}

pub async fn delete_inbound(pool: &SqlitePool, id: &str) -> ApiResult<()> {
    sqlx::query("DELETE FROM inbounds WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn reset_inbound_traffic(pool: &SqlitePool, id: &str) -> ApiResult<()> {
    sqlx::query("UPDATE inbounds SET up = 0, down = 0 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn reset_all_inbound_traffic(pool: &SqlitePool) -> ApiResult<()> {
    sqlx::query("UPDATE inbounds SET up = 0, down = 0")
        .execute(pool)
        .await?;
    Ok(())
}
