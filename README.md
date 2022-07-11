# Simple NFT contract

This module may serve as a demonstration of importing and implementing CW721 spec.
A small subset of base specification have been implementing so as to confirm the
proper functioning. For help with rest of the implementation, refer to [CW721-base].

[CW721-base]: https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw721-base

## Deployment on blockchain
For testing, let's use malaga testnet. Check [this](https://docs.cosmwasm.com/docs/1.0/getting-started/setting-env) page for setting up an environment for testing.

### Compile

1. To compile the smart contract for deployment, use the following(on M1 Mac):

```zsh
$ docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer-arm64:0.12.6
```

This will create a *.wasm* binary file in *artifacts/* folder.

### Deploy

2. To store the binary file on blockchain, use *wasmd*.

```zsh
$ RES=$(wasmd tx wasm store artifacts/simple_nft-aarch64.wasm --from wallet $TXFLAG -y --output json -b block)
echo $RES | jq .
```
**output**
```json
{
  "height": "614976",
  "txhash": "F8503588FCBC18F4EF22A3B52D2C3E0A15ED4A839AF92E5934F8A2EF518DA5E8",
  "data": "0A250A1E2F636F736D7761736D2E7761736D2E76312E4D736753746F7265436F6465120308B302",
  "raw_log": "[{\"events\":[{\"type\":\"message\",\"attributes\":[{\"key\":\"action\",\"value\":\"/cosmwasm.wasm.v1.MsgStoreCode\"},{\"key\":\"module\",\"value\":\"wasm\"},{\"key\":\"sender\",\"value\":\"wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf\"}]},{\"type\":\"store_code\",\"attributes\":[{\"key\":\"code_id\",\"value\":\"307\"}]}]}]",
  // more text
}
```

3. Retrieve code id for the uploaded binary.

```zsh
$ CODE_ID=$(echo $RES | jq -r '.logs[0].events[-1].attributes[0].value')
$ echo $CODE_ID
```
**output**(Might be something different for you)
```zsh
307
```

## Interaction on blockchain

### Instantiate
4. Create the InstantiateMsg

```zsh
$ INIT='{"name":"TestNFT","symbol":"TNFT"}'
```

5. Instantiate the contract

```zsh
$ wasmd tx wasm instantiate $CODE_ID $INIT --from wallet --label "Test simple NFT" $TXFLAG -y --no-admin
```
**output**
```
logs: []
raw_log: '[]'
txhash: A9B8E576F4B3AD2F183AFFB7A2E1593E00594E375B9C522AC5B0710984223C2D
```

6. Retrieve the contract address

```zsh
$ CONTRACT=$(wasmd query wasm list-contract-by-code $CODE_ID $NODE --output json | jq -r '.contracts[-1]')
$ echo $CONTRACT
```
**output**
```
wasm1ydfppp7qhh5m28zvwy2gk98j2hu8fs4ky3h4h0yt7rhwrymjqdlssh63sg
```

7. Query the contract info

```zsh
$ wasmd query wasm contract $CONTRACT $NODE
```
**output**
```zsh
address: wasm1ydfppp7qhh5m28zvwy2gk98j2hu8fs4ky3h4h0yt7rhwrymjqdlssh63sg
contract_info:
  code_id: "307"
  creator: wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf
  label: Test simple NFT
```

8. Retrive the [state](src/state.rs) of the contract. Name and symbol should with
the INIT msg. Also note the minter address is set to address that instatiated the
contract. This is to make things simpler.

```zsh
$ wasmd query wasm contract-state all $CONTRACT $NODE --output json | jq -r '.models[0].value' | base64 -d | jq .
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

9. Let's mint a new token

```zsh
$ MINT='{"mint":{"token_id":1,"owner":"wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s","token_uri":"None","price":[{"amount":"1000","denom":"umlg"}]}}'
$ echo $MINT | jq .
```
**output**
```json
{
  "mint": {
    "token_id": 1,
    "owner": "wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s",
    "token_uri": "None",
    "price": [
      {
        "amount": "1000",
        "denom": "umlg"
      }
    ]
  }
}
```

10. execute the mint command in the contract

```zsh
$ wasmd tx wasm execute $CONTRACT "$MINT" --from wallet $TXFLAG -y
```
**output**
```zsh
logs: []
raw_log: '[]'
txhash: 79C89F6CC934C2921B284458D7B0DC33AA9EFCD2A95BB5C54C217E8EF019FA3E
```

11. View the newly minted [token info](src/contract.rs)

```zsh
$ wasmd query wasm contract-state all $CONTRACT $NODE --output json | jq -r '.models[0].value' | base64 -d | jq .
```
**output**
```json
{
  "owner": "wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s",
  "approvals": null,
  "base_price": [
    {
      "denom": "umlg",
      "amount": "1000"
    }
  ],
  "token_uri": null,
  "token_id": 1
}
```

12. Let's mint another token

```zsh
$ MINT='{"mint":{"token_id":2,"owner":"wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf","token_uri":"None","price":[{"amount":"1000","denom":"umlg"}]}}'
$ echo $MINT | jq .
```
**output**
```json
{
  "mint": {
    "token_id": 2,
    "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
    "token_uri": "None",
    "price": [
      {
        "amount": "1000",
        "denom": "umlg"
      }
    ]
  }
}
```

13. Execute mint command

```zsh
$ wasmd tx wasm execute $CONTRACT "$MINT" --from wallet $TXFLAG -y
```
**output**
```zsh
gas estimate: 160602
logs: []
raw_log: '[]'
txhash: 5AE13707AD8C944607FEF23B43648F983CFBE298724A3EA4966934254BABB984
```

14. View another newly minted token
```zsh
$ wasmd query wasm contract-state all $CONTRACT $NODE --output json | jq -r '.models[1].value' | base64 -d | jq .
```
**output**
```json
{
  "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
  "approvals": null,
  "base_price": [
    {
      "denom": "umlg",
      "amount": "1000"
    }
  ],
  "token_uri": null,
  "token_id": 2
}
```

15. Transfer token 2 to another owner

```zsh
$ EXECUTE='{"transfer_nft":{"recipient":"wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s","token_id":2}}'
$ echo $EXECUTE | jq .
```
**output**
```json
{
  "transfer_nft": {
    "recipient": "wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s",
    "token_id": 2
  }
}
```

16. Execute transfer command

```zsh
$ wasmd tx wasm execute $CONTRACT "$EXECUTE" --from wallet $TXFLAG -y --output json | jq .
```
**output**
```json
{
  "txhash": "5CD670F4EF8A836E8AFDC28E84CFCEC4046174AA63C6899409856EF9C756B500",
  "raw_log": "[]",
  "logs": []
}
```

17. Both the tokens should have the same owner now.
```zsh
$ wasmd query wasm contract-state all $CONTRACT $NODE --output json | jq -r '.models[0].value' | base64 -d | jq .
```
**output**
```json
{
  "owner": "wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s",
  "approvals": null,
  "base_price": [
    {
      "denom": "umlg",
      "amount": "1000"
    }
  ],
  "token_uri": null,
  "token_id": 1
}
```

```zsh
$ wasmd query wasm contract-state all $CONTRACT $NODE --output json | jq -r '.models[1].value' | base64 -d | jq .
```
**output**
```json
{
  "owner": "wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s",
  "approvals": null,
  "base_price": [
    {
      "denom": "umlg",
      "amount": "1000"
    }
  ],
  "token_uri": null,
  "token_id": 2
}
```
### Query