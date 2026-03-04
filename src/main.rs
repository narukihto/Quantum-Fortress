use axum::{routing::{post, get}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use moka::future::Cache;
use std::time::Duration;
use tokio::net::TcpListener;

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
    let state = Arc::new(AppState {
        nonce_cache: Cache::builder()
            .max_capacity(50_000)
            .time_to_live(Duration::from_secs(300))
            .build(),
        total_requests: AtomicUsize::new(0),
        blocked_replay: AtomicUsize::new(0),
        blocked_entropy: AtomicUsize::new(0),
    });

    let app = Router::new()
        .route("/v1/quantum-verify", post(handlers::verify))
        .route("/api/stats", get(handlers::get_stats))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 QuantumFortress Sentinel active on http://{}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}use axum::{routing::{post, get}, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use moka::future::Cache;
use std::time::Duration;
use tokio::net::TcpListener;

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
    let state = Arc::new(AppState {
        nonce_cache: Cache::builder()
            .max_capacity(50_000)
            .time_to_live(Duration::from_secs(300))
            .build(),
        total_requests: AtomicUsize::new(0),
        blocked_replay: AtomicUsize::new(0),
        blocked_entropy: AtomicUsize::new(0),
    });

    let app = Router::new()
        .route("/v1/quantum-verify", post(handlers::verify))
        .route("/api/stats", get(handlers::get_stats))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🚀 QuantumFortress Sentinel active on http://{}", addr);

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
