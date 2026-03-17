use std::io::Write;
use std::process::{Command, Stdio};

use bitcoin::blockdata::script::ScriptBuf;
use bitcoin::blockdata::transaction::{OutPoint, Sequence, Transaction, TxIn, TxOut};
use bitcoin::blockdata::witness::Witness;
use bitcoin::psbt::Psbt;
use bitcoin::secp256k1::{self, Secp256k1};
use bitcoin::transaction::Version;
use bitcoin::{absolute::LockTime, Amount, Network, PrivateKey, Txid};

fn btc_sign_bin() -> String {
    env!("CARGO_BIN_EXE_btc-sign").to_string()
}

/// Create a test PSBT file on disk, returning the path.
fn create_test_psbt_file(dir: &tempfile::TempDir) -> std::path::PathBuf {
    let psbt = make_test_psbt();
    let path = dir.path().join("test.psbt");
    std::fs::write(&path, psbt.serialize()).unwrap();
    path
}

fn make_test_psbt() -> Psbt {
    let secp = Secp256k1::new();
    let mut key_bytes = [0u8; 32];
    key_bytes[31] = 0x01;
    let secret_key = secp256k1::SecretKey::from_slice(&key_bytes).unwrap();
    let private_key = PrivateKey::new(secret_key, Network::Bitcoin);
    let pubkey = private_key.public_key(&secp);
    let wpkh = pubkey.wpubkey_hash().unwrap();
    let input_script = ScriptBuf::new_p2wpkh(&wpkh);

    let txid: Txid = "0000000000000000000000000000000000000000000000000000000000000001"
        .parse()
        .unwrap();

    let tx = Transaction {
        version: Version::TWO,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: OutPoint { txid, vout: 0 },
            script_sig: ScriptBuf::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::default(),
        }],
        output: vec![TxOut {
            value: Amount::from_sat(90_000),
            script_pubkey: input_script.clone(),
        }],
    };

    let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
    psbt.inputs[0].witness_utxo = Some(TxOut {
        value: Amount::from_sat(100_000),
        script_pubkey: input_script,
    });

    psbt
}

// ── inspect subcommand ──────────────────────────────────────────────

/// 12.1 — inspect exits 0 and shows transaction details on stderr.
#[test]
fn test_inspect_success() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);

    let output = Command::new(btc_sign_bin())
        .args(["inspect", psbt_path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Inputs (1):"));
    assert!(stderr.contains("Outputs (1):"));
    assert!(stderr.contains("100000 sat"));
    assert!(stderr.contains("90000 sat"));
    assert!(stderr.contains("Fee:"));
}

/// 12.2 — inspect shows addresses.
#[test]
fn test_inspect_shows_addresses() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);

    let output = Command::new(btc_sign_bin())
        .args(["inspect", psbt_path.to_str().unwrap()])
        .output()
        .unwrap();

    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("bc1q"));
}

/// 12.3 — inspect with missing file exits 1.
#[test]
fn test_inspect_missing_file() {
    let output = Command::new(btc_sign_bin())
        .args(["inspect", "/tmp/btc-sign-nonexistent.psbt"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("error"));
}

/// 12.4 — inspect with invalid file exits 1.
#[test]
fn test_inspect_invalid_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.psbt");
    std::fs::write(&path, b"not a psbt").unwrap();

    let output = Command::new(btc_sign_bin())
        .args(["inspect", path.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(!output.status.success());
}

// ── sign subcommand ─────────────────────────────────────────────────

/// 12.5 — sign with correct key and approval succeeds.
#[test]
fn test_sign_success() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);
    let output_path = dir.path().join("signed.psbt");

    let mut child = Command::new(btc_sign_bin())
        .args([
            "sign",
            psbt_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
        )
        .unwrap();
        writeln!(stdin, "approve").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify signed PSBT is valid and has signatures.
    let signed_bytes = std::fs::read(&output_path).unwrap();
    let signed_psbt = Psbt::deserialize(&signed_bytes).unwrap();
    assert!(!signed_psbt.inputs[0].partial_sigs.is_empty());
}

/// 12.6 — sign to stdout produces base64.
#[test]
fn test_sign_stdout_base64() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);

    let mut child = Command::new(btc_sign_bin())
        .args(["sign", psbt_path.to_str().unwrap(), "--output", "-"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
        )
        .unwrap();
        writeln!(stdin, "approve").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());

    // Stdout should be valid base64 PSBT.
    let stdout = String::from_utf8(output.stdout).unwrap();
    let parsed: Psbt = stdout
        .trim()
        .parse()
        .expect("stdout should be valid base64 PSBT");
    assert!(!parsed.inputs[0].partial_sigs.is_empty());
}

