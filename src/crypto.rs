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
#[cfg(test)]
mod security_audit_tests {
    use super::*;
    use pqcrypto_dilithium::dilithium2;
    use pqcrypto_traits::sign::{PublicKey, SecretKey};

    #[test]
    fn test_quantum_integrity_protection() {
        // 1. Setup: Generate a valid Post-Quantum Keypair (Dilithium2)
        let (pk, sk) = dilithium2::keypair();
        let message = b"Critical Transaction: Transfer 1000 BTC";
        let sig = dilithium2::detached_sign(message, &sk);

        // 2. Success Scenario: Valid signature must pass verification
        assert!(QuantumCrypto::verify_signature(
            message,
            sig.as_bytes(),
            pk.as_bytes()
        ), "🛡️ Audit Failed: Valid PQC signature was rejected!");

        // 3. Forgery Attack: Changing a single byte in the message (Data Integrity Test)
        // Description: Attacker attempts to modify the transaction amount after it was signed.
        let tampered_message = b"Critical Transaction: Transfer 9999 BTC";
        assert!(!QuantumCrypto::verify_signature(
            tampered_message,
            sig.as_bytes(),
            pk.as_bytes()
        ), "🚨 Security Breach: Tampered message was accepted by the gateway!");

        // 4. Man-in-the-middle (MITM) / Identity Theft: Using a different public key
        // Description: Attacker tries to verify a signature using their own public key instead of the sender's.
        let (attacker_pk, _) = dilithium2::keypair();
        assert!(!QuantumCrypto::verify_signature(
            message,
            sig.as_bytes(),
            attacker_pk.as_bytes()
        ), "🚨 Security Breach: Signature verified with an unauthorized public key!");

        // 5. Malformed Signature Attack: Sending random/zeroed data as a signature
        // Description: Attacker attempts to crash the system or bypass security with invalid signature formatting.
        let fake_sig = vec![0u8; dilithium2::signature_bytes()]; 
        assert!(!QuantumCrypto::verify_signature(
            message,
            &fake_sig,
            pk.as_bytes()
        ), "🚨 Security Breach: Malformed/Null signature was not caught by the validator!");

        println!("✅ Security Audit Passed: All Quantum Attack Vectors Blocked.");
    }
}
