use crate::errors::ApiResult;
use crate::middleware::auth::AuthUser;
use crate::models::inbound::{
    CreateInboundRequest, DeleteInboundRequest, ResetTrafficRequest, UpdateInboundRequest,
};
use crate::services::{inbound_service, system_service::SharedMonitor, xray_service};
use crate::utils::{reality, response::ApiResponse};
use axum::extract::{Extension, Json, State};

use sqlx::SqlitePool;

pub async fn list_inbounds(
    _user: AuthUser,
    Extension(pool): Extension<SqlitePool>,
) -> ApiResult<ApiResponse<Vec<crate::models::inbound::Inbound>>> {
    let list = inbound_service::get_all_inbounds(&pool).await?;
    Ok(ApiResponse::success(list))
}

pub async fn add_inbound(
    _user: AuthUser,
    State(monitor): State<SharedMonitor>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<CreateInboundRequest>,
) -> ApiResult<ApiResponse<crate::models::inbound::Inbound>> {
    let port = payload.port;
    let inbound = inbound_service::add_inbound(&pool, payload).await?;

    tokio::task::spawn_blocking(move || {
        tracing::info!("Starting background firewall task for port {}", port);
        crate::utils::firewall::open_port(port as u16);
        tracing::info!("Finished background firewall task for port {}", port);
    });

    xray_service::apply_config(&pool, monitor).await?;

    Ok(ApiResponse::success_with_msg(inbound, "Added successfully"))
}

pub async fn update_inbound(
    _user: AuthUser,
    State(monitor): State<SharedMonitor>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<UpdateInboundRequest>,
) -> ApiResult<ApiResponse<crate::models::inbound::Inbound>> {
    let port = payload.port;
    let inbound = inbound_service::update_inbound(&pool, payload).await?;

    if let Some(p) = port {
        tokio::task::spawn_blocking(move || {
            tracing::info!("Starting background firewall task for port {}", p);
            crate::utils::firewall::open_port(p as u16);
            tracing::info!("Finished background firewall task for port {}", p);
        });
    }

    xray_service::apply_config(&pool, monitor).await?;

    Ok(ApiResponse::success_with_msg(
        inbound,
        "Updated successfully",
    ))
}

pub async fn del_inbound_post(
    _user: AuthUser,
    State(monitor): State<SharedMonitor>,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<DeleteInboundRequest>,
) -> ApiResult<ApiResponse<()>> {
    inbound_service::delete_inbound(&pool, &payload.id).await?;
    xray_service::apply_config(&pool, monitor).await?;
    Ok(ApiResponse::success_no_data("Deleted successfully"))
}
pub async fn check_reality(
    _user: AuthUser,
    Json(payload): Json<reality::RealityCheckRequest>,
) -> ApiResult<ApiResponse<reality::RealityCheckResponse>> {
    let result = reality::check_domain(&payload.domain)
        .await
        .map_err(|e| crate::errors::ApiError::InternalError(e.to_string()))?;
    Ok(ApiResponse::success(result))
}

pub async fn reset_traffic(
    _user: AuthUser,
    Extension(pool): Extension<SqlitePool>,
    Json(payload): Json<ResetTrafficRequest>,
) -> ApiResult<ApiResponse<()>> {
    inbound_service::reset_inbound_traffic(&pool, &payload.id).await?;
    Ok(ApiResponse::success_no_data("Traffic reset successfully"))
}

pub async fn reset_all_traffic(
    _user: AuthUser,
    Extension(pool): Extension<SqlitePool>,
) -> ApiResult<ApiResponse<()>> {
    inbound_service::reset_all_inbound_traffic(&pool).await?;
    Ok(ApiResponse::success_no_data("All traffic reset successfully"))
}
