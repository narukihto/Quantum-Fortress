use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tracing::{info, warn, error};
use crate::{AppState, SecurePayload}; 
use crate::entropy::EntropyScanner;
use crate::crypto::QuantumCrypto;

/// Core Verification Pipeline
/// This handler orchestrates the security checks in a 3-phase sequence.
pub async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SecurePayload>,
) -> Result<Json<String>, StatusCode> {
    
    // Logging the start of the process 
    info!("Processing verification request for nonce: {}", payload.nonce);

    // --- PHASE 1: Replay Protection ---
    // Check if the nonce exists in the high-performance moka cache 
    if state.nonce_registry.contains_key(&payload.nonce) {
        warn!("⚠️ SECURITY ALERT: Replay attack detected! Nonce: {}", payload.nonce);
        return Err(StatusCode::CONFLICT);
    }
    // Register the nonce to block future duplicate requests 
    state.nonce_registry.insert(payload.nonce.clone(), true).await;

    // --- PHASE 2: Behavioral Entropy Analysis ---
    // Use the dynamic threshold loaded from the .env file 
    if !EntropyScanner::is_secure(payload.data.as_bytes()) {
        error!("🛑 MALICIOUS PAYLOAD: Data randomness below threshold. Potential injection/malware.");
        return Err(StatusCode::BAD_REQUEST);
    }

    // --- PHASE 3: Post-Quantum Cryptographic Verification ---
    // Verify the ML-DSA (Dilithium2) signature 
    if !QuantumCrypto::verify_signature(
        payload.data.as_bytes(),
        &payload.signature,
        &payload.public_key
    ) {
        warn!("🚫 AUTH FAILURE: Cryptographic signature mismatch for nonce: {}", payload.nonce);
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Logging successful verification 
    info!("✅ Request verified and authorized: Nonce [{}]", payload.nonce);
    
    Ok(Json("Verified_Secure_PQC".to_string()))
}
