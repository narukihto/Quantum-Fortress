use axum::{
    extract::State,
    Json,
    response::IntoResponse,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use crate::AppState;
use crate::crypto::QuantumCrypto;
use crate::entropy::EntropyScanner;

#[derive(Deserialize)]
pub struct QuantumRequest {
    pub nonce: String,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[derive(Serialize)]
pub struct QuantumResponse {
    pub status: String,
    pub message: String,
}

/// Core Verification Handler
pub async fn verify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QuantumRequest>,
) -> impl IntoResponse {
    // 1. Increment Total Request Counter immediately upon arrival
    state.total_requests.fetch_add(1, Ordering::SeqCst);

    // 2. REPLAY PROTECTION LAYER
    // Check if nonce exists in the high-speed Moka cache
    if state.nonce_cache.get(&payload.nonce).await.is_some() {
        state.blocked_replay.fetch_add(1, Ordering::SeqCst);
        return (StatusCode::FORBIDDEN, Json(QuantumResponse {
            status: "error".into(),
            message: "Replay attack detected: Nonce already used".into(),
        }));
    }

    // 3. ENTROPY VALIDATION LAYER
    // Reject low-entropy data to prevent resource exhaustion or malformed injection
    if !EntropyScanner::is_high_entropy(&payload.data) {
        state.blocked_entropy.fetch_add(1, Ordering::SeqCst);
        return (StatusCode::BAD_REQUEST, Json(QuantumResponse {
            status: "error".into(),
            message: "Security violation: Low entropy data rejected".into(),
        }));
    }

    // 4. POST-QUANTUM CRYPTOGRAPHIC LAYER
    // Verify the Dilithium2 signature against the message and public key
    let is_valid = QuantumCrypto::verify_signature(
        &payload.data,
        &payload.signature,
        &payload.public_key,
    );

    if !is_valid {
        // Incrementing total requests is enough here, but failed signatures are rejected
        return (StatusCode::UNAUTHORIZED, Json(QuantumResponse {
            status: "error".into(),
            message: "Cryptographic failure: Invalid Quantum Signature".into(),
        }));
    }

    // 5. SUCCESS: Register Nonce and grant access
    state.nonce_cache.insert(payload.nonce, ()).await;

    (StatusCode::OK, Json(QuantumResponse {
        status: "success".into(),
        message: "Quantum Verification Successful".into(),
    }))
}

/// Statistics API for the Dashboard
pub async fn get_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "total_requests": state.total_requests.load(Ordering::SeqCst),
        "blocked_replay": state.blocked_replay.load(Ordering::SeqCst),
        "blocked_entropy": state.blocked_entropy.load(Ordering::SeqCst),
    }))
}
