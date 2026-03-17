pub mod display;
pub mod output;
pub mod psbt;
pub mod sign;
pub mod wif;

/// Test helpers for creating valid PSBTs with known keys.
#[cfg(test)]
pub mod test_helpers {
    use bitcoin::absolute::LockTime;
    use bitcoin::blockdata::script::ScriptBuf;
    use bitcoin::blockdata::transaction::{OutPoint, Sequence, Transaction, TxIn, TxOut};
    use bitcoin::blockdata::witness::Witness;
    use bitcoin::psbt::Psbt;
    use bitcoin::secp256k1::{self, Secp256k1};
    use bitcoin::transaction::Version;
    use bitcoin::{Amount, Network, PrivateKey, Txid};

    /// Private key = 1 (same test vector as btc-keygen).
    /// WIF: KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn
    /// Address: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
    fn test_keypair() -> (bitcoin::PublicKey, ScriptBuf) {
        let secp = Secp256k1::new();
        let mut key_bytes = [0u8; 32];
        key_bytes[31] = 0x01;
        let secret_key = secp256k1::SecretKey::from_slice(&key_bytes).unwrap();
        let private_key = PrivateKey::new(secret_key, Network::Bitcoin);
        let pubkey = private_key.public_key(&secp);
        let wpkh = pubkey.wpubkey_hash().unwrap();
        let script = ScriptBuf::new_p2wpkh(&wpkh);
        (pubkey, script)
    }

    /// Create a simple test PSBT: 1 input (100k sat), 1 output (90k sat), fee 10k sat.
    /// Input is from private key = 1's address.
    pub fn make_test_psbt() -> Psbt {
        let (_, input_script) = test_keypair();

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

    /// Create a PSBT with a very high fee to trigger the fee warning.
    /// Input: 21,000,000 sat, output: 1,000 sat, fee: 20,999,000 sat.
    pub fn make_high_fee_psbt() -> Psbt {
        let (_, input_script) = test_keypair();

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
                value: Amount::from_sat(1_000),
                script_pubkey: input_script.clone(),
            }],
        };

        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(21_000_000),
            script_pubkey: input_script,
        });

        psbt
    }

    /// Create a PSBT with 2 inputs from the same key.
    pub fn make_multi_input_psbt() -> Psbt {
        let (_, input_script) = test_keypair();

        let txid1: Txid = "0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .unwrap();
        let txid2: Txid = "0000000000000000000000000000000000000000000000000000000000000002"
            .parse()
            .unwrap();

        let tx = Transaction {
            version: Version::TWO,
            lock_time: LockTime::ZERO,
            input: vec![
                TxIn {
                    previous_output: OutPoint {
                        txid: txid1,
                        vout: 0,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness::default(),
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: txid2,
                        vout: 0,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                    witness: Witness::default(),
                },
            ],
            output: vec![TxOut {
                value: Amount::from_sat(180_000),
                script_pubkey: input_script.clone(),
            }],
        };

        let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(100_000),
            script_pubkey: input_script.clone(),
        });
        psbt.inputs[1].witness_utxo = Some(TxOut {
            value: Amount::from_sat(100_000),
            script_pubkey: input_script,
        });

        psbt
    }
}

#[cfg(test)]
mod pipeline_tests {
    use crate::test_helpers;
    use crate::wif;

    /// Full round-trip: create PSBT, sign, verify signature.
    #[test]
    fn test_full_round_trip() {
        let mut psbt = test_helpers::make_test_psbt();
        let wif_key =
            wif::decode_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn").unwrap();

        // Inspect.
        let mut buf = Vec::new();
        crate::display::display_psbt(&psbt, &mut buf).unwrap();
        let display = String::from_utf8(buf).unwrap();
        assert!(display.contains("100000 sat"));
        assert!(display.contains("90000 sat"));
        assert!(display.contains("10000 sat"));

        // Sign.
        let signed = crate::sign::sign_psbt(&mut psbt, &wif_key).unwrap();
        assert_eq!(signed, 1);

        // Verify PSBT has signature.
        assert!(!psbt.inputs[0].partial_sigs.is_empty());

        // Verify serialization round-trip.
        let bytes = psbt.serialize();
        let loaded = bitcoin::psbt::Psbt::deserialize(&bytes).unwrap();
        assert!(!loaded.inputs[0].partial_sigs.is_empty());
    }
}
