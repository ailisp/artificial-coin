#!/bin/bash
#near deploy --accountId ausd.artcoin.testnet --wasmFile ./ausd/res/ausd.wasm --initFunction new --initArgs '{"owner_id": "art.artcoin.testnet", "total_supply": "0", "art_token": "art.artcoin.testnet"}'

near deploy --accountId art.artcoin.testnet --wasmFile ./art/res/art.wasm --initFunction new --initArgs '{"owner_id": "art.artcoin.testnet", "total_supply": "1000000000000000000000000000000000", "ausd_token": "ausd.artcoin.testnet"}'

