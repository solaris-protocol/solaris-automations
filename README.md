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

