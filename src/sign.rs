use bitcoin::hashes::Hash;
use bitcoin::psbt::Psbt;
use bitcoin::secp256k1::{self, Message, Secp256k1};
use bitcoin::sighash::{EcdsaSighashType, SighashCache};
use bitcoin::ScriptBuf;

use crate::wif::WifKey;

#[derive(Debug)]
pub enum SignError {
    InvalidKey(String),
    NoMatchingInputs,
    NoWitnessUtxo(usize),
    NotP2wpkh(usize),
    SighashError(String),
}

impl std::fmt::Display for SignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SignError::InvalidKey(msg) => write!(f, "invalid key: {}", msg),
            SignError::NoMatchingInputs => {
                write!(f, "private key does not match any input address")
            }
            SignError::NoWitnessUtxo(i) => {
                write!(f, "input {} has no witness_utxo (required for signing)", i)
            }
            SignError::NotP2wpkh(i) => {
                write!(
                    f,
                    "input {} is not P2WPKH (only native SegWit supported)",
                    i
                )
            }
            SignError::SighashError(msg) => write!(f, "sighash computation failed: {}", msg),
        }
    }
}

/// Derive the bitcoin public key and expected P2WPKH script from a WIF key.
fn derive_pubkey_and_script(
    wif_key: &WifKey,
) -> Result<(bitcoin::PublicKey, ScriptBuf), SignError> {
    let secp = Secp256k1::new();
    let secret_key = secp256k1::SecretKey::from_slice(wif_key.secret_bytes())
        .map_err(|e| SignError::InvalidKey(e.to_string()))?;
    let private_key = bitcoin::PrivateKey::new(secret_key, bitcoin::Network::Bitcoin);
    let pubkey = private_key.public_key(&secp);
    let wpkh = pubkey
        .wpubkey_hash()
        .map_err(|_| SignError::InvalidKey("uncompressed key".to_string()))?;
    let expected_script = ScriptBuf::new_p2wpkh(&wpkh);
    Ok((pubkey, expected_script))
}

/// Count how many PSBT inputs match the given key's P2WPKH address.
pub fn count_matching_inputs(psbt: &Psbt, wif_key: &WifKey) -> Result<usize, SignError> {
    let (_, expected_script) = derive_pubkey_and_script(wif_key)?;

    let count = psbt
        .inputs
        .iter()
        .filter(|input| {
            input
                .witness_utxo
                .as_ref()
                .map(|utxo| utxo.script_pubkey == expected_script)
                .unwrap_or(false)
        })
        .count();

    Ok(count)
}

