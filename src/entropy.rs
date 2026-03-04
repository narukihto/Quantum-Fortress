pub struct EntropyScanner;

impl EntropyScanner {
    /// Calculates Shannon Entropy to detect non-random or malicious patterns.
    pub fn calculate_shannon_entropy(data: &[u8]) -> f64 {
        if data.is_empty() { return 0.0; }
        let mut frequencies = [0usize; 256];
        for &byte in data { frequencies[byte as usize] += 1; }

        let len = data.len() as f64;
        frequencies.iter()
            .filter(|&&count| count > 0)
            .map(|&count| {
                let p = count as f64 / len;
                -p * p.log2()
            })
            .sum()
    }

    pub fn is_secure(data: &[u8]) -> bool {
        Self::calculate_shannon_entropy(data) > 3.2
    }
}
