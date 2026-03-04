use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{PublicKey, Signature};

pub struct QuantumCrypto;

impl QuantumCrypto {
    /// Verifies an ML-DSA (Dilithium2) signature.
    pub fn verify_signature(msg: &[u8], sig_bytes: &[u8], pk_bytes: &[u8]) -> bool {
        let pk = match dilithium2::PublicKey::from_bytes(pk_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };
        let sig = match dilithium2::Signature::from_bytes(sig_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };
        dilithium2::verify(&sig, msg, &pk).is_ok()
    }
}
