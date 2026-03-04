use pqcrypto_dilithium::dilithium2;

pub struct QuantumCrypto;

impl QuantumCrypto {
    pub fn verify_signature(
        message: &[u8],
        signature_bytes: &[u8],
        public_key_bytes: &[u8],
    ) -> bool {

        let pk = match dilithium2::PublicKey::from_bytes(public_key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let sig = match dilithium2::DetachedSignature::from_bytes(signature_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        dilithium2::verify_detached_signature(&sig, message, &pk).is_ok()
    }
}
