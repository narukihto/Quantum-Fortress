use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::{AppState, crypto::QuantumCrypto, entropy::EntropyScanner};

const MAX_NONCE: usize = 128;
const MAX_DATA: usize = 65_536;
const DILITHIUM2_PK: usize = 1312;
const DILITHIUM2_SIG: usize = 2420;
const MIN_ENTROPY: f64 = 4.5;

#[derive(Deserialize)]
pub struct SecurePayload {
    pub nonce: String,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: &'static str,
    pub message: &'static str,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub total_requests: usize,
    pub blocked_replay: usize,
    pub blocked_entropy: usize,
}

pub async fn verify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SecurePayload>,
) -> Json<ApiResponse> {
    state.total_requests.fetch_add(1, Ordering::SeqCst);

    // 1. Size Validation
    if payload.nonce.len() > MAX_NONCE || payload.data.len() > MAX_DATA 
        || payload.public_key.len() != DILITHIUM2_PK || payload.signature.len() != DILITHIUM2_SIG {
        return Json(ApiResponse { status: "error", message: "Invalid payload structure" });
    }

    // 2. Replay Protection
    if state.nonce_cache.insert(payload.nonce.clone(), ()).await.is_some() {
        state.blocked_replay.fetch_add(1, Ordering::SeqCst);
        return Json(ApiResponse { status: "error", message: "Replay attack detected" });
    }

    // 3. Entropy Check
    if !EntropyScanner::is_secure(&payload.data, MIN_ENTROPY) {
        state.blocked_entropy.fetch_add(1, Ordering::SeqCst);
        return Json(ApiResponse { status: "error", message: "Low entropy payload" });
    }

    // 4. Quantum Signature Verification
    if !QuantumCrypto::verify_signature(&payload.data, &payload.signature, &payload.public_key) {
        return Json(ApiResponse { status: "error", message: "Invalid signature" });
    }

    Json(ApiResponse { status: "success", message: "Verification passed" })
}

pub async fn get_stats(State(state): State<Arc<AppState>>) -> Json<StatsResponse> {
    Json(StatsResponse {
        total_requests: state.total_requests.load(Ordering::SeqCst),
        blocked_replay: state.blocked_replay.load(Ordering::SeqCst),
        blocked_entropy: state.blocked_entropy.load(Ordering::SeqCst),
    })
}
