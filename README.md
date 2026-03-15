# btc-sign

> **ALPHA — v0.0.1** — This software has not been independently audited. Use at your own risk. Verify the source code before trusting it with real funds.

Minimal offline Bitcoin transaction signer for cold storage. Designed for air-gapped machines.

## What it does

Signs Bitcoin PSBTs (BIP174) offline using a WIF private key. Nothing else.

- Parses PSBT from file (binary or base64)
- Displays transaction details: inputs, outputs, fee
- Signs P2WPKH (native SegWit) inputs with ECDSA
- Requires explicit "approve" before signing
- Outputs signed PSBT to file or stdout
- Zeroizes private key material on exit
- Zero network code — no dependencies on networking crates

## Workflow

1. **Online machine** (Bitcoin Core): create unsigned PSBT with `walletcreatefundedpsbt`
2. **USB transfer**: copy `.psbt` file to air-gapped machine
3. **Air-gapped machine** (btc-sign): inspect, verify, sign
4. **USB transfer**: copy signed `.psbt` back
5. **Online machine** (Bitcoin Core): `finalizepsbt` + `sendrawtransaction`

## Installation

### Pre-built binaries

Download from [Releases](https://github.com/aguimaraes/btc-sign/releases). Verify SHA256 checksums.

### Build from source

```
cargo build --release
cp target/release/btc-sign /usr/local/bin/
```

## Usage

### Inspect a PSBT (read-only)

```
btc-sign inspect tx.psbt
```

Shows inputs, outputs, amounts, addresses, and fee on stderr. No key needed.

### Sign a PSBT

```
btc-sign sign tx.psbt --output signed.psbt
```

1. Displays transaction details on stderr
2. Prompts for WIF private key (no echo on terminal)
3. Verifies key matches at least one input
4. Prompts: type "approve" to sign
5. Writes signed PSBT to `signed.psbt`

### Sign and pipe to stdout (base64)

```
btc-sign sign tx.psbt --output -
```

Outputs base64-encoded signed PSBT to stdout, for use with:

```
bitcoin-cli finalizepsbt "$(btc-sign sign tx.psbt --output -)"
```

## Security

- WIF key read from stdin only — never in CLI args
- Terminal echo disabled during key input (Unix)
- Transaction details displayed before signing
- Explicit "approve" required — not just y/n
- Fee rate warning when >50,000 sat/vB
- Key-address match verified before signing
- Private key bytes zeroized on drop
- No network code, no state, no config files

See [docs/](docs/) for threat model, security assumptions, and design documentation.

## Companion tool

[btc-keygen](https://github.com/aguimaraes/btc-keygen) — Minimal offline Bitcoin key generator for cold storage.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE), at your option.
