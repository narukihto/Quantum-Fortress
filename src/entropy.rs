pub struct EntropyScanner;

impl EntropyScanner {
    pub fn is_secure(data: &[u8], threshold: f64) -> bool {
        if data.is_empty() {
            return false;
        }

        let mut freq = [0usize; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;

        for &count in freq.iter() {
            if count == 0 {
                continue;
            }
            let p = count as f64 / len;
            entropy -= p * p.log2();
        }

        entropy >= threshold
    }
}
