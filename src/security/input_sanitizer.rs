use anyhow::Result;
use std::collections::HashSet;

pub struct InputSanitizer {
    max_data_size: usize,
    blacklisted_opcodes: HashSet<u8>,
    dangerous_patterns: Vec<Vec<u8>>,
}

impl InputSanitizer {
    pub fn new() -> Self {
        let mut blacklisted_opcodes = HashSet::new();
        
        // Add dangerous opcodes
        blacklisted_opcodes.insert(0xff); // SELFDESTRUCT
        blacklisted_opcodes.insert(0x32); // ORIGIN (can be dangerous in some contexts)
        
        let mut dangerous_patterns = Vec::new();
        
        // Add known malicious patterns
        dangerous_patterns.push(b"selfdestruct".to_vec());
        dangerous_patterns.push(b"delegatecall".to_vec());

        Self {
            max_data_size: 100_000, // 100KB max
            blacklisted_opcodes,
            dangerous_patterns,
        }
    }

    pub fn validate_call_data(&self, data: &[u8]) -> Result<()> {
        // Check data size
        if data.len() > self.max_data_size {
            return Err(anyhow::anyhow!("Call data too large"));
        }

        // Check for dangerous opcodes
        for &byte in data {
            if self.blacklisted_opcodes.contains(&byte) {
                return Err(anyhow::anyhow!("Dangerous opcode detected: {:#04x}", byte));
            }
        }

        // Check for dangerous patterns
        for pattern in &self.dangerous_patterns {
            if data.windows(pattern.len()).any(|window| window == pattern.as_slice()) {
                return Err(anyhow::anyhow!("Dangerous pattern detected"));
            }
        }

        Ok(())
    }

    pub fn validate_message(&self, message: &[u8]) -> Result<()> {
        // Check message length
        if message.len() > 10_000 {
            return Err(anyhow::anyhow!("Message too long"));
        }

        // Check for null bytes (potential injection)
        if message.contains(&0) {
            return Err(anyhow::anyhow!("Message contains null bytes"));
        }

        // Check for non-printable characters in reasonable messages
        let printable_count = message.iter().filter(|&&b| b >= 32 && b <= 126).count();
        let printable_ratio = printable_count as f64 / message.len() as f64;
        
        if printable_ratio < 0.8 && message.len() > 10 {
            return Err(anyhow::anyhow!("Message contains too many non-printable characters"));
        }

        Ok(())
    }

    pub fn sanitize_string(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_ascii() && !c.is_control())
            .take(1000) // Limit length
            .collect()
    }

    pub fn validate_address_string(&self, address: &str) -> Result<()> {
        // Check format
        if !address.starts_with("0x") {
            return Err(anyhow::anyhow!("Invalid address format"));
        }

        // Check length (40 hex chars + 0x prefix)
        if address.len() != 42 {
            return Err(anyhow::anyhow!("Invalid address length"));
        }

        // Check hex characters
        let hex_part = &address[2..];
        if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(anyhow::anyhow!("Invalid hex characters in address"));
        }

        Ok(())
    }
}
