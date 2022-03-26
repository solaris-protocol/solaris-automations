## Project description
Solaris automations is a way to automate your DeFi operations using on-chain or off-chain conditional limit orders with callbacks

## Short example
Let's say we have Alice and Bob. 
Alice has a position in a lending/borrowing protocol with 1 SOL as collateral and 80 USDC debt, that is close to liquidation.
She already spent her 80 USDC and has no money to repay her debt to get her collateral back.
Using Solaris automations Alice can create a 0.8 SOL/80 USDC limit order and send it to Bob. 
When Bob will start filling it and will send 80 USDC to Solaris automations contract, 
we can use these 80 USDC in a callback to repay Alice's debt, withdraw her collateral and send 0.8 SOL to Bob and 0.2 SOL to Alice.

So Alice's position will not be liquidated and she will not lose 5% of her collateral.
Also Alice can specify a price point after which her order can be filled. So if SOL price will go app, her position will stay as is and she will be able to borrow more USDC.

## How it works

<img width="1920" alt="image" src="https://raw.githubusercontent.com/solaris-protocol/solaris-automations/main/how_it_works_predicate_plus_callback.png">

1. Alice creates [liquidation protection order](https://github.com/solaris-protocol/solaris-automations/blob/solend_integration/cli_main/order_base_2.json), signs it and publishes it on-chain or off-chain

2. Bob tries to fill this order by invoking fill_order instruction in Solaris Automations program

3. Solaris Automations checks Alice's signature and predicate by invoking predicate instruction from order

4. Predicate instruction gets data from an oracle and checks if it satisfies stop-loss condition

5. If predicate instruction returns true Bob's assets are transfered to Solaris Automations token account and callback instructions are executed

6. Alice's assets transferred to Solaris Automations contract token account and then to Bob's account

## Use example
If you want to test Solaris yourself then you need to follow this steps.

1. Create `maker_keypair.json` and `taker_keypair.json` with following content:
```
maker_keypair.json
[249,174,85,194,173,37,129,121,109,93,213,116,225,113,98,210,27,60,56,30,218,250,206,153,191,194,34,10,178,200,31,22,107,227,238,11,176,71,2,2,180,249,247,164,233,17,65,74,188,229,231,3,249,105,139,148,211,172,52,242,117,159,133,174]

taker_keypair.json
[131,170,163,183,179,179,133,212,44,82,92,234,168,214,89,50,185,201,223,20,74,86,8,152,164,112,106,102,52,249,220,253,34,63,10,90,219,233,26,167,154,170,55,153,227,216,94,125,246,239,152,231,30,245,47,232,124,113,125,70,170,217,206,34]
```

2. Build `sol-auto` CLI with [cargo](https://doc.rust-lang.org/book/ch01-01-installation.html).

```
$ cd cli_main
```
```
$ cargo build
```

3. Create [liquidation protection order](https://github.com/solaris-protocol/solaris-automations/blob/solend_integration/cli_main/order_base_2.json). 

`order_base.json` is actually not order. It's just human-readable description of order. In this example order we give opportunity to close our debt in lending protocol to another wallet. Before create order we need to borrow USDC in [solend(devnet)](https://devnet.solend.fi/dashboard) from `maker_keypair.json`.

Don't forget to change `"maker"` field in `order_base_2.json`. You need to put there path to your `maker_keypair.json`. 

Then execute
```
$ target/debug/./sol-auto --settings settings.json create_order order_base_2.json
```

This command creates [order_test.json](https://github.com/solaris-protocol/solaris-automations/blob/main/cli_main/order_test.json). This `order_test.json` you can send Bob to execute it or upload onchain if you don't want to do this order private.

4. For current version if your order have got `callback` then contract can execute it only with 2 transaction. First transaction upload order onchain. Order can be uploaded with taker or maker transactoin sign. Second transaction do swaps and execute callback. It requires `taker` as signer. 

Our example order have got `callback` so first transaction upload order onchain. Don't forget to change `"payer_keypair"` field in settings.json.
```
$ target/debug/./sol-auto --settings settings.json fill_order order_test.json
```

5. If order with `callback` upload onchain then we can do the same command to execute it.
