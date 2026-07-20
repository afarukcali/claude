---
name: setup-nexus-wallet
description: >-
  Set up a Sui wallet with testnet gas and configure nexus-cli, from a
  completely empty environment. Use when the user wants to start building a
  Talus Agent and hasn't installed/configured nexus-cli or a Sui wallet yet,
  or when `nexus conf get` shows missing sui.pk/rpc-url/nexus.objects.
---

# Set up a Sui wallet and nexus-cli

## Steps

1. **Rust**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
   if `cargo --version` fails.

2. **Sui CLI**: point the user at
   <https://docs.sui.io/guides/developer/getting-started/sui-install> for
   their platform; verify with `sui --version`.

3. **nexus-cli**:
   ```sh
   brew tap talus-network/tap && brew install nexus-cli
   # or: cargo binstall --git https://github.com/talus-network/nexus-sdk nexus-cli
   # or: cargo install nexus-cli --git https://github.com/talus-network/nexus-sdk --tag v2.0.0-rc.4 --locked
   ```
   Verify with `nexus help`.

4. **Wallet + testnet gas**:
   ```sh
   sui client new-env --alias testnet --rpc https://fullnode.testnet.sui.io:443
   sui client switch --env testnet
   sui client new-address ed25519
   sui client switch --address <THE_ADDRESS_JUST_PRINTED>
   sui client faucet
   ```
   Use a fresh address dedicated to this project — it makes step 5
   unambiguous.

5. **Get the private key in the format nexus-cli expects.**
   `nexus conf set --sui.pk` needs **base64**, either the raw 32-byte
   ed25519 seed or Sui's own 33-byte `flag+seed` keystore form. It does
   **not** accept the `suiprivkey1...` Bech32 string `sui keytool export`
   prints by default.
   ```sh
   cat ~/.sui/sui_config/sui.keystore
   ```
   This is a JSON array of base64 strings, one per address ever created in
   that keystore — the fresh address from step 4 is the last entry. If
   unsure which entry is which, cross-check with `sui keytool list`; never
   guess.

6. **Configure nexus-cli**:
   ```sh
   nexus conf set --sui.pk <BASE64_KEY> --sui.rpc-url https://fullnode.testnet.sui.io:443
   nexus conf set --nexus.objects <path/to/objects.toml>
   nexus conf get
   ```
   The `objects.toml` (deployed Nexus package IDs for the target network)
   must come from the official Talus Nexus deployment info — never
   fabricate one.

## When done

`nexus conf get` should show `sui.pk`, `sui.rpc_url`, and `nexus` all
populated. Hand off to the `tool-new` skill next to build a Tool.

## Ground rules

- Never guess a `nexus` or `sui` CLI flag — run `<command> --help` first;
  both evolve independently of this guide.
- Never fabricate a private key, object ID, or `objects.toml` — ask the
  user or point them at the source instead.
