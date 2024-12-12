# Multisig Launcher

## Mainnet Instantiate Msg

``{\"code_ids\":{\"main\":7,\"voting\":5,\"proposal\":4,\"pre_proposal\":6,\"cw4\":3}}``

## Testnet Instantiate Msg

``{\"code_ids\":{\"main\":1,\"voting\":2,\"proposal\":3,\"pre_proposal\":4,\"cw4\":5}}``

## Devnet Instantiate Msg

``{\"code_ids\":{\"main\":2,\"voting\":5,\"proposal\":4,\"pre_proposal\":3,\"cw4\":1}}``

# Deployment step by step

First store the contract
``archway contracts store msig-launcher --from my_wallet``

Then you need to instantiate the contract

``archway contracts instantiate msig-launcher --args 'PASTE_HERE_ONE_OF_THE_ABOVE_MSGS' --from my_wallet --admin "my_wallet_addr" --label "my_contract"``

Finally, we need to modify the contract's metadata.
Currently the CLI doesnt support setting the automatic withdraw function so we need to manually set it by generating a
tx.
``archwayd tx rewards set-contract-metadata msig_address --generate-only --from my_wallet --gas auto --gas-prices 900000000000aarch --gas-adjustment 1.4 > tx.json``

Now inside the generated file, set `withdraw_to_wallet` to true.

Finally run both
``archwayd tx sign tx.json --from my_wallet > signed.json``

``archwayd tx broadcast signed.json``

And the metadata should be set.