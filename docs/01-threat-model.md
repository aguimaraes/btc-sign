# Threat Model

## What btc-sign protects against

1. **Signing on a compromised machine** — btc-sign is designed for air-gapped use. The machine running btc-sign should have no network access. PSBT files are transferred via USB.

2. **Key exposure via process arguments** — The WIF private key is read from stdin, never from CLI arguments. This prevents exposure via `ps`, `/proc/*/cmdline`, or shell history.

3. **Key persistence in memory** — Private key material is zeroized on drop using the `zeroize` crate. This reduces the window where key bytes exist in process memory.

4. **Blind signing** — Transaction details (inputs, outputs, fee) are displayed on stderr before any signing occurs. The user must type "approve" to proceed.

5. **Fee manipulation** — Unusually high fee rates (>50,000 sat/vB) trigger a warning. This catches common mistakes like confusing satoshi and BTC values.

6. **Key-address mismatch** — btc-sign verifies the WIF key matches at least one PSBT input before prompting for approval. This prevents signing with the wrong key.

## What btc-sign does NOT protect against

- Physical access to the air-gapped machine
- Side-channel attacks on the CPU (timing, power analysis)
- Compromised hardware (keyloggers, screen capture)
- Malicious PSBT files (beyond standard parsing validation)
- Social engineering (user approves a transaction they don't understand)
- Memory forensics after process exit (zeroize is best-effort on modern OSes)
