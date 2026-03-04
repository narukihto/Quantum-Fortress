use axum::{
    routing::{get, post},
    extract::State,
    Router,
    Json,
};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use moka::future::Cache;
use std::net::SocketAddr;

// Module declarations (Links to your files)
mod crypto;
mod entropy;
mod handlers;

// Shared State Structure
pub struct Metrics {
    pub total_requests: AtomicU64,
    pub blocked_replay: AtomicU64,
    pub blocked_entropy: AtomicU64,
}

pub struct AppState {
    pub nonce_registry: Cache<String, bool>,
    pub metrics: Metrics,
}

// Payload structure used by handlers.rs
#[derive(serde::Deserialize)]
pub struct SecurePayload {
    pub nonce: String,
    pub data: String,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[tokio::main]
async fn main() {
    // 1. Initialize Logging/Tracing
    tracing_subscriber::fmt::init();

    // 2. Initialize Shared State (Nonce Registry and Metrics)
    let state = Arc::new(AppState {
        nonce_registry: Cache::new(10_000), // Protects against Replay Attacks
        metrics: Metrics {
            total_requests: AtomicU64::new(0),
            blocked_replay: AtomicU64::new(0),
            blocked_entropy: AtomicU64::new(0),
        },
    });

    // 3. Define API Routes
    let app = Router::new()
        // Serve the SOC Dashboard
        .route("/", get(|| async { ax_html(include_str!("../dashboard.html")) }))
        
        // Post-Quantum Verification Endpoint
        .route("/api/verify", post(handlers::verify_handler))
        
        // Stats Endpoint (Fixes Docker Healthcheck mismatch)
        .route("/api/stats", get(move |State(st): State<Arc<AppState>>| async move {
            Json(serde_json::json!({
                "total_requests": st.metrics.total_requests.load(std::sync::atomic::Ordering::Relaxed),
                "blocked_replay": st.metrics.blocked_replay.load(std::sync::atomic::Ordering::Relaxed),
                "blocked_entropy": st.metrics.blocked_entropy.load(std::sync::atomic::Ordering::Relaxed),
            }))
        }))
        .with_state(state);

    // 4. Start the Server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🛡️ QuantumFortress active on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// Helper function for HTML response
fn ax_html(content: &'static str) -> axum::response::Html<&'static str> {
    axum::response::Html(content)
}
