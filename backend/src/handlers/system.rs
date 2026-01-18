use crate::middleware::auth::AuthUser;
use axum::extract::{Json, State};

use crate::{
    errors::ApiResult,
    services::system_service::{self, SharedMonitor},
    utils::response::ApiResponse,
};

pub async fn get_sys_stats(
    State(monitor): State<SharedMonitor>,
    _user: AuthUser,
) -> ApiResult<ApiResponse<system_service::SysStats>> {
    let stats = monitor
        .lock()
        .map_err(|e| crate::errors::ApiError::SystemError(format!("Monitor lock poisoned: {}", e)))?
        .get_system_stats()?;
    Ok(ApiResponse::success(stats))
}

pub async fn restart_xray(
    State(monitor): State<SharedMonitor>,
    _user: AuthUser,
) -> ApiResult<ApiResponse<()>> {
    system_service::restart_xray(monitor).await?;
    Ok(ApiResponse::success_no_data("Xray service restarted"))
}

pub async fn restart_panel(_user: AuthUser) -> ApiResult<ApiResponse<()>> {
    system_service::restart_panel().await?;
    Ok(ApiResponse::success_no_data("Panel restart command sent"))
}

pub async fn stop_xray(
    State(monitor): State<SharedMonitor>,
    _user: AuthUser,
) -> ApiResult<ApiResponse<()>> {
    system_service::stop_xray(monitor).await?;
    Ok(ApiResponse::success_no_data("Xray service stopped"))
}

pub async fn start_xray(
    State(monitor): State<SharedMonitor>,
    _user: AuthUser,
) -> ApiResult<ApiResponse<()>> {
    system_service::start_xray(monitor).await?;
    Ok(ApiResponse::success_no_data("Xray service started"))
}

pub async fn update_xray(
    State(monitor): State<SharedMonitor>,
    _user: AuthUser,
    Json(req): Json<system_service::UpdateXrayRequest>,
) -> ApiResult<ApiResponse<()>> {
    system_service::update_xray(monitor, req.version).await?;
    Ok(ApiResponse::success_no_data("Xray update started"))
}

pub async fn apply_config(
    State(monitor): State<SharedMonitor>,
    axum::Extension(pool): axum::Extension<sqlx::SqlitePool>,
    _user: AuthUser,
) -> ApiResult<ApiResponse<()>> {
    crate::services::xray_service::apply_config(&pool, monitor).await?;
    Ok(ApiResponse::success_no_data(
        "Xray config applied and service restarted",
    ))
}

pub async fn get_xray_releases(_user: AuthUser) -> ApiResult<ApiResponse<Vec<String>>> {
    let releases = system_service::get_xray_releases().await?;
    Ok(ApiResponse::success(releases))
}

pub async fn get_logs(_user: AuthUser) -> ApiResult<ApiResponse<Vec<String>>> {
    let logs = system_service::get_logs().await?;
    Ok(ApiResponse::success(logs))
}

pub async fn export_db(
    axum::Extension(pool): axum::Extension<sqlx::SqlitePool>,
    _user: AuthUser,
) -> impl axum::response::IntoResponse {
    use axum::body::Body;
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;
    use serde::Serialize;
    
    // Define temporary structs for export
    #[derive(Serialize, sqlx::FromRow)]
    struct RawUser {
        id: i64,
        username: String,
        password_hash: String,
        password_version: i64,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[derive(Serialize)]
    struct BackupData {
        version: String,
        inbounds: Vec<crate::models::inbound::Inbound>,
        users: Vec<RawUser>,
    }

    // Fetch data
    let inbounds = match sqlx::query_as::<_, crate::models::inbound::Inbound>("SELECT * FROM inbounds")
        .fetch_all(&pool)
        .await {
            Ok(v) => v,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch inbounds: {}", e)).into_response(),
        };

    let users = match sqlx::query_as::<_, RawUser>("SELECT * FROM users")
        .fetch_all(&pool)
        .await {
            Ok(v) => v,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch users: {}", e)).into_response(),
        };

    let backup = BackupData {
        version: "x-ui-lite-json-v1".to_string(),
        inbounds,
        users,
    };

    let filename = format!(
        "x-ui-backup-{}.json",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );

    match serde_json::to_string_pretty(&backup) {
        Ok(json_data) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/json"),
                (
                    header::CONTENT_DISPOSITION,
                    &format!("attachment; filename=\"{}\"", filename),
                ),
            ],
            Body::from(json_data),
        )
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Serialization error: {}", e)).into_response(),
    }
}

