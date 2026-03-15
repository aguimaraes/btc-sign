# Dependencies

## Runtime dependencies

| Crate | Version | Purpose | Justification |
|---|---|---|---|
| `bitcoin` | 0.32 | PSBT parsing, transaction types, sighash computation, script handling, WIF decoding, address derivation | PSBT (BIP174) is a complex binary format. The `bitcoin` crate is maintained by the rust-bitcoin project and is the standard Rust implementation. Includes `secp256k1` and `bitcoin_hashes` as transitive dependencies. |
| `zeroize` | 1 | Memory zeroing of private key material | Overwrites secret bytes on drop. Same crate used in btc-keygen. |
| `clap` | 4 | CLI argument parsing | Same crate used in btc-keygen. Provides subcommand support for `inspect` and `sign`. |
| `libc` | 0.2 | Terminal echo control (Unix only) | Used to disable terminal echo when reading the WIF private key. Unix-only dependency. |

## Dev dependencies

| Crate | Version | Purpose |
|---|---|---|
| `tempfile` | 3 | Temporary files for integration tests |

## Not needed

| Crate | Why not |
|---|---|
| `getrandom` | No key generation, no randomness needed |
| `bech32` | `bitcoin` crate handles address encoding/decoding |
| `serde` / `serde_json` | No JSON output in v0.0.1 |
| `base64` | Enabled via `bitcoin`'s `base64` feature flag |
| `rpassword` | Terminal echo control implemented via `libc` directly |
| `secp256k1` | Re-exported by `bitcoin` crate |
| `bitcoin_hashes` | Re-exported by `bitcoin` crate |
