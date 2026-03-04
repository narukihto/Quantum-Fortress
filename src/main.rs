use axum::{
    routing::{get, post},
    Router,
    response::Html,
    extract::State,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering}; // Use Atomics for thread-safe speed
use moka::future::Cache;
use dotenvy::dotenv;
use std::env;
use tracing::{info, Level};
use serde::{Serialize, Deserialize};
use tower_http::limit::RequestBodyLimitLayer; // Prevents DoS by limiting body size

mod entropy;
mod crypto;
mod handlers;

// --- Data Structures ---

/// Metrics using Atomics to avoid Mutex contention during high-traffic attacks
#[derive(Serialize)]
pub struct Stats {
    pub total_requests: AtomicU64,
    pub blocked_replay: AtomicU64,
    pub blocked_entropy: AtomicU64,
}

pub struct AppConfig {
    pub port: u16,
    pub entropy_threshold: f64,
}

pub struct AppState {
    pub nonce_registry: Cache<String, bool>,
    pub config: AppConfig,
    pub metrics: Stats, 
}

#[derive(Debug, Deserialize)]
pub struct SecurePayload {
    pub nonce: String,
    pub data: String,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

// --- Main Logic ---

#[tokio::main]
async fn main() {
    // 1. Initialize Tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();

    // 2. Load Configuration
    dotenv().ok();
    let port: u16 = env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string()).parse().expect("Invalid Port");
    let entropy_threshold: f64 = env::var("ENTROPY_THRESHOLD").unwrap_or_else(|_| "3.2".to_string()).parse().expect("Invalid Threshold");
    
    let config = AppConfig { port, entropy_threshold };
    
    info!("🚀 QuantumFortress booting up in Atomic Mode...");

    // 3. Application State (No Mutex here!)
    let state = Arc::new(AppState {
        nonce_registry: Cache::builder()
            .max_capacity(1_000_000)
            .time_to_live(std::time::Duration::from_secs(120))
            .build(),
        config,
        metrics: Stats {
            total_requests: AtomicU64::new(0),
            blocked_replay: AtomicU64::new(0),
            blocked_entropy: AtomicU64::new(0),
        },
    });

    // 4. Routes with Security Layers
    let app = Router::new()
        .route("/", get(dashboard_ui_handler))
        .route("/api/stats", get(get_stats_handler))
        .route("/v1/quantum-verify", post(handlers::verify_handler))
        // CRITICAL: Limit request size to 64KB to prevent Memory-based DoS attacks
        .layer(RequestBodyLimitLayer::new(64 * 1024)) 
        .with_state(state);

    // 5. Start Server
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    info!("🛡️ Gateway Protection Active on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

// --- Dashboard Handlers ---

async fn dashboard_ui_handler() -> Html<String> {
    Html(include_str!("dashboard.html").to_string())
}

/// Fetches stats using Atomic Loads (Non-blocking)
async fn get_stats_handler(State(state): State<Arc<AppState>>) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "total_requests": state.metrics.total_requests.load(Ordering::Relaxed),
        "blocked_replay": state.metrics.blocked_replay.load(Ordering::Relaxed),
        "blocked_entropy": state.metrics.blocked_entropy.load(Ordering::Relaxed),
    }))
}
