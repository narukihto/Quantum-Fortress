use pqcrypto_dilithium::dilithium2;
use pqcrypto_traits::sign::{PublicKey, SecretKey};

fn main() {
    // 1. Generate a Post-Quantum Keypair (Public and Secret)
    // Dilithium2 is used for its balance of security and performance
    let (pk, sk) = dilithium2::keypair();

    // 2. Convert keys into byte arrays for storage or transmission
    let pk_bytes = pk.as_bytes();
    let sk_bytes = sk.as_bytes();

    println!("✅ Quantum Keys Generated Successfully!");
    println!("--------------------------------------");
    println!("Public Key Size: {} bytes", pk_bytes.len()); // Standard: 1312
    println!("Secret Key Size: {} bytes", sk_bytes.len()); // Standard: 2528
    println!("--------------------------------------");

    // CRITICAL SECURITY NOTE: 
    // In a production environment, 'sk_bytes' (Secret Key) must be stored 
    // in a Hardware Security Module (HSM) or a secure vault. 
    // Never expose the Secret Key in logs or unsecured databases.
}
