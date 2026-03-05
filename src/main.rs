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
use ethers::prelude::*;
use dotenvy::dotenv;

// Module declarations
mod handlers;
mod crypto;
mod entropy;

/// Global Application State
/// Combining Blockchain infrastructure with Post-Quantum monitoring
pub struct AppState {
    pub nonce_cache: Cache<String, ()>,
    pub total_requests: AtomicUsize,
    pub blocked_replay: AtomicUsize,
    pub blocked_entropy: AtomicUsize,
    // VeriPhys Ledger Engine (Generated from IntegrityLedger.json)
    pub contract: handlers::VeriPhysContract<SignerMiddleware<Provider<Http>, LocalWallet>>,
    pub registry_path: String,
}

#[tokio::main]
async fn main() {
    // Initialize Environment (Using dotenvy for modern Rust compatibility)
    dotenv().ok(); 

    // 1. Blockchain Engine Configuration (Ethers-rs)
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8545".to_string());
    let contract_addr: Address = std::env::var("CONTRACT_ADDRESS")
        .expect("CONTRACT_ADDRESS missing from .env")
        .parse()
        .expect("Invalid Ethereum Address format");
    let private_key = std::env::var("PRIVATE_KEY").expect("PRIVATE_KEY missing");
    
    let provider = Provider::<Http>::try_from(rpc_url).unwrap();
    
    // ChainID 1337 is standard for local development (Anvil/Hardhat)
    let wallet: LocalWallet = private_key.parse::<LocalWallet>()
        .expect("Invalid Private Key")
        .with_chain_id(1337u64);
        
    let client = Arc::new(SignerMiddleware::new(provider, wallet));

    // 2. Security State Initialization
    let state = Arc::new(AppState {
        nonce_cache: Cache::builder()
            .max_capacity(50_000)
            .time_to_live(Duration::from_secs(300)) // 5-minute replay window
            .build(),
        total_requests: AtomicUsize::new(0),
        blocked_replay: AtomicUsize::new(0),
        blocked_entropy: AtomicUsize::new(0),
        contract: handlers::VeriPhysContract::new(contract_addr, client),
        registry_path: std::env::var("REGISTRY_PATH").unwrap_or_else(|_| "registry.txt".to_string()),
    });

    // 3. Security Sentinel Middleware Stack (Tower-based Protection)
    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|err: BoxError| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("🛡️ Quantum-Fortress Blocked Request: {}", err),
            )
        }))
        .load_shed()             // Prevent system crash during DDoS
        .timeout(Duration::from_secs(30)) 
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024)); // Strict 10MB payload limit

    // 4. Router Construction (Post-Quantum Anchoring & Stats)
    let app = Router::new()
        .route("/v1/anchor", post(handlers::anchor_content))     // Secure File Anchoring
        .route("/v1/quantum-verify", post(handlers::verify))    // PQC Signature Gate
        .route("/api/stats", get(handlers::get_stats))          // Live Security Telemetry
        .layer(middleware_stack)
        .layer(tower_http::cors::CorsLayer::permissive())       // Allow Dashboard access
        .with_state(state);

    // 5. Server Execution (Optimized for Docker/Distroless Environments)
    let port = std::env::var("SERVER_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();

    println!("🚀 Quantum-Fortress Sentinel is ONLINE at http://{}", addr);
    println!("🛡️  Active Defense: SHA3 + Dilithium2 + ReplayGuard + BlockchainRegistry");

    // Standard Axum 0.7 listener (fixes the 'Buf' trait compile error)
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