/// 12.7 — sign aborted when user does not type "approve".
#[test]
fn test_sign_abort_no_approve() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);
    let output_path = dir.path().join("signed.psbt");

    let mut child = Command::new(btc_sign_bin())
        .args([
            "sign",
            psbt_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
        )
        .unwrap();
        writeln!(stdin, "no").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("aborted"));
    assert!(!output_path.exists());
}

/// 12.8 — sign with wrong key (no matching inputs) exits 1.
#[test]
fn test_sign_wrong_key() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);
    let output_path = dir.path().join("signed.psbt");

    let mut child = Command::new(btc_sign_bin())
        .args([
            "sign",
            psbt_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        // Private key = 2 does not match the PSBT input (key = 1).
        writeln!(
            stdin,
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU74NMTptX4"
        )
        .unwrap();
        writeln!(stdin, "approve").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("does not match"));
}

/// 12.9 — sign with invalid WIF exits 1.
#[test]
fn test_sign_invalid_wif() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);
    let output_path = dir.path().join("signed.psbt");

    let mut child = Command::new(btc_sign_bin())
        .args([
            "sign",
            psbt_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(stdin, "notavalidwif").unwrap();
        writeln!(stdin, "approve").unwrap();
    }

    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
}

// ── CLI structure ───────────────────────────────────────────────────

/// 12.10 — no subcommand prints help.
#[test]
fn test_no_subcommand() {
    let output = Command::new(btc_sign_bin()).output().unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Usage") || stderr.contains("usage"));
}

/// 12.11 — --help exits 0.
#[test]
fn test_help() {
    let output = Command::new(btc_sign_bin()).arg("--help").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("inspect"));
    assert!(stdout.contains("sign"));
}

// ── statelessness ───────────────────────────────────────────────────

/// 12.12 — sign creates no files other than the output.
#[test]
fn test_no_extra_files() {
    let dir = tempfile::tempdir().unwrap();
    let psbt_path = create_test_psbt_file(&dir);
    let output_path = dir.path().join("signed.psbt");

    let before: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();

    let mut child = Command::new(btc_sign_bin())
        .args([
            "sign",
            psbt_path.to_str().unwrap(),
            "--output",
            output_path.to_str().unwrap(),
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    {
        let stdin = child.stdin.as_mut().unwrap();
        writeln!(
            stdin,
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn"
        )
        .unwrap();
        writeln!(stdin, "approve").unwrap();
    }

    child.wait_with_output().unwrap();

    let after: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();

    // Only one new file: signed.psbt.
    assert_eq!(after.len(), before.len() + 1);
}

// ── no network dependencies ─────────────────────────────────────────

/// 12.13 — cargo tree has no network crates.
#[test]
fn test_no_network_dependencies() {
    let output = Command::new("cargo")
        .args(["tree", "--prefix", "none"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();

    let tree = String::from_utf8(output.stdout).unwrap();
    let banned = [
        "reqwest",
        "hyper",
        "tokio",
        "async-std",
        "surf",
        "ureq",
        "curl",
    ];
    for crate_name in banned {
        assert!(
            !tree.contains(crate_name),
            "dependency tree contains banned network crate: {}",
            crate_name
        );
    }
}
