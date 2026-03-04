use axum::{routing::post, Router};
use std::sync::Arc;
use moka::future::Cache;
use dotenvy::dotenv;
use std::env;
use tracing::{info, warn, error};

mod entropy;
mod crypto;
mod handlers;

pub struct AppConfig {
    pub port: u16,
    pub entropy_threshold: f64,
}

pub struct AppState {
    pub nonce_registry: Cache<String, bool>,
    pub config: AppConfig,
}

#[tokio::main]
async fn main() {
    // 1. Initialize Tracing (Logging) System
    // This allows the buyer to see attacks in real-time in the console
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into()))
        .init();

    // 2. Load External Configuration from .env
    dotenv().ok();
    let port: u16 = env::var("SERVER_PORT").unwrap_or("3000".into()).parse().expect("Invalid Port");
    let entropy_threshold: f64 = env::var("ENTROPY_THRESHOLD").unwrap_or("3.2".into()).parse().expect("Invalid Threshold");
    
    let config = AppConfig { port, entropy_threshold };
    
    info!("🚀 QuantumFortress booting up...");
    info!("Settings: [Port: {}] [Entropy Sensitivity: {}]", config.port, config.entropy_threshold);

    let state = Arc::new(AppState {
        nonce_registry: Cache::builder()
            .max_capacity(1_000_000)
            .time_to_live(std::time::Duration::from_secs(120))
            .build(),
        config,
    });

    let app = Router::new()
        .route("/v1/quantum-verify", post(handlers::verify_handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    info!("🛡️ Gateway Protection Active on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
