use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::SqlitePool;

use crate::{handlers, middleware::auth::auth_middleware, services::system_service::SharedMonitor};

pub fn create_router(pool: SqlitePool, monitor: SharedMonitor) -> Router {
    let auth_routes = Router::new()
        .route("/login", post(handlers::auth::login))
        .route("/update", post(handlers::auth::update_credentials))
        .nest(
            "/",
            Router::new()
                .route("/logout", post(handlers::auth::logout))
                .route("/change-password", post(handlers::auth::change_password))
                .route("/verify", get(handlers::auth::verify))
                .route_layer(middleware::from_fn_with_state(
                    pool.clone(),
                    auth_middleware,
                )),
        )
        .with_state(pool.clone());

    let system_routes = Router::new()
        .route("/sysStats", post(handlers::system::get_sys_stats))
        .route("/restartXray", post(handlers::system::restart_xray))
        .route("/restartPanel", post(handlers::system::restart_panel))
        .route("/startXray", post(handlers::system::start_xray))
        .route("/stopXray", post(handlers::system::stop_xray))
        .route("/applyConfig", post(handlers::system::apply_config))
        .route("/xrayReleases", get(handlers::system::get_xray_releases))
        .route("/updateXray", post(handlers::system::update_xray))
        .route("/getLogs", post(handlers::system::get_logs))
        .route("/export-db", get(handlers::system::export_db))
        .route("/import-db", post(handlers::system::import_db))
        .route("/updateConfig", post(handlers::system::update_config))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .layer(axum::Extension(pool.clone()))
        .with_state(monitor.clone());

    let inbound_routes = Router::new()
        .route("/list", get(handlers::inbound::list_inbounds))
        .route("/add", post(handlers::inbound::add_inbound))
        .route("/update", post(handlers::inbound::update_inbound))
        .route("/del", post(handlers::inbound::del_inbound_post))
        .route("/reset-traffic", post(handlers::inbound::reset_traffic))
        .route("/reset-all", post(handlers::inbound::reset_all_traffic))
        .route("/check-reality", post(handlers::inbound::check_reality))
        .route_layer(middleware::from_fn_with_state(
            pool.clone(),
            auth_middleware,
        ))
        .layer(axum::Extension(pool.clone()))
        .with_state(monitor.clone());

    let xray_routes = Router::new().route(
        "/generate-reality-keys",
        get(crate::handlers::xray::generate_reality_keys),
    );

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/server", system_routes)
        .nest("/inbound", inbound_routes)
        .nest("/xray", xray_routes)
}
