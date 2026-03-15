use std::io::Write;
use std::path::Path;

use bitcoin::psbt::Psbt;

#[derive(Debug)]
pub enum OutputError {
    WriteError(String),
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputError::WriteError(msg) => write!(f, "failed to write output: {}", msg),
        }
    }
}

/// Write a signed PSBT to the specified output.
///
/// If output_path is "-", writes base64 to stdout.
/// Otherwise, writes binary PSBT to the file.
pub fn write_psbt(psbt: &Psbt, output_path: &str) -> Result<(), OutputError> {
    if output_path == "-" {
        let base64 = psbt.to_string();
        let mut stdout = std::io::stdout().lock();
        writeln!(stdout, "{}", base64).map_err(|e| OutputError::WriteError(e.to_string()))?;
    } else {
        let bytes = psbt.serialize();
        let path = Path::new(output_path);

        // Refuse to overwrite existing files silently — user must remove first.
        if path.exists() {
            return Err(OutputError::WriteError(format!(
                "output file already exists: {}",
                output_path
            )));
        }

        std::fs::write(path, bytes).map_err(|e| OutputError::WriteError(e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers;

    /// 11.1 — Write to file produces valid binary PSBT.
    #[test]
    fn test_write_binary() {
        let psbt = test_helpers::make_test_psbt();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.psbt");

        write_psbt(&psbt, path.to_str().unwrap()).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        let loaded = Psbt::deserialize(&bytes).unwrap();
        assert_eq!(loaded.unsigned_tx.input.len(), 1);
    }

    /// 11.2 — Refusing to overwrite existing file.
    #[test]
    fn test_refuse_overwrite() {
        let psbt = test_helpers::make_test_psbt();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("out.psbt");

        // Write once.
        write_psbt(&psbt, path.to_str().unwrap()).unwrap();

        // Second write should fail.
        let result = write_psbt(&psbt, path.to_str().unwrap());
        assert!(result.is_err());
    }
}
