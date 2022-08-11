# Tether Token

The source code of Tether USDT contract.

## Packages

rust : `curl https://sh.rustup.rs -sSf | sh`   or manually https://forge.rust-lang.org/infra/other-installation-methods.html

near-cli : `npm i -g near-cli`


## MetaData
Meta data can be set after deploy, however it can also be adjusted before compile/build by modifing the details listed under the function `pub fn new_default_meta` on line 66 in `src/lib.rs` 

## Build

Add Rust `wasm32` target:
```bash
rustup target add wasm32-unknown-unknown
```
Build the contract:

```bash
cargo build --target wasm32-unknown-unknown --release
```

```bash
cargo test
```

## Deploy

### On `sandbox`:

Install sandbox:

```bash
npm install -g near-sandbox
near-sandbox --home /tmp/near-sandbox init
near-sandbox --home /tmp/near-sandbox run
```

Deploy:

```bash
$ near deploy --wasmFile target/wasm32-unknown-unknown/release/tether_token.wasm --initFunction new_default_meta --initArgs '{"owner_id": "usdt.near", "1000000000000000000"}' --accountId test.near --networkId sandbox --nodeUrl http://0.0.0.0:3030 --keyPath /tmp/near-sandbox/validator_key.json
```

### On `mainnet`:

#### Note on address ownership/deploy:
`--accountId=` : This will be the address the contract is deployed to and the community will use to interact with the token. It should have ~ 25 Near to deploy the contract (1 Near / 100kb , deployed contract ~ 2MB)


`owner_id`  : This should be the Multi Safe multisig address for admin management


```bash
$ near deploy --wasmFile target/wasm32-unknown-unknown/release/tether_token.wasm --initFunction new_default_meta --initArgs '{"owner_id": "tether-admin-id.multisafe.near", "total_supply":  "0"}' --accountId=usdt.tether-token.near --networkId=mainnet --nodeUrl=https://rpc.mainnet.near.org

```
