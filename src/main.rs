use axum::{
    routing::{post, get},
    Router,
    error_handling::HandleErrorLayer,
    http::StatusCode,
    extract::DefaultBodyLimit,
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use moka::future::Cache;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::{ServiceBuilder, BoxError};
use tower_http::limit::RequestBodyLimitLayer;
use ethers::prelude::*;
use dotenvy::dotenv;

// Module declarations
mod handlers;
mod crypto;
mod entropy;

/// Global Application State
/// Combining Blockchain infrastructure with Security monitoring
pub struct AppState {
    pub nonce_cache: Cache<String, ()>,
    pub total_requests: AtomicUsize,
    pub blocked_replay: AtomicUsize,
    pub blocked_entropy: AtomicUsize,
    // VeriPhys Ledger Engine
    pub contract: handlers::VeriPhysContract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    pub registry_path: String,
}

#[tokio::main]
async fn main() {
    // Initialize Environment
    dotenv().ok(); 

    // 1. Blockchain Engine Configuration
    let rpc_url = std::env::var("RPC_URL").expect("RPC_URL missing");
    let contract_addr: Address = std::env::var("CONTRACT_ADDRESS")
        .expect("ADDR missing")
        .parse()
        .expect("Invalid Contract Address");
    let private_key = std::env::var("PRIVATE_KEY").expect("KEY missing");
    
    let provider = Provider::<Http>::try_from(rpc_url).unwrap();
    // Default Chain ID for Local Testing (Anvil/Hardhat)
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()
        .unwrap()
        .with_chain_id(1337u64);
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    // 2. State Initialization (Cache + Metrics + Ledger)
    let state = Arc::new(AppState {
        nonce_cache: Cache::builder()
            .max_capacity(50_000)
            .time_to_live(Duration::from_secs(300))
            .build(),
        total_requests: AtomicUsize::new(0),
        blocked_replay: AtomicUsize::new(0),
        blocked_entropy: AtomicUsize::new(0),
        contract: handlers::VeriPhysContract::new(contract_addr, client),
        registry_path: std::env::var("REGISTRY_PATH").unwrap_or_else(|_| "registry.txt".to_string()),
    });

    // 3. Security Sentinel Middleware Stack
    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("🛡️ VeriPhys Guard Blocked: {}", err),
            )
        }))
        .load_shed() // Dropping excess traffic during bursts
        .timeout(Duration::from_secs(30)) // Anti-Slowloris Protection
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // 10MB Secure Limit

    // 4. Router Construction (Anchoring + Verification + Monitoring)
    let app = Router::new()
        .route("/v1/anchor", post(handlers::anchor_content))     // Register Content
        .route("/v1/quantum-verify", post(handlers::verify))    // Verify PQC Content
        .route("/api/stats", get(handlers::get_stats))          // Live Security Stats
        .layer(middleware_stack)
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);

    // 5. Server Execution (Docker-Compatible Binding)
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    println!("🚀 VeriPhys Sentinel Core is ONLINE at http://{}", addr);
    println!("🛡️  Active Protections: SHA3-256 + PQC + ReplayGuard + RateLimit");

    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
