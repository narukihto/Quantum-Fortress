use axum::{routing::post, Json, Router, extract::State, http::{StatusCode, Request}, body::Body};
use std::sync::Arc;
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use tower_http::limit::RequestBodyLimitLayer;
use tower::ServiceExt;

mod entropy;
mod crypto;

#[derive(Serialize, Deserialize, Clone)]
pub struct SecurePayload {
    pub nonce: String,
    pub data: String,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

pub struct AppState {
    pub nonce_registry: Cache<String, bool>,
}

pub fn create_app() -> Router {
    let state = Arc::new(AppState {
        nonce_registry: Cache::builder()
            .max_capacity(1_000_000)
            .time_to_live(std::time::Duration::from_secs(120))
            .build(),
    });

    Router::new()
        .route("/v1/quantum-verify", post(verify_handler))
        .layer(RequestBodyLimitLayer::new(10 * 1024))
        .with_state(state)
}

#[tokio::main]
async fn main() {
    let app = create_app();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("🛡️ QuantumFortress active on port 3000");
    axum::serve(listener, app).await.unwrap();
}

async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SecurePayload>,
) -> Result<Json<String>, StatusCode> {
    if state.nonce_registry.contains_key(&payload.nonce) {
        return Err(StatusCode::CONFLICT);
    }
    state.nonce_registry.insert(payload.nonce.clone(), true).await;

    if !entropy::EntropyScanner::is_secure(payload.data.as_bytes()) {
        return Err(StatusCode::BAD_REQUEST);
    }

    if !crypto::QuantumCrypto::verify_signature(
        payload.data.as_bytes(),
        &payload.signature,
        &payload.public_key
    ) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(Json("Verified_Secure_PQC".to_string()))
}

// --- INTEGRATED TESTS ---
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_security_pipeline() {
        let app = create_app();
        let payload = json!({
            "nonce": "test_1",
            "data": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", // Low entropy
            "signature": vec![0u8; 10],
            "public_key": vec![0u8; 10]
        });

        let res = app.oneshot(Request::builder()
            .method("POST")
            .uri("/v1/quantum-verify")
            .header("Content-Type", "application/json")
            .body(Body::from(payload.to_string())).unwrap()).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }
}
