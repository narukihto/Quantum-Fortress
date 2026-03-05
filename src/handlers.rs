use axum::{
    extract::{Multipart, State},
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
use ethers::prelude::*;
use sha3::{Sha3_256, Digest};
use tokio::io::AsyncWriteExt;

// 1. Generate Bindings (Ensure IntegrityLedger.json is in your root directory)
ethers::prelude::abigen!(VeriPhysContract, "./IntegrityLedger.json");

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

#[derive(Serialize)]
pub struct AnchorResponse {
    pub status: String,
    pub content_hash: String,
    pub tx_hash: String,
}

// --- QUANTUM VERIFICATION HANDLER ---
pub async fn verify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QuantumRequest>,
) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::SeqCst);

    // 1. Replay Attack Prevention (Temporal Gate)
    if state.nonce_cache.get(&payload.nonce).await.is_some() {
        state.blocked_replay.fetch_add(1, Ordering::SeqCst);
        return (StatusCode::CONFLICT, Json(QuantumResponse {
            status: "blocked".into(),
            message: "Replay attack detected: Nonce has already been consumed.".into(),
        })).into_response();
    }
    state.nonce_cache.insert(payload.nonce, ()).await;

    // 2. Behavioral Entropy Analysis (Injection Shield)
    if !EntropyScanner::is_high_entropy(&payload.data) {
        state.blocked_entropy.fetch_add(1, Ordering::SeqCst);
        return (StatusCode::BAD_REQUEST, Json(QuantumResponse {
            status: "rejected".into(),
            message: "Payload rejected: Low entropy pattern detected (Potential Malformed Data).".into(),
        })).into_response();
    }

    // 3. Post-Quantum Signature Validation (Dilithium2)
    let is_valid = QuantumCrypto::verify_signature(
        &payload.data,
        &payload.signature,
        &payload.public_key,
    );

    if is_valid {
        (StatusCode::OK, Json(QuantumResponse {
            status: "verified".into(),
            message: "Quantum-grade signature validated successfully.".into(),
        })).into_response()
    } else {
        (StatusCode::UNAUTHORIZED, Json(QuantumResponse {
            status: "failed".into(),
            message: "Quantum signature invalid: Cryptographic integrity compromise.".into(),
        })).into_response()
    }
}

// --- BLOCKCHAIN ANCHORING HANDLER ---
pub async fn anchor_content(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<AnchorResponse>, (StatusCode, String)> {
    let mut file_name = String::from("unknown");
    let mut data = Vec::new();

    // Secure extraction of multipart data stream
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            file_name = field.file_name().unwrap_or("unnamed").to_string();
            data = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?.to_vec();
        }
    }

    if data.is_empty() { 
        return Err((StatusCode::BAD_REQUEST, "Payload Error: Zero-byte file rejected.".into())); 
    }

    // 1. Generate SHA3-256 Physical Fingerprint (FIPS 202)
    let hash_bytes: [u8; 32] = Sha3_256::digest(&data).into();
    let hash_hex = hex::encode(hash_bytes);

    // 2. Immutable Anchoring via Ethers-rs
    let tx = state.contract.anchor_content(hash_bytes).send().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Blockchain Ledger Error: {}", e)))?;
    
    let receipt = tx.await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Transaction failed to reach finality.".to_string()))?;

    // 3. Local Audit Log (Async Non-blocking)
    let log_entry = format!("{},{}\n", file_name, hash_hex);
    let mut f = tokio::fs::OpenOptions::new().append(true).create(true).open(&state.registry_path).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    f.write_all(log_entry.as_bytes()).await.ok();

    Ok(Json(AnchorResponse {
        status: "success".into(),
        content_hash: hash_hex,
        tx_hash: format!("{:?}", receipt.transaction_hash),
    }))
}

// --- SYSTEM STATISTICS HANDLER ---
pub async fn get_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "total_requests": state.total_requests.load(Ordering::SeqCst),
        "blocked_replay": state.blocked_replay.load(Ordering::SeqCst),
        "blocked_entropy": state.blocked_entropy.load(Ordering::SeqCst),
        "system_status": "Operational",
        "pqc_standard": "Dilithium2 (ML-DSA)"
    }))
}
