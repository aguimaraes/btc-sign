# Security Assumptions

btc-sign assumes:

1. **The machine is air-gapped.** No network interface is active. PSBT files arrive and depart via USB only.

2. **The OS is not compromised.** btc-sign trusts the OS to properly isolate process memory and handle terminal I/O. Memory zeroization is best-effort — a compromised OS could still read process memory.

3. **The PSBT was created correctly.** btc-sign trusts the PSBT structure and `witness_utxo` values. A malicious PSBT with incorrect `witness_utxo` amounts could mislead the fee display. The on-chain transaction is ultimately validated by Bitcoin consensus rules, not by btc-sign.

4. **libsecp256k1 is correct.** ECDSA signing uses the same libsecp256k1 library maintained by the Bitcoin Core project. We trust its implementation of RFC 6979 deterministic nonce generation and secp256k1 curve operations.

5. **The `zeroize` crate works as documented.** Secret bytes are overwritten on drop. Compiler optimizations that might elide zeroing are mitigated by `zeroize`'s use of volatile writes.

6. **The user reads the displayed transaction details.** btc-sign shows inputs, outputs, and fees before signing. The security model depends on the user actually verifying this information.

7. **The WIF key was generated securely.** btc-sign does not validate the entropy quality of the imported key. Use btc-keygen or another audited tool.
