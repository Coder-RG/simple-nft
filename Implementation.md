# Executing and Querying using wasmd

For the following commands, four addresses have been used:
- [wallet] wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf

- [wallet2] wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s

- [wallet3] wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm

- [wallet4] wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54

Keep note which address is being used at various places, like *owner* and *operator*.

## Table of Content
1. [Execute](#execute)
   1. [Mint](#mint)
   2. [Approve](#approve)
   3. [Transfer](#transfer)
   4. [Revoke](#revoke)
   5. [Approve all](#approve-all)
   6. [Revoke all](#revoke-all)
2. [Query](#query)
   1. [Asking price](#asking-price)
   2. [Owner of](#owner-of)
   3. [Approval](#approval)
   4. [Approvals](#approvals)
   5. [All Operator](#all-operator)
   6. [Number of tokens](#number-of-tokens)
   7. [NFT info](#nft-info)
   8. [All NFT info](#all-nft-info)
   9. [Contract Info](#contract-info)

## Execute

### Mint

**token 1**

```zsh
MINT='{"mint":{"owner":"wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf","token_uri":"test://token1","price”:[{“amount":"10000","denom":"umlg"}]}}'
```
```json
{
  "mint": {
    "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
    "token_uri": "test://token1",
    "price": [
      {
        "amount": "1000",
        "denom": "umlg"
      }
    ]
  }
}
```
```sh
wasmd tx wasm execute $CONTRACT $MINT --from wallet $TXFLAG -y -b block --output json
```

**token 2**

```zsh
MINT='{"mint":{"owner":"wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf","token_uri":"test://token2","price":[{"amount":"10000","denom":"umlg"}]}}'
```
```zsh
wasmd tx wasm execute $CONTRACT $MINT --from wallet $TXFLAG -y -b block --output json
```

### Approve
**Approval 1**

```zsh
APPROVE='{"approve":{"operator":"wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54","token_id":1,"expires":{"at_height": 900000}}}'
```
```json
{
  "approve": {
    "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
    "token_id": 1,
    "expires": {
      "at_height": 900000
    }
  }
}
```
```zsh
wasmd tx wasm execute $CONTRACT $APPROVE --from wallet $TXFLAG -y --output json | jq .
```

**Approval 2**
```zsh
APPROVE='{"approve":{"operator":"wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm","token_id":1,"expires":null}}'
```
```json
{
  "approve": {
    "operator": "wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm",
    "token_id": 1,
    "expires": null
  }
}
```
```zsh
wasmd tx wasm execute $CONTRACT $APPROVE --from wallet $TXFLAG -y --output json | jq . 
```

### Transfer
```zsh
TRANSFER='{"transfer_nft":{"recipient":"wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s","token_id":1}}'
```
```json
{
  "transfer_nft": {
    "recipient": "wasm1qka2er800suxsy7y9yz9wqgt8p3ktw5ptpf28s",
    "token_id": 1
  }
}
```
```zsh
wasmd tx wasm execute $CONTRACT $TRANSFER --from wallet4 $TXFLAG -y --output json | jq .
```

### Revoke
```zsh
REVOKE='{"revoke":{"operator":"wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54","token_id":2}}'
```
```zsh
wasmd tx wasm execute $CONTRACT $REVOKE --from wallet3 $TXFLAG -y --output json | jq .
```
**Before revoking approval**
```json
{
  "data": {
    "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
    "approvals": [
      {
        "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
        "expires": {
          "never": {}
        }
      }
    ]
  }
}
```
**After revoking approval**
```json
{
  "data": {
    "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
    "approvals": []
  }
}
```

### Approve all
```zsh
APPROVE_ALL='{"approve_all":{"operator":"wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54", "expires":{"at_height":890000}}}'
```
```json
{
  "approve_all": {
    "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
    "expires": {
      "at_height": 890000
    }
  }
}
```
```zsh
wasmd tx wasm execute $CONTRACT $APPROVE_ALL --from wallet $TXFLAG -y --output json | jq .
```
```zsh
APPROVE_ALL='{"approve_all":{"operator":"wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm", "expires":{"at_height":886820}}}'
```

**Querying all operators for **:
```json
{
  "data": {
    "operators": [
      {
        "spender": "wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm",
        "expires": {
          "at_height": 886820
        }
      },
      {
        "spender": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
        "expires": {
          "at_height": 890000
        }
      }
    ]
  }
}
```

### Revoke all
```zsh
REVOKE_ALL='{"revoke_all":{"operator":"wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54"}}'
```
```json
{
  "revoke_all": {
    "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54"
  }
}
```
```zsh
wasmd tx wasm execute $CONTRACT $REVOKE_ALL --from wallet $TXFLAG -y --output json | jq .
```

**Operators after this command**
```json
{
  "data": {
    "operators": [
      {
        "spender": "wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm",
        "expires": {
          "at_height": 886820
        }
      }
    ]
  }
}
```

## Query

### Asking price
This is not part of CW721 spec.
```zsh
PRICE='{"asking_price":{"token_id":1}}'
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $PRICE $NODE --output json | jq .
```
```json
{
  "data": {
    "price": [
      {
        "denom": "umlg",
        "amount": "1000"
      }
    ]
  }
}
```

### Owner of
```zsh
OWNER='{"owner_of":{"token_id":1}}'
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $OWNER $NODE --output json | jq .
```
```json
{
  "data": {
    "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
    "approvals": [
      {
        "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
        "expires": {
          "at_height": 900000
        }
      },
      {
        "operator": "wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm",
        "expires": {
          "never": {}
        }
      }
    ]
  }
}
```

### Approval
```zsh
APPROVAL='{"approval":{"token_id":1,"operator":"wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54"}}'
```
```json
{
  "approval": {
    "token_id": 1,
    "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54"
  }
}
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $APPROVAL $NODE --output json | jq .
```
```json
{
  "data": {
    "approval": {
      "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
      "expires": {
        "at_height": 900000
      }
    }
  }
}
```

### Approvals
```zsh
APPROVALS='{"approvals":{"token_id":1,"include_expired":true}}'
```
```json
{
  "approvals": {
    "token_id": 1,
    "include_expired": true
  }
}
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $APPROVALS $NODE --output json | jq .
```
```json
{
  "data": {
    "approvals": [
      {
        "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
        "expires": {
          "at_height": 900000
        }
      },
      {
        "operator": "wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm",
        "expires": {
          "never": {}
        }
      }
    ]
  }
}
```

### All Operator
```zsh
ALL_OPERATORS='{"all_operators":{"owner":"wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf","include_expired":true}}'
```
```json
{
  "all_operators": {
    "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
    "include_expired": true
  }
}
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $ALL_OPERATORS $NODE --output json | jq .
```
```json
{
  "data": {
    "operators": [
      {
        "spender": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
        "expires": {
          "at_height": 890000
        }
      }
    ]
  }
}
```

### Number of tokens
```zsh
NUM_TOKENS='{"num_tokens":{}}'
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $NUM_TOKENS $NODE --output json | jq .
```
```json
{
  "data": {
    "tokens": 2
  }
}
```

### NFT info

```zsh
NFTNFO='{"nft_info":{"token_id":1}}'
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $NFTNFO $NODE --output json | jq .
```
```json
{
  "data": {
    "token_uri": "test://token1"
  }
}
```

### All NFT info
```zsh
ALLNFTINFO='{"all_nft_info":{"token_id":1}}' 
```
```zsh
wasmd query wasm contract-state smart $CONTRACT $ALLNFTINFO $NODE --output json | jq .
```
```json
{
  "data": {
    "owner": {
      "owner": "wasm1g9urk8rj9news03dv7wfckcu49a6yk8z5rldwf",
      "approvals": [
        {
          "operator": "wasm197r20d43mch8tuzaa8h7mmshnveex75rs2zt54",
          "expires": {
            "at_height": 900000
          }
        },
        {
          "operator": "wasm10macmllfdsf9dkmgd6sxmcpv8umgkdq8e4rmrm",
          "expires": {
            "never": {}
          }
        }
      ]
    },
    "info": {
      "token_uri": "test://token1"
    }
  }
}
```

### Contract Info

```zsh
CONTRACT_INFO='{"contract_info":{}}'
```
```rs
wasmd query wasm contract-state smart $CONTRACT $CONTRACT_INFO $NODE --output json | jq .
```
```json
{
  "data": {
    "name": "TestNFT",
    "symbol": "TNFT"
  }
}
```

