use std::path::Path;

use bitcoin::psbt::Psbt;

#[derive(Debug)]
pub enum PsbtLoadError {
    ReadError(String),
    ParseError(String),
}

impl std::fmt::Display for PsbtLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PsbtLoadError::ReadError(msg) => write!(f, "failed to read PSBT file: {}", msg),
            PsbtLoadError::ParseError(msg) => write!(f, "failed to parse PSBT: {}", msg),
        }
    }
}

/// Load and parse a PSBT from a file.
///
/// Tries binary format first, then base64.
pub fn load(path: &Path) -> Result<Psbt, PsbtLoadError> {
    let bytes = std::fs::read(path).map_err(|e| PsbtLoadError::ReadError(e.to_string()))?;

    // Try binary first.
    if let Ok(psbt) = Psbt::deserialize(&bytes) {
        return Ok(psbt);
    }

    // Try base64.
    if let Ok(text) = std::str::from_utf8(&bytes) {
        if let Ok(psbt) = text.trim().parse::<Psbt>() {
            return Ok(psbt);
        }
    }

    Err(PsbtLoadError::ParseError(
        "file is neither valid binary PSBT nor valid base64 PSBT".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn make_test_psbt() -> Psbt {
        crate::test_helpers::make_test_psbt()
    }

    /// 8.1 — Binary PSBT round-trips through load.
    #[test]
    fn test_load_binary() {
        let psbt = make_test_psbt();
        let bytes = psbt.serialize();

        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(&bytes).unwrap();

        let loaded = load(tmp.path()).expect("should load binary PSBT");
        assert_eq!(loaded.unsigned_tx.input.len(), 1);
        assert_eq!(loaded.unsigned_tx.output.len(), 1);
    }

    /// 8.2 — Base64 PSBT round-trips through load.
    #[test]
    fn test_load_base64() {
        let psbt = make_test_psbt();
        let base64 = psbt.to_string();

        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "{}", base64).unwrap();

        let loaded = load(tmp.path()).expect("should load base64 PSBT");
        assert_eq!(loaded.unsigned_tx.input.len(), 1);
    }

    /// 8.3 — Invalid file is rejected.
    #[test]
    fn test_load_invalid() {
        let mut tmp = NamedTempFile::new().unwrap();
        tmp.write_all(b"this is not a psbt").unwrap();

        let result = load(tmp.path());
        assert!(result.is_err());
    }

    /// 8.4 — Missing file is rejected.
    #[test]
    fn test_load_missing_file() {
        let result = load(Path::new("/tmp/btc-sign-nonexistent.psbt"));
        assert!(matches!(result, Err(PsbtLoadError::ReadError(_))));
    }
}
