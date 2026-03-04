use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use tracing::{info, warn, error};
use crate::{AppState, SecurePayload}; 
use crate::entropy::EntropyScanner;
use crate::crypto::QuantumCrypto;

/// Core Verification Pipeline with Real-time Metrics Update
pub async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SecurePayload>,
) -> Result<Json<String>, StatusCode> {
    
    // Log start of the process
    info!("Processing verification request for nonce: {}", payload.nonce);

    // --- PHASE 1: Replay Protection ---
    if state.nonce_registry.contains_key(&payload.nonce) {
        warn!("⚠️ SECURITY ALERT: Replay attack detected! Nonce: {}", payload.nonce);
        
        // Update Dashboard Stats: Increment Blocked Replay
        let mut stats = state.metrics.lock().unwrap();
        stats.blocked_replay += 1;
        stats.total_requests += 1;
        
        return Err(StatusCode::CONFLICT);
    }
    state.nonce_registry.insert(payload.nonce.clone(), true).await;

    // --- PHASE 2: Behavioral Entropy Analysis ---
    if !EntropyScanner::is_secure(payload.data.as_bytes()) {
        error!("🛑 MALICIOUS PAYLOAD: Low entropy detected. Potential injection.");
        
        // Update Dashboard Stats: Increment Entropy Violations
        let mut stats = state.metrics.lock().unwrap();
        stats.blocked_entropy += 1;
        stats.total_requests += 1;
        
        return Err(StatusCode::BAD_REQUEST);
    }

    // --- PHASE 3: Post-Quantum Cryptographic Verification ---
    if !QuantumCrypto::verify_signature(
        payload.data.as_bytes(),
        &payload.signature,
        &payload.public_key
    ) {
        warn!("🚫 AUTH FAILURE: Signature mismatch for nonce: {}", payload.nonce);
        
        // Update Dashboard Stats: Count as request but failed auth
        let mut stats = state.metrics.lock().unwrap();
        stats.total_requests += 1;
        
        return Err(StatusCode::UNAUTHORIZED);
    }

    // --- SUCCESS ---
    info!("✅ Request verified and authorized: Nonce [{}]", payload.nonce);
    
    // Update Dashboard Stats: Increment Total Successful Requests
    let mut stats = state.metrics.lock().unwrap();
    stats.total_requests += 1;
    
    Ok(Json("Verified_Secure_PQC".to_string()))
}
