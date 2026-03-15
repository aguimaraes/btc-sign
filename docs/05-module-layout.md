# Module Layout

```
src/
├── main.rs      CLI entry point: parse args, orchestrate subcommands, stderr prompts
├── lib.rs       Public module re-exports, test helpers, pipeline tests
├── wif.rs       Decode WIF string → private key bytes (ZeroizeOnDrop)
├── psbt.rs      Load PSBT from file (binary or base64)
├── display.rs   Human-readable transaction display (inputs/outputs/fee/warnings)
├── sign.rs      Sign P2WPKH inputs: sighash computation + ECDSA signing
└── output.rs    Write signed PSBT to file (binary) or stdout (base64)
```

## Data flow

```
inspect:  file → psbt::load → display::display_psbt → stderr

sign:     file → psbt::load → display::display_psbt → stderr
          stdin → wif::decode_wif → WifKey
          WifKey + Psbt → sign::count_matching_inputs → validation
          stdin → "approve" → approval check
          WifKey + Psbt → sign::sign_psbt → signed Psbt
          signed Psbt → output::write_psbt → file or stdout
          WifKey dropped → zeroize
```

## Key types

- `WifKey` — Holds 32 secret bytes with ZeroizeOnDrop. Created by `wif::decode_wif`.
- `Psbt` — From the `bitcoin` crate. Represents a BIP174 Partially Signed Bitcoin Transaction.
- `SignError` — Error enum for signing failures (no matching inputs, sighash errors).
- `PsbtLoadError` — Error enum for file read/parse failures.
- `OutputError` — Error enum for write failures.
