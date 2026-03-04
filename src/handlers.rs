use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use std::sync::atomic::Ordering; // High-performance memory ordering
use tracing::{info, warn, error};
use crate::{AppState, SecurePayload}; 
use crate::entropy::EntropyScanner;
use crate::crypto::QuantumCrypto;

/// High-Performance Verification Pipeline
pub async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SecurePayload>,
) -> Result<Json<String>, StatusCode> {
    
    // Increment total requests immediately using Atomic fetch_add
    // This is non-blocking and thread-safe
    state.metrics.total_requests.fetch_add(1, Ordering::Relaxed);

    // --- SECURITY LAYER 0: Size Validation ---
    // Prevent Memory Bloat Attacks
    if payload.nonce.len() > 128 || payload.data.len() > 65536 {
        error!("⚠️ DOS ATTEMPT: Payload size exceeds safety limits.");
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }

    // --- PHASE 1: Replay Protection ---
    if state.nonce_registry.contains_key(&payload.nonce) {
        warn!("⚠️ SECURITY ALERT: Replay attack detected! Nonce: {}", payload.nonce);
        state.metrics.blocked_replay.fetch_add(1, Ordering::Relaxed);
        return Err(StatusCode::CONFLICT);
    }
    state.nonce_registry.insert(payload.nonce.clone(), true).await;

    // --- PHASE 2: Behavioral Entropy Analysis ---
    if !EntropyScanner::is_secure(payload.data.as_bytes()) {
        error!("🛑 MALICIOUS PAYLOAD: Low entropy detected (Potential Injection).");
        state.metrics.blocked_entropy.fetch_add(1, Ordering::Relaxed);
        return Err(StatusCode::BAD_REQUEST);
    }

    // --- PHASE 3: Post-Quantum Cryptographic Verification ---
    // This is the most CPU-intensive part; good thing we removed the Mutex locks!
    if !QuantumCrypto::verify_signature(
        payload.data.as_bytes(),
        &payload.signature,
        &payload.public_key
    ) {
        warn!("🚫 AUTH FAILURE: PQC Signature mismatch for nonce: {}", payload.nonce);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // --- SUCCESS ---
    info!("✅ PQC Verified: Nonce [{}]", payload.nonce);
    Ok(Json("Verified_Secure_PQC".to_string()))
}
