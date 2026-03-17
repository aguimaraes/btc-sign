use bitcoin::PrivateKey;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Decoded private key material from a WIF string.
///
/// Zeroizes the secret bytes on drop.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct WifKey {
    secret_bytes: [u8; 32],
}

impl WifKey {
    pub fn secret_bytes(&self) -> &[u8; 32] {
        &self.secret_bytes
    }
}

#[derive(Debug)]
pub enum WifError {
    InvalidWif(String),
    NotCompressed,
    NotMainnet,
}

impl std::fmt::Display for WifError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WifError::InvalidWif(msg) => write!(f, "invalid WIF: {}", msg),
            WifError::NotCompressed => {
                write!(
                    f,
                    "WIF key is not compressed (only compressed keys supported)"
                )
            }
            WifError::NotMainnet => {
                write!(f, "WIF key is not mainnet (only mainnet supported)")
            }
        }
    }
}

/// Decode a WIF string into private key bytes.
///
/// Only accepts compressed mainnet WIF keys (prefix K or L, 52 chars).
pub fn decode_wif(wif: &str) -> Result<WifKey, WifError> {
    let private_key = PrivateKey::from_wif(wif).map_err(|e| WifError::InvalidWif(e.to_string()))?;

    if !private_key.compressed {
        return Err(WifError::NotCompressed);
    }

    if private_key.network != bitcoin::NetworkKind::Main {
        return Err(WifError::NotMainnet);
    }

    let mut secret_bytes = [0u8; 32];
    secret_bytes.copy_from_slice(&private_key.inner.secret_bytes());

    Ok(WifKey { secret_bytes })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 7.1 — Known vector: private key = 1.
    #[test]
    fn test_decode_known_vector_one() {
        let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
        let key = decode_wif(wif).expect("valid WIF");

        let mut expected = [0u8; 32];
        expected[31] = 0x01;
        assert_eq!(key.secret_bytes(), &expected);
    }

    /// 7.2 — Known vector: private key = 2.
    #[test]
    fn test_decode_known_vector_two() {
        let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU74NMTptX4";
        let key = decode_wif(wif).expect("valid WIF");

        let mut expected = [0u8; 32];
        expected[31] = 0x02;
        assert_eq!(key.secret_bytes(), &expected);
    }

    /// 7.3 — Invalid WIF string is rejected.
    #[test]
    fn test_invalid_wif_rejected() {
        let result = decode_wif("notavalidwif");
        assert!(result.is_err());
    }

    /// 7.4 — Empty string is rejected.
    #[test]
    fn test_empty_string_rejected() {
        let result = decode_wif("");
        assert!(result.is_err());
    }

    /// 7.5 — Uncompressed WIF (5-prefix) is rejected.
    #[test]
    fn test_uncompressed_wif_rejected() {
        // Uncompressed WIF for private key = 1 (5-prefix, 51 chars).
        let wif = "5HueCGU8rMjxEXxiPuD5BDku4MkFqeZyd4dZ1jvhTVqvbTLvyTJ";
        let result = decode_wif(wif);
        assert!(matches!(result, Err(WifError::NotCompressed)));
    }

    /// 7.6 — Testnet WIF is rejected.
    #[test]
    fn test_testnet_wif_rejected() {
        // Testnet compressed WIF for private key = 1 (cN prefix).
        let wif = "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87JcbXMTcA";
        let result = decode_wif(wif);
        assert!(matches!(result, Err(WifError::NotMainnet)));
    }

    /// 7.7 — Corrupted checksum is rejected.
    #[test]
    fn test_corrupted_checksum_rejected() {
        // Valid WIF with last char changed.
        let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWm";
        let result = decode_wif(wif);
        assert!(result.is_err());
    }
}
