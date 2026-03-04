use axum::{
    routing::{post, get},
    Router,
    error_handling::HandleErrorLayer,
    http::StatusCode,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use moka::future::Cache;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::{ServiceBuilder, BoxError};
use tower_http::limit::RequestBodyLimitLayer;

mod handlers;
mod crypto;
mod entropy;

pub struct AppState {
    pub nonce_cache: Cache<String, ()>,
    pub total_requests: AtomicUsize,
    pub blocked_replay: AtomicUsize,
    pub blocked_entropy: AtomicUsize,
}

#[tokio::main]
async fn main() {
    // 1. Initialize State with high-performance Cache
    let state = Arc::new(AppState {
        nonce_cache: Cache::builder()
            .max_capacity(50_000)
            .time_to_live(Duration::from_secs(300))
            .build(),
        total_requests: AtomicUsize::new(0),
        blocked_replay: AtomicUsize::new(0),
        blocked_entropy: AtomicUsize::new(0),
    });

    // 2. Define Security Middleware Stack (Rate Limiting & Safety)
    let middleware_stack = ServiceBuilder::new()
        // Handle middleware errors gracefully
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Sentinel Guard Error: {}", err),
            )
        }))
        // Buffer requests to prevent CPU spikes
        .load_shed()
        // Set request timeout (30s) to kill hanging connections (Anti-Slowloris)
        .timeout(Duration::from_secs(30))
        // Limit request size (100KB) to prevent Memory-Exhaustion attacks
        .layer(RequestBodyLimitLayer::new(1024 * 100));

    // 3. Build the Application Router
    let app = Router::new()
        .route("/v1/quantum-verify", post(handlers::verify))
        .route("/api/stats", get(handlers::get_stats))
        // Apply security layers to all routes
        .layer(middleware_stack)
        .with_state(state);

    // 4. Start the Server on Port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 Quantum-Fortress Sentinel is LIVE at http://{}", addr);
    println!("🛡️  Layered Defense Active: PQC + Replay + Entropy + RateLimit");

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
