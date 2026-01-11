mod config;
mod db;
mod errors;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use axum::http::Method;
use axum::response::IntoResponse;
use axum::Router;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn auto_init_env() {
    let env_path = std::path::Path::new(".env");
    if !env_path.exists() {
        let secret = uuid::Uuid::new_v4().to_string();
        let content = format!(
            r#"# Auto-generated configuration - created on first start
DATABASE_URL=sqlite:/usr/local/x-ui/data/x-ui.db

# JWT secret - auto-generated strong random string
JWT_SECRET={}
JWT_EXPIRATION_HOURS=24

# Panel listening configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Xray binary path
XRAY_BIN_PATH=./bin/xray
XRAY_CONFIG_PATH=./data/xray.json

# Log level
RUST_LOG=debug,sqlx=warn
"#,
            secret
        );
        if let Err(e) = std::fs::write(env_path, content) {
            eprintln!("Failed to auto-create .env file: {}", e);
        } else {
            println!("First run: Auto-generated .env configuration and random encryption key");
        }
    }

    let _ = std::fs::create_dir_all("data");
    let _ = std::fs::create_dir_all("logs");
    let _ = std::fs::create_dir_all("bin");
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> anyhow::Result<()> {
    auto_init_env();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
            println!("X-UI Backend CLI");
            println!("Usage:");
            println!("  --version, -v                      Show version");
            println!("  --reset, -r                        Reset admin to defaults (admin/admin)");
            println!("  --user, -u <username>              Set admin username");
            println!("  --password, -p <password>          Set admin password");
            println!("  --port <port>                      Update port in .env");
            println!("  --web-root <path>                  Update web root in .env");
            return Ok(());
        }

        if args.contains(&"--version".to_string()) || args.contains(&"-v".to_string()) {
            println!("x-ui-backend v{}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }

        if args.contains(&"--reset".to_string()) || args.contains(&"-r".to_string()) {
            dotenvy::dotenv().ok();
            let pool = db::init_pool().await?;
            services::auth_service::reset_admin(&pool).await?;
            println!("Admin credentials reset to: admin / admin");
            return Ok(());
        }

        if let Some(u_idx) = args.iter().position(|r| r == "--user" || r == "-u") {
            if let Some(username) = args.get(u_idx + 1) {
                if let Some(p_idx) = args.iter().position(|r| r == "--password" || r == "-p") {
                    if let Some(password) = args.get(p_idx + 1) {
                        dotenvy::dotenv().ok();
                        let pool = db::init_pool().await?;
                        let hashed = utils::password::hash_password(password)?;
                        sqlx::query("UPDATE users SET username = ?, password_hash = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1")
                            .bind(username)
                            .bind(&hashed)
                            .execute(&pool)
                            .await?;
                        println!("Admin username updated to: {} / ***", username);
                        return Ok(());
                    }
                }
            }
        }
    }

    dotenvy::dotenv().ok();

    // Setup log rotation: daily rotation, keep 7 days
    let log_dir = std::path::Path::new("logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }

    let file_appender = tracing_appender::rolling::daily("logs", "x-ui.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false),
        )
        .init();

    let pool = db::init_pool().await?;
    db::run_migrations(&pool).await?;

    services::auth_service::init_default_admin(&pool).await?;

    let monitor = std::sync::Arc::new(std::sync::Mutex::new(
        services::system_service::SystemMonitor::new(),
    ));

    if std::env::var("XRAY_BIN_PATH").is_err() {
        std::env::set_var("XRAY_BIN_PATH", "./bin/xray");
    }
    if std::env::var("XRAY_CONFIG_PATH").is_err() {
        std::env::set_var("XRAY_CONFIG_PATH", "./data/xray.json");
    }

    let mut web_root = std::env::var("WEB_ROOT").unwrap_or_else(|_| "/".to_string());
    if !web_root.starts_with('/') {
        web_root = format!("/{}", web_root);
    }
    if !web_root.ends_with('/') {
        web_root = format!("{}/", web_root);
    }
    std::env::set_var("WEB_ROOT", web_root);

    if let Err(e) = services::xray_service::apply_config(&pool, monitor.clone()).await {
        tracing::error!("Failed to apply config on startup: {}", e);
    } else {
        tracing::info!("Initial Xray core successfully started or updated");
    }

    services::traffic_service::start_traffic_stats_task(pool.clone(), monitor.clone());

    #[cfg(debug_assertions)]
    let cors_layer = match std::env::var("SERVER_HOST") {
        Ok(_) => CorsLayer::new().allow_origin(tower_http::cors::Any),
        Err(_) => CorsLayer::new()
            .allow_origin(
                "http://localhost:5173"
                    .parse::<axum::http::HeaderValue>()
                    .unwrap(),
            )
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ])
            .allow_headers([
                axum::http::header::CONTENT_TYPE,
                axum::http::header::AUTHORIZATION,
            ])
            .allow_credentials(true),
    };

    #[cfg(not(debug_assertions))]
    let cors_layer = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ])
        .allow_credentials(false);

    let api_router = routes::create_router(pool, monitor)
        .layer(axum::middleware::from_fn(
            middleware::security::security_headers_middleware,
        ))
        .layer(cors_layer);

    let env_dist_path = std::env::var("WEB_DIST_PATH").unwrap_or_default();

    let mut candidates = vec![
        "./bin/dist",
        "./dist",
        "../web/dist",
        "dist",
        "/usr/local/x-ui/bin/dist",
        "/usr/local/x-ui/dist",
    ];

    if !env_dist_path.is_empty() {
        candidates.insert(0, env_dist_path.as_str());
    }

    let mut dist_path = "./bin/dist".to_string();
    for path in candidates {
        if !path.is_empty() && std::path::Path::new(path).exists() {
            dist_path = path.to_string();
            break;
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        tracing::info!("Current Working Directory: {:?}", cwd);
    }

    let index_dist_path = dist_path.clone();
    let index_handler = move |req: axum::http::Request<axum::body::Body>| async move {
        let web_root = std::env::var("WEB_ROOT").unwrap_or_else(|_| "/".to_string());

        let path_str = req.uri().path();
        if path_str.ends_with(".js")
            || path_str.ends_with(".css")
            || path_str.ends_with(".png")
            || path_str.ends_with(".ico")
            || path_str.ends_with(".svg")
            || path_str.ends_with(".woff2")
        {
            return (axum::http::StatusCode::NOT_FOUND, "Asset not found").into_response();
        }

        let path = std::path::Path::new(&index_dist_path).join("index.html");
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => {
                let mut replaced = content.replace("{{WEB_ROOT}}", &web_root);

                if !replaced.contains("<base") {
                    let base_tag = format!("<base href=\"{}\">", web_root);
                    replaced = replaced.replace("<head>", &format!("<head>{}", base_tag));
                }

                (
                    axum::http::StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, "text/html")],
                    replaced,
                )
                    .into_response()
            }
            Err(e) => {
                tracing::error!("Failed to read index.html at {:?}: {}", path, e);
                (axum::http::StatusCode::NOT_FOUND, "index.html not found").into_response()
            }
        }
    };

    let file_service = tower_http::services::ServeDir::new(&dist_path)
        .fallback(axum::routing::get(index_handler.clone()));

    let web_root = std::env::var("WEB_ROOT").unwrap_or_else(|_| "/".to_string());
    let mut normalized_web_root = if !web_root.starts_with('/') {
        format!("/{}", web_root)
    } else {
        web_root.clone()
    };
    if !normalized_web_root.ends_with('/') {
        normalized_web_root = format!("{}/", normalized_web_root);
    }

    let base_path = normalized_web_root.trim_end_matches('/').to_string();

    let router = Router::new()
        .nest("/api", api_router)
        .route("/", axum::routing::get(index_handler.clone()))
        .route("/index.html", axum::routing::get(index_handler.clone()))
        .fallback_service(file_service);

    let app = if base_path.is_empty() || base_path == "/" {
        router.with_state(())
    } else {
        let subpath = base_path.clone();
        let nest_path = subpath.trim_end_matches('/').to_string();

        let subpath_slash = format!("{}/", nest_path);

        Router::new()
            .route(&subpath_slash, axum::routing::get(index_handler.clone()))
            .nest(&nest_path, router)
            .fallback(
                move |req: axum::http::Request<axum::body::Body>| async move {
                    tracing::debug!("Global fallback 404: {}", req.uri().path());
                    axum::http::StatusCode::NOT_FOUND.into_response()
                },
            )
            .with_state(())
    };

    tracing::info!(
        "Web Root: {}",
        if web_root.is_empty() { "/" } else { &web_root }
    );
    tracing::info!("Using web dist path: {}", dist_path);
    tracing::info!("Security middleware enabled: CSP, X-Frame-Options, X-XSS-Protection");

    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(
        "🚀 X-UI-Lite Backend v2.4.0 [Powered by xray-lite v0.2.74]"
    );
    tracing::info!(
        "X-UI Backend listening on http://{}",
        listener.local_addr()?
    );

    axum::serve(listener, app).await?;

    Ok(())
}
