# Non-Goals

btc-sign deliberately does NOT:

1. **Create PSBTs** — Use Bitcoin Core's `createpsbt` or `walletcreatefundedpsbt`.
2. **Broadcast transactions** — Use Bitcoin Core's `finalizepsbt` + `sendrawtransaction`.
3. **Estimate fees** — Fee estimation requires network access. Set fees when creating the PSBT.
4. **Support multi-sig** — v0.0.1 supports single-key P2WPKH only.
5. **Support legacy scripts** — Only native SegWit (P2WPKH/bech32). No P2PKH, P2SH, or P2SH-P2WPKH.
6. **Support testnet/regtest** — Mainnet only. Use Bitcoin Core for testnet operations.
7. **Generate keys** — Use btc-keygen for key generation.
8. **Connect to the network** — Zero network code, same as btc-keygen.
9. **Store state** — No config files, no wallet files, no databases.
10. **Display QR codes** — Future consideration, not in v0.0.1.
