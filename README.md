# Simple NFT contract

This module may serve as a demonstration of importing and implementing CW721 spec.
A small subset of base specification have been implementing so as to confirm the
proper functioning. For help with rest of the implementation, refer to [CW721-base].

[CW721-base]: https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-base

## Deployment on blockchain
For testing, let's use malaga testnet. Check [this](https://docs.cosmwasm.com/docs/1.0/getting-started/setting-env) page for setting up an environment for testing.

### Compile
To compile the smart contract for deployment, use the following(on M1 Mac):

```zsh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer-arm64:0.12.6
```

This will create a *.wasm* binary file in *artifacts/* folder.

### Deploy
To store the binary file on blockchain, use *wasmd*.

```zsh
RES=$(wasmd tx wasm store artifacts/simple_nft-aarch64.wasm --from wallet $TXFLAG -y --output json -b block)
echo $RES | jq .
```
**output**
```json
{
  "height": "614976",
  "txhash": "F8503588FCBC18F4EF22A3B52D2C3E0A15ED4A839AF92E5934F8A2EF518DA5E8",
  "data": "0A250A1E2F636F736D7761736D2E7761736D2E76312E4D736753746F7265436F6465120308B302",
  "raw_log": "[{\"events\":[{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/cosmwasm.wasm.v1.MsgStoreCode\"},{\"key\":\"module\",\"value\":\"wasm\"},{\"key\":\"sender\",\"value\":\"wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf\"}]},{\"type\":\"store_code\",\"attributes\":[{\"key\":\"code_id\",\"value\":\"307\"}]}]}]",
  .
  .
  .
}
```

```zsh
CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
echo $CODE_ID
```
**output**(Might be something different for you)
```zsh
307
```

## Interaction on blockchain

### Instantiate
```zsh
INIT='{"name":"TestNFT","symbol":"TNFT"}'
```

```zsh
wasmd tx wasm instantiate $CODE_ID $INIT --from wallet --label "Test simple NFT" $TXFLAG -y --no-admin
```
**output**
```
logs: []
raw_log: '[]'
txhash: A9B8E576F4B3AD2F183AFFB7A2E1593E00594E375B9C522AC5B0710984223C2D
```

```zsh
CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
echo $CONTRACT
```
**output**
```
wasm1ydfppp7qhh5m28zvwy2gk98j2hu8fs4ky3h4h0yt7rhwrymjqdlssh63sg
```

```zsh
wasmd query wasm contract $CONTRACT $NODE
```
**output**
```zsh
address: wasm1ydfppp7qhh5m28zvwy2gk98j2hu8fs4ky3h4h0yt7rhwrymjqdlssh63sg
contract_info:
  code_id: "307"
  creator: wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf
  label: Test simple NFT
```

```zsh
wasmd query wasm contract-state all $CONTRACT $NODE --output json | jq -r '.models[0].value' | base64 -d | jq .
```
**output**
```json
{
  "name": "TestNFT",
  "symbol": "TNFT",
  "minter": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
  "num_tokens": 0
}
```

### Execute

### Query