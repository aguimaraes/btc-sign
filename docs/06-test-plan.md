# Test Plan

## Unit tests (per module)

### wif.rs (7.x)
- 7.1 Known vector: private key = 1 round-trips through WIF decode
- 7.2 Known vector: private key = 2 round-trips through WIF decode
- 7.3 Invalid WIF string rejected
- 7.4 Empty string rejected
- 7.5 Uncompressed WIF (5-prefix) rejected
- 7.6 Testnet WIF rejected
- 7.7 Corrupted checksum rejected

### psbt.rs (8.x)
- 8.1 Binary PSBT round-trips through load
- 8.2 Base64 PSBT round-trips through load
- 8.3 Invalid file content rejected
- 8.4 Missing file rejected

### display.rs (9.x)
- 9.1 Display shows input count
- 9.2 Display shows output count
- 9.3 Display shows fee
- 9.4 Display shows amounts
- 9.5 Display shows address
- 9.6 High fee triggers warning

### sign.rs (10.x)
- 10.1 Signing populates partial_sigs
- 10.2 Signing with wrong key fails (NoMatchingInputs)
- 10.3 count_matching_inputs returns correct count
- 10.4 count_matching_inputs returns 0 for non-matching key
- 10.5 Signature is valid ECDSA (recompute sighash, verify)
- 10.6 Multiple matching inputs are all signed

### output.rs (11.x)
- 11.1 Write to file produces valid binary PSBT
- 11.2 Refusing to overwrite existing file

## Pipeline tests (lib.rs)
- Full round-trip: create PSBT → display → sign → serialize → deserialize → verify signature

## Integration tests (12.x)
- 12.1 inspect exits 0 and shows transaction details
- 12.2 inspect shows addresses
- 12.3 inspect with missing file exits 1
- 12.4 inspect with invalid file exits 1
- 12.5 sign with correct key and approval succeeds
- 12.6 sign to stdout produces valid base64 PSBT
- 12.7 sign aborted when user does not type "approve"
- 12.8 sign with wrong key exits 1
- 12.9 sign with invalid WIF exits 1
- 12.10 No subcommand prints usage
- 12.11 --help exits 0
- 12.12 sign creates no files other than the output
- 12.13 No network dependencies in cargo tree

## Test vectors

All tests use deterministic keys (private key = 1 and private key = 2) matching btc-keygen's test vectors:
- Private key 1: WIF `KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn`, address `bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4`
- Private key 2: WIF `KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU74NMTptX4`, address `bc1qq6hag67dl53wl99vzg42z8eyzfz2xlkvxechjp`
