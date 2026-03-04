use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

pub async fn verify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SecurePayload>,
) -> Json<ApiResponse> {

    // 1. Strict size validation
    if payload.nonce.len() > MAX_NONCE
        || payload.data.len() > MAX_DATA
        || payload.public_key.len() != DILITHIUM2_PK
        || payload.signature.len() != DILITHIUM2_SIG
    {
        return Json(ApiResponse {
            status: "error",
            message: "Invalid payload structure",
        });
    }

    // 2. Replay protection (atomic insert)
    if state.nonce_cache.insert(payload.nonce.clone(), ()).await.is_some() {
        return Json(ApiResponse {
            status: "error",
            message: "Replay attack detected",
        });
    }

    // 3. Entropy validation
    if !EntropyScanner::is_secure(&payload.data, MIN_ENTROPY) {
        return Json(ApiResponse {
            status: "error",
            message: "Low entropy payload",
        });
    }

    // 4. Post-Quantum signature verification
    if !QuantumCrypto::verify_signature(
        &payload.data,
        &payload.signature,
        &payload.public_key,
    ) {
        return Json(ApiResponse {
            status: "error",
            message: "Invalid signature",
        });
    }

    Json(ApiResponse {
        status: "success",
        message: "Verification passed",
    })
}
