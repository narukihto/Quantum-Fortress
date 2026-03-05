use std::env;

pub struct EntropyScanner;

impl EntropyScanner {
    /// Calculates the Shannon Entropy of the input data (0.0 to 8.0).
    /// Used to detect low-entropy patterns like SQL injections or repeating NOP slides.
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
    /// Encrypted PQC payloads usually score > 7.0.
    pub fn is_high_entropy(data: &[u8]) -> bool {
        let threshold: f64 = env::var("MIN_ENTROPY")
            .unwrap_or_else(|_| "3.8".to_string()) // Adjusted fallback to 3.8 for better compatibility
            .parse()
            .unwrap_or(3.8);

        // Minimum length check: small payloads yield unstable entropy scores.
        if data.len() < 32 {
            return true; 
        }

        let score = Self::calculate_entropy(data);
        
        if score < threshold {
            // Use tracing or println for audit logs
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
        // 1. Predictable data (Low Entropy: 0.0)
        let low = [0u8; 64];
        
        // 2. High Entropy data (Full byte range 0-255)
        // A sequence of all unique bytes results in maximum entropy (8.0)
        let mut high = Vec::with_capacity(256);
        for i in 0..256 {
            high.push(i as u8);
        }
        
        let low_score = EntropyScanner::calculate_entropy(&low);
        let high_score = EntropyScanner::calculate_entropy(&high);
        
        println!("Test Scores -> Low: {:.2}, High: {:.2}", low_score, high_score);

        assert!(low_score < 1.0, "Low entropy data should score near 0");
        assert!(high_score > 7.5, "Full-range byte data should score near 8.0");
    }
}