/// Sign all matching P2WPKH inputs in the PSBT.
///
/// Returns the number of inputs signed.
pub fn sign_psbt(psbt: &mut Psbt, wif_key: &WifKey) -> Result<usize, SignError> {
    let secp = Secp256k1::new();
    let secret_key = secp256k1::SecretKey::from_slice(wif_key.secret_bytes())
        .map_err(|e| SignError::InvalidKey(e.to_string()))?;
    let (pubkey, expected_script) = derive_pubkey_and_script(wif_key)?;

    let sighash_type = EcdsaSighashType::All;

    // Find matching input indices.
    let matching_indices: Vec<usize> = psbt
        .inputs
        .iter()
        .enumerate()
        .filter(|(_, input)| {
            input
                .witness_utxo
                .as_ref()
                .map(|utxo| utxo.script_pubkey == expected_script)
                .unwrap_or(false)
        })
        .map(|(i, _)| i)
        .collect();

    if matching_indices.is_empty() {
        return Err(SignError::NoMatchingInputs);
    }

    // Compute sighashes (borrows psbt.unsigned_tx immutably).
    let mut sighash_results = Vec::new();
    {
        let mut cache = SighashCache::new(&psbt.unsigned_tx);

        for &i in &matching_indices {
            let utxo = psbt.inputs[i]
                .witness_utxo
                .as_ref()
                .ok_or(SignError::NoWitnessUtxo(i))?;

            let sighash = cache
                .p2wpkh_signature_hash(i, &utxo.script_pubkey, utxo.value, sighash_type)
                .map_err(|e| SignError::SighashError(e.to_string()))?;

            sighash_results.push((i, sighash));
        }
    }

    // Sign and apply signatures.
    for (i, sighash) in sighash_results {
        let msg = Message::from_digest(sighash.to_byte_array());
        let sig = secp.sign_ecdsa(&msg, &secret_key);

        let bitcoin_sig = bitcoin::ecdsa::Signature {
            signature: sig,
            sighash_type,
        };

        psbt.inputs[i].partial_sigs.insert(pubkey, bitcoin_sig);
    }

    Ok(matching_indices.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers;
    use crate::wif::decode_wif;

    /// 10.1 — Signing populates partial_sigs.
    #[test]
    fn test_sign_populates_partial_sigs() {
        let mut psbt = test_helpers::make_test_psbt();
        let wif_key =
            decode_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn").unwrap();

        let signed = sign_psbt(&mut psbt, &wif_key).unwrap();
        assert_eq!(signed, 1);
        assert!(!psbt.inputs[0].partial_sigs.is_empty());
    }

    /// 10.2 — Signing with wrong key fails.
    #[test]
    fn test_sign_wrong_key_fails() {
        let mut psbt = test_helpers::make_test_psbt();
        // Private key = 2 does not match the PSBT inputs (key = 1).
        let wif_key =
            decode_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU74NMTptX4").unwrap();

        let result = sign_psbt(&mut psbt, &wif_key);
        assert!(matches!(result, Err(SignError::NoMatchingInputs)));
    }

    /// 10.3 — count_matching_inputs returns correct count.
    #[test]
    fn test_count_matching() {
        let psbt = test_helpers::make_test_psbt();
        let wif_key =
            decode_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn").unwrap();

        let count = count_matching_inputs(&psbt, &wif_key).unwrap();
        assert_eq!(count, 1);
    }

    /// 10.4 — count_matching_inputs returns 0 for non-matching key.
    #[test]
    fn test_count_no_match() {
        let psbt = test_helpers::make_test_psbt();
        let wif_key =
            decode_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU74NMTptX4").unwrap();

        let count = count_matching_inputs(&psbt, &wif_key).unwrap();
        assert_eq!(count, 0);
    }

    /// 10.5 — Signature is valid ECDSA.
    #[test]
    fn test_signature_verifies() {
        let mut psbt = test_helpers::make_test_psbt();
        let wif_key =
            decode_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn").unwrap();

        sign_psbt(&mut psbt, &wif_key).unwrap();

        let (pubkey, _) = derive_pubkey_and_script(&wif_key).unwrap();
        let sig = &psbt.inputs[0].partial_sigs[&pubkey];

        // Recompute sighash and verify.
        let secp = Secp256k1::new();
        let utxo = psbt.inputs[0].witness_utxo.as_ref().unwrap();
        let mut cache = SighashCache::new(&psbt.unsigned_tx);
        let sighash = cache
            .p2wpkh_signature_hash(
                0,
                &utxo.script_pubkey,
                utxo.value,
                EcdsaSighashType::All,
            )
            .unwrap();
        let msg = Message::from_digest(sighash.to_byte_array());
        let secp_pubkey = pubkey.inner;
        secp.verify_ecdsa(&msg, &sig.signature, &secp_pubkey)
            .expect("signature must verify");
    }

    /// 10.6 — Multiple matching inputs are all signed.
    #[test]
    fn test_sign_multiple_inputs() {
        let mut psbt = test_helpers::make_multi_input_psbt();
        let wif_key =
            decode_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn").unwrap();

        let signed = sign_psbt(&mut psbt, &wif_key).unwrap();
        assert_eq!(signed, 2);
        assert!(!psbt.inputs[0].partial_sigs.is_empty());
        assert!(!psbt.inputs[1].partial_sigs.is_empty());
    }
}
