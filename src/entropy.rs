use std::env;

pub struct EntropyScanner;

impl EntropyScanner {
    /// Calculates the Shannon Entropy of the input data (0.0 to 8.0).
    /// Professional entropy analysis for detecting malformed payloads or injections.
    pub fn calculate_entropy(data: &[u8]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let mut counts = [0usize; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in counts.iter() {
            if count > 0 {
                let p = count as f64 / len;
                // Shannon Entropy Formula: H(X) = -Σ p(x) log2 p(x)
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Determines if the data meets the security threshold.
    /// Standard PQC (Dilithium) usually scores > 7.0.
    /// SQL injections or repetitive patterns usually score < 3.0.
    pub fn is_high_entropy(data: &[u8]) -> bool {
        // 1. Get threshold from .env or fallback to 4.0
        let threshold: f64 = env::var("MIN_ENTROPY")
            .unwrap_or_else(|_| "4.0".to_string())
            .parse()
            .unwrap_or(4.0);

        // 2. Minimum length check: payloads smaller than 32 bytes 
        // often give unstable entropy scores.
        if data.len() < 32 {
            return true; // Assume safe if tiny, or implement stricter check
        }

        let score = Self::calculate_entropy(data);
        
        // Log low entropy for security auditing (Optional)
        if score < threshold {
            println!("🛡️ Security Alert: Low entropy data detected ({:.2})", score);
        }

        score > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shannon_entropy() {
        // Predictable data (Low Entropy)
        let low = b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        // Random data (High Entropy)
        let high = b"8f3a9d2c1b5e4f7a0d9c8b7a6f5e4d3c2b1a0f9e8d7c6b5a4f3e2d1c0b9a8f7";
        
        assert!(EntropyScanner::calculate_entropy(low) < 1.0);
        assert!(EntropyScanner::calculate_entropy(high) > 4.0);
    }
}
