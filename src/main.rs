use axum::{
    routing::{get, post},
    Router,
    response::Html,
    extract::State,
};
use std::sync::{Arc, Mutex};
use moka::future::Cache;
use dotenvy::dotenv;
use std::env;
use tracing::{info, Level};
use serde::{Serialize, Deserialize};

mod entropy;
mod crypto;
mod handlers;

// --- Data Structures ---

#[derive(Serialize, Clone)]
pub struct Stats {
    pub total_requests: u64,
    pub blocked_replay: u64,
    pub blocked_entropy: u64,
}

pub struct AppConfig {
    pub port: u16,
    pub entropy_threshold: f64,
}

pub struct AppState {
    pub nonce_registry: Cache<String, bool>,
    pub config: AppConfig,
    pub metrics: Mutex<Stats>, // Added for Dashboard metrics
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
    // 1. Initialize Tracing (Logging) System
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();

    // 2. Load Configuration
    dotenv().ok();
    let port: u16 = env::var("SERVER_PORT").unwrap_or("3000".into()).parse().expect("Invalid Port");
    let entropy_threshold: f64 = env::var("ENTROPY_THRESHOLD").unwrap_or("3.2".into()).parse().expect("Invalid Threshold");
    
    let config = AppConfig { port, entropy_threshold };
    
    info!("🚀 QuantumFortress booting up...");
    info!("Settings: [Port: {}] [Entropy Sensitivity: {}]", config.port, config.entropy_threshold);

    // 3. Application State
    let state = Arc::new(AppState {
        nonce_registry: Cache::builder()
            .max_capacity(1_000_000)
            .time_to_live(std::time::Duration::from_secs(120))
            .build(),
        config,
        metrics: Mutex::new(Stats {
            total_requests: 0,
            blocked_replay: 0,
            blocked_entropy: 0,
        }),
    });

    // 4. Routes
    let app = Router::new()
        .route("/", get(dashboard_ui_handler))      // Modern UI
        .route("/api/stats", get(get_stats_handler)) // JSON Data for UI
        .route("/v1/quantum-verify", post(handlers::verify_handler))
        .with_state(state);

    // 5. Start Server
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    
    info!("🛡️ Gateway Protection Active on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

// --- Dashboard Handlers ---

async fn dashboard_ui_handler() -> Html<String> {
    // We embed the HTML directly for the buyer's ease of use
    Html(include_str!("dashboard.html").to_string())
}

async fn get_stats_handler(State(state): State<Arc<AppState>>) -> axum::Json<Stats> {
    let stats = state.metrics.lock().unwrap();
    axum::Json(stats.clone())
}
