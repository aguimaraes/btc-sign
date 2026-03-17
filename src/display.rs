use std::io::Write;

use bitcoin::psbt::Psbt;
use bitcoin::Amount;

/// Display human-readable transaction details from a PSBT.
///
/// Intended for stderr output so the user can review before signing.
pub fn display_psbt(psbt: &Psbt, w: &mut impl Write) -> std::io::Result<()> {
    let tx = &psbt.unsigned_tx;

    writeln!(w, "Transaction Details")?;
    writeln!(w, "===================")?;
    writeln!(w)?;

    // Inputs.
    writeln!(w, "Inputs ({}):", tx.input.len())?;
    let mut total_in = Amount::ZERO;
    let mut all_amounts_known = true;

    for (i, input) in psbt.inputs.iter().enumerate() {
        let txin = &tx.input[i];
        write!(
            w,
            "  [{}] {}:{}",
            i, txin.previous_output.txid, txin.previous_output.vout
        )?;

        if let Some(ref utxo) = input.witness_utxo {
            writeln!(w, " — {} sat", utxo.value.to_sat())?;
            total_in = total_in
                .checked_add(utxo.value)
                .unwrap_or(Amount::MAX_MONEY);

            if let Ok(addr) =
                bitcoin::Address::from_script(&utxo.script_pubkey, bitcoin::Network::Bitcoin)
            {
                writeln!(w, "        address: {}", addr)?;
            }
        } else {
            writeln!(w, " — amount unknown (no witness_utxo)")?;
            all_amounts_known = false;
        }
    }

    writeln!(w)?;

    // Outputs.
    writeln!(w, "Outputs ({}):", tx.output.len())?;
    let mut total_out = Amount::ZERO;

    for (i, output) in tx.output.iter().enumerate() {
        total_out = total_out
            .checked_add(output.value)
            .unwrap_or(Amount::MAX_MONEY);

        if let Ok(addr) =
            bitcoin::Address::from_script(&output.script_pubkey, bitcoin::Network::Bitcoin)
        {
            writeln!(w, "  [{}] {} — {} sat", i, addr, output.value.to_sat())?;
        } else {
            writeln!(
                w,
                "  [{}] <unknown script> — {} sat",
                i,
                output.value.to_sat()
            )?;
        }
    }

    writeln!(w)?;

    // Totals and fee.
    writeln!(w, "Total in:  {} sat", total_in.to_sat())?;
    writeln!(w, "Total out: {} sat", total_out.to_sat())?;

    if all_amounts_known && total_in > Amount::ZERO {
        if let Some(fee) = total_in.checked_sub(total_out) {
            writeln!(w, "Fee:       {} sat", fee.to_sat())?;

            // Fee rate warning.
            let tx_vsize = estimate_vsize(tx.input.len(), tx.output.len());
            if tx_vsize > 0 {
                let fee_rate = fee.to_sat() / tx_vsize as u64;
                if fee_rate > 50_000 {
                    writeln!(w)?;
                    writeln!(
                        w,
                        "WARNING: Fee rate is ~{} sat/vB — this seems very high!",
                        fee_rate
                    )?;
                    writeln!(w, "         A typical fee rate is 1-100 sat/vB.")?;
                }
            }
        } else {
            writeln!(w, "Fee:       NEGATIVE (outputs exceed inputs!)")?;
        }
    } else {
        writeln!(w, "Fee:       cannot calculate (input amounts unknown)")?;
    }

    Ok(())
}

/// Estimate transaction virtual size for P2WPKH.
///
/// Overhead: ~10.5 vbytes
/// Per P2WPKH input: ~68 vbytes (with witness discount)
/// Per output: ~31 vbytes
fn estimate_vsize(num_inputs: usize, num_outputs: usize) -> usize {
    11 + num_inputs * 68 + num_outputs * 31
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_psbt() -> Psbt {
        crate::test_helpers::make_test_psbt()
    }

    /// 9.1 — Display shows input count.
    #[test]
    fn test_display_shows_inputs() {
        let psbt = make_test_psbt();
        let mut buf = Vec::new();
        display_psbt(&psbt, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Inputs (1):"));
    }

    /// 9.2 — Display shows output count.
    #[test]
    fn test_display_shows_outputs() {
        let psbt = make_test_psbt();
        let mut buf = Vec::new();
        display_psbt(&psbt, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Outputs (1):"));
    }

    /// 9.3 — Display shows fee.
    #[test]
    fn test_display_shows_fee() {
        let psbt = make_test_psbt();
        let mut buf = Vec::new();
        display_psbt(&psbt, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        // Input is 100,000 sat, output is 90,000 sat, fee is 10,000 sat.
        assert!(output.contains("Fee:       10000 sat"));
    }

    /// 9.4 — Display shows amounts.
    #[test]
    fn test_display_shows_amounts() {
        let psbt = make_test_psbt();
        let mut buf = Vec::new();
        display_psbt(&psbt, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("100000 sat"));
        assert!(output.contains("90000 sat"));
    }

    /// 9.5 — Display shows address.
    #[test]
    fn test_display_shows_address() {
        let psbt = make_test_psbt();
        let mut buf = Vec::new();
        display_psbt(&psbt, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("bc1q"));
    }

    /// 9.6 — High fee triggers warning.
    #[test]
    fn test_high_fee_warning() {
        let psbt = crate::test_helpers::make_high_fee_psbt();
        let mut buf = Vec::new();
        display_psbt(&psbt, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("WARNING"));
        assert!(output.contains("very high"));
    }
}
