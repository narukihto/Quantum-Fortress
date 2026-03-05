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
use sha3::{Sha3_256, Digest};
use tokio::io::AsyncWriteExt;

// 1. Generate Bindings from the ABI file
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

// --- QUANTUM VERIFICATION ---
pub async fn verify(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<QuantumRequest>,
) -> impl IntoResponse {
    state.total_requests.fetch_add(1, Ordering::SeqCst);

    if state.nonce_cache.get(&payload.nonce).await.is_some() {
        state.blocked_replay.fetch_add(1, Ordering::SeqCst);
        return (StatusCode::CONFLICT, Json(QuantumResponse {
            status: "blocked".into(),
            message: "Replay attack detected: Nonce conflict.".into(),
        })).into_response();
    }
    state.nonce_cache.insert(payload.nonce, ()).await;

    if !EntropyScanner::is_high_entropy(&payload.data) {
        state.blocked_entropy.fetch_add(1, Ordering::SeqCst);
        return (StatusCode::BAD_REQUEST, Json(QuantumResponse {
            status: "rejected".into(),
            message: "Payload rejected: Low entropy pattern.".into(),
        })).into_response();
    }

    if QuantumCrypto::verify_signature(&payload.data, &payload.signature, &payload.public_key) {
        (StatusCode::OK, Json(QuantumResponse {
            status: "verified".into(),
            message: "Quantum signature validated successfully.".into(),
        })).into_response()
    } else {
        (StatusCode::UNAUTHORIZED, Json(QuantumResponse {
            status: "failed".into(),
            message: "Invalid quantum signature.".into(),
        })).into_response()
    }
}

// --- BLOCKCHAIN ANCHORING ---
pub async fn anchor_content(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<AnchorResponse>, (StatusCode, String)> {
    let mut file_name = String::from("unknown");
    let mut data = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() == Some("file") {
            file_name = field.file_name().unwrap_or("unnamed").to_string();
            let bytes = field.bytes().await.map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
            data = bytes.to_vec();
        }
    }

    if data.is_empty() { 
        return Err((StatusCode::BAD_REQUEST, "File is empty or missing".into())); 
    }

    let hash_bytes: [u8; 32] = Sha3_256::digest(&data).into();
    let hash_hex = hex::encode(hash_bytes);

    // Fixed Lifetime Error E0716 by using explicit variable binding
    let contract_call = state.contract.anchor_content(hash_bytes);
    let pending_tx = contract_call.send().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Chain Error: {}", e)))?;
    
    let receipt = pending_tx.await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Transaction failed to reach finality".to_string()))?;

    let log_entry = format!("{},{}\n", file_name, hash_hex);
    if let Ok(mut f) = tokio::fs::OpenOptions::new().append(true).create(true).open(&state.registry_path).await {
        let _ = f.write_all(log_entry.as_bytes()).await;
    }

    Ok(Json(AnchorResponse {
        status: "success".into(),
        content_hash: hash_hex,
        tx_hash: format!("{:?}", receipt.transaction_hash),
    }))
}

// --- MONITORING ---
pub async fn get_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({
        "total_requests": state.total_requests.load(Ordering::SeqCst),
        "blocked_replay": state.blocked_replay.load(Ordering::SeqCst),
        "blocked_entropy": state.blocked_entropy.load(Ordering::SeqCst),
        "system_status": "Operational",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
