pub struct EntropyScanner;

impl EntropyScanner {
    /// Calculates the Shannon Entropy of the input data.
    /// Returns a value between 0.0 (totally predictable) and 8.0 (pure randomness).
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
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Determines if the data is "random enough" to be a valid PQC ciphertext/signature.
    /// Standard PQC data usually has entropy > 7.5.
    pub fn is_high_entropy(data: &[u8]) -> bool {
        // We set the threshold at 4.0 to allow for varied data while blocking
        // obvious patterns or repetitive "filler" bytes.
        let score = Self::calculate_entropy(data);
        score > 4.0
    }
}
