use pqcrypto_dilithium::dilithium2;
// Essential traits for byte-to-struct conversion
use pqcrypto_traits::sign::PublicKey as _; 
use pqcrypto_traits::sign::DetachedSignature as _;

pub struct QuantumCrypto;

impl QuantumCrypto {
    /// Verifies a Dilithium2 post-quantum signature.
    /// Returns true only if the signature is valid for the given message and public key.
    pub fn verify_signature(
        message: &[u8],
        signature_bytes: &[u8],
        public_key_bytes: &[u8],
    ) -> bool {
        // 1. Length validation (Fast-fail)
        // Dilithium2 Public Keys are 1312 bytes, Signatures are 2420 bytes.
        if public_key_bytes.len() != dilithium2::public_key_bytes() || 
           signature_bytes.len() != dilithium2::signature_bytes() {
            return false;
        }

        // 2. Parse Public Key from bytes
        let pk = match dilithium2::PublicKey::from_bytes(public_key_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };

        // 3. Parse Detached Signature from bytes
        let sig = match dilithium2::DetachedSignature::from_bytes(signature_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // 4. Cryptographic Verification
        dilithium2::verify_detached_signature(&sig, message, &pk).is_ok()
    }
}