pub async fn import_db(
    axum::Extension(pool): axum::Extension<sqlx::SqlitePool>,
    _user: AuthUser,
    mut multipart: axum::extract::Multipart,
) -> ApiResult<ApiResponse<()>> {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct RawUser {
        username: String,
        password_hash: String,
        password_version: i64,
        created_at: chrono::NaiveDateTime,
        updated_at: chrono::NaiveDateTime,
    }

    #[derive(Deserialize)]
    struct BackupData {
        inbounds: Vec<crate::models::inbound::Inbound>,
        users: Vec<RawUser>,
    }

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| crate::errors::ApiError::InternalError(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "db" {
            let data = field.bytes().await.map_err(|e| {
                crate::errors::ApiError::InternalError(format!(
                    "Failed to read multipart data: {}",
                    e
                ))
            })?;

            // 1. Try to parse as JSON (New Backup Format)
            if let Ok(backup) = serde_json::from_slice::<BackupData>(&data) {
                let mut tx = pool.begin().await.map_err(|e| {
                    crate::errors::ApiError::InternalError(format!("Tx begin failed: {}", e))
                })?;

                // Clear existing data
                sqlx::query("DELETE FROM inbounds")
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        crate::errors::ApiError::InternalError(format!(
                            "Clear inbounds failed: {}",
                            e
                        ))
                    })?;
                sqlx::query("DELETE FROM users")
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| {
                        crate::errors::ApiError::InternalError(format!("Clear users failed: {}", e))
                    })?;

                // Restore Users
                for user in backup.users {
                    sqlx::query("INSERT INTO users (username, password_hash, password_version, created_at, updated_at) VALUES (?, ?, ?, ?, ?)")
                        .bind(user.username)
                        .bind(user.password_hash)
                        .bind(user.password_version)
                        .bind(user.created_at)
                        .bind(user.updated_at)
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| crate::errors::ApiError::InternalError(format!("Restore user failed: {}", e)))?;
                }

                // Restore Inbounds
                let now = chrono::Local::now().naive_local();
                for inbound in backup.inbounds {
                     sqlx::query(
                        r#"INSERT INTO inbounds (
                            id, remark, protocol, port, enable, tag, listen, allocate, 
                            settings, stream_settings, sniffing, up, down, total, expiry, created_at, updated_at
                        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
                    )
                    .bind(inbound.id)
                    .bind(inbound.remark)
                    .bind(inbound.protocol)
                    .bind(inbound.port)
                    .bind(inbound.enable)
                    .bind(inbound.tag)
                    .bind(inbound.listen)
                    .bind(inbound.allocate)
                    .bind(inbound.settings)
                    .bind(inbound.stream_settings)
                    .bind(inbound.sniffing)
                    .bind(inbound.up)
                    .bind(inbound.down)
                    .bind(inbound.total)
                    .bind(inbound.expiry)
                    .bind(inbound.created_at.unwrap_or(now)) // Handle Option
                    .bind(inbound.updated_at.unwrap_or(now))
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| crate::errors::ApiError::InternalError(format!("Restore inbound failed: {}", e)))?;
                }

                tx.commit().await.map_err(|e| {
                    crate::errors::ApiError::InternalError(format!("Tx commit failed: {}", e))
                })?;

                return Ok(ApiResponse::success_no_data(
                    "JSON Backup imported successfully.",
                ));
            }

            // 2. Fallback to Legacy DB Replace
            let db_path = "data/x-ui.db";
            let backup_path = "data/x-ui.db.bak";
            
            // Backup current before replace
            if tokio::fs::metadata(db_path).await.is_ok() {
                let _ = tokio::fs::copy(db_path, backup_path).await;
            }

            tokio::fs::write(db_path, data).await.map_err(|e| {
                crate::errors::ApiError::InternalError(format!("Failed to write legacy DB file: {}", e))
            })?;

            return Ok(ApiResponse::success_no_data(
                "Legacy Database file restored. Please restart the panel to apply changes.",
            ));
        }
    }

    Err(crate::errors::ApiError::BadRequest(
        "No 'db' field found in request".to_string(),
    ))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigReq {
    pub web_root: String,
    pub port: u16,
}

pub async fn update_config(
    _user: AuthUser,
    Json(req): Json<UpdateConfigReq>,
) -> ApiResult<ApiResponse<()>> {
    let env_path_str = if std::path::Path::new("/usr/local/x-ui/.env").exists() {
        "/usr/local/x-ui/.env"
    } else {
        ".env"
    };
    let env_path = std::path::Path::new(env_path_str);

    let content = tokio::fs::read_to_string(env_path)
        .await
        .unwrap_or_default();

    let mut clean_root = req.web_root.trim().to_string();
    if !clean_root.starts_with('/') {
        clean_root = format!("/{}", clean_root);
    }
    if !clean_root.ends_with('/') && clean_root != "/" {
        clean_root = format!("{}/", clean_root);
    }

    let mut new_lines = Vec::new();
    let mut has_port = false;
    let mut has_root = false;

    for line in content.lines() {
        if line.starts_with("SERVER_PORT=") {
            new_lines.push(format!("SERVER_PORT={}", req.port));
            has_port = true;
        } else if line.starts_with("WEB_ROOT=") {
            new_lines.push(format!("WEB_ROOT={}", clean_root));
            has_root = true;
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !has_port {
        new_lines.push(format!("SERVER_PORT={}", req.port));
    }
    if !has_root {
        new_lines.push(format!("WEB_ROOT={}", clean_root));
    }

    let new_content = new_lines.join("\n");
    if let Err(e) = tokio::fs::write(env_path, new_content).await {
        return Err(crate::errors::ApiError::InternalError(format!(
            "Failed to write .env: {}",
            e
        )));
    }

    Ok(ApiResponse::success_no_data("Config updated"))
}
