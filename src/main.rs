use axum::{
    routing::post,
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use moka::future::Cache;
use std::time::Duration;

mod handlers;
mod crypto;
mod entropy;

#[derive(Clone)]
pub struct AppState {
    pub nonce_cache: Cache<String, ()>,
}

#[tokio::main]
async fn main() {
    let nonce_cache = Cache::builder()
        .max_capacity(50_000)
        .time_to_live(Duration::from_secs(300))
        .build();

    let state = AppState {
        nonce_cache,
    };

    let app = Router::new()
        .route("/api/verify", post(handlers::verify))
        .with_state(Arc::new(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Secure PQ Gateway running on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
