pub async fn verify_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SecurePayload>,
) -> Result<Json<String>, StatusCode> {
    
    info!("Incoming request: Nonce [{}]", payload.nonce);

    // 1. Anti-Replay with Logging
    if state.nonce_registry.contains_key(&payload.nonce) {
        warn!("⚠️ SECURITY ALERT: Replay attack blocked! Nonce: {}", payload.nonce);
        return Err(StatusCode::CONFLICT);
    }
    state.nonce_registry.insert(payload.nonce.clone(), true).await;

    // 2. Entropy Check with Dynamic Threshold
    if !EntropyScanner::is_secure(payload.data.as_bytes(), state.config.entropy_threshold) {
        error!("🛑 MALICIOUS PAYLOAD: Low entropy detected. Blocking request.");
        return Err(StatusCode::BAD_REQUEST);
    }

    // 3. Cryptographic Proof
    if !QuantumCrypto::verify_signature(...) {
        warn!("🚫 AUTH FAILURE: Invalid PQC signature for nonce {}", payload.nonce);
        return Err(StatusCode::UNAUTHORIZED);
    }

    info!("✅ Request verified and authorized: Nonce [{}]", payload.nonce);
    Ok(Json("Verified_Secure".into()))
}
