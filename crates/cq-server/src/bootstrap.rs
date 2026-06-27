use axum::{extract::DefaultBodyLimit, Router};
use cq_db::{Db, DbConfig};
use cq_db::repositories::bot::BotRepository;
use tokio::net::TcpListener;
use tokio::time::{interval, Duration};
use tower_http::{compression::CompressionLayer, cors::{Any, CorsLayer}, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::{config::Config, http, state::AppState, static_files};

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Config::from_env();
    let db = Db::connect_lazy(&DbConfig {
        database_url: config.database_url.clone(),
        max_connections: 16,
    })?;
    let state = AppState::new(config.clone(), db).await;
    spawn_bot_auto_tick(state.clone());
    let app = build_router(state);
    let listener = TcpListener::bind(config.bind_addr()).await?;
    tracing::info!(
        addr = %config.bind_addr(),
        env = %config.app_env,
        public_base_url = %config.public_base_url,
        "cq-server started"
    );
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn spawn_bot_auto_tick(state: AppState) {
    if !state.config.bot_auto_tick_enabled {
        tracing::info!("bot auto tick disabled");
        return;
    }
    let interval_seconds = state.config.bot_auto_tick_interval_seconds.max(1);
    let limit = state.config.bot_auto_tick_limit.clamp(1, 200);
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(interval_seconds));
        loop {
            ticker.tick().await;
            match BotRepository::new(state.db.pool()).tick(limit).await {
                Ok((_, changed)) if changed > 0 => {
                    tracing::debug!(changed, limit, "bot auto tick completed");
                }
                Ok(_) => {}
                Err(err) => {
                    tracing::warn!(error = ?err, "bot auto tick failed");
                }
            }
        }
    });
}

pub fn build_router(state: AppState) -> Router {
    let body_limit = state.config.json_body_limit_bytes;
    let public_dir = state.config.public_dir.clone();
    let admin_path = state.config.admin_path.trim_matches('/').to_string();
    let admin_mount = format!("/{admin_path}");

    Router::new()
        .nest_service(
            &admin_mount,
            static_files::subdir_service(&public_dir, &admin_path),
        )
        .nest("/api", http::router(&state.config))
        .route("/ws", axum::routing::get(crate::ws::handler))
        .fallback_service(static_files::service(&public_dir))
        .layer(DefaultBodyLimit::max(body_limit))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any))
        .with_state(state)
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer()).init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
