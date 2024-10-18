<div align="center"> 
<h1>Pop DRink!</h1>

<a href="r0gue.io"><img width="100px" style="border-radius:10px;" src="https://github.com/user-attachments/assets/96830651-c3db-412a-9cb4-6fcd8ea6231b" alt="R0GUE Logo" /></a>

[![Twitter URL](https://img.shields.io/twitter/follow/Pop?style=social)](https://x.com/onpopio/)
[![Twitter URL](https://img.shields.io/twitter/follow/R0GUE?style=social)](https://twitter.com/gor0gue)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/onpopio)

Forked version of [inkdevhub/drink](https://github.com/inkdevhub/drink) for E2E testing of smart contract using the [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api) on Pop Network.

</div>

## Overview

About the repository folder structure:

- [pop-drink](/crates/pop-drink): Library for testing contracts deployed on Pop Network using drink.
- [drink](/crates/drink/drink): drink is a toolbox for ink! developers to test contracts in a sandbox environment.
- [ink-sandbox](/crates/ink-sandbox): Creates a sandbox environment for a given runtime, without having to run a node.
- [examples](/crates/drink/examples): A collection of example contracts tested with drink.
- [drink-cli](/crates/drink/drink-cli): Simple command line tool to help you play with your local contracts in a convenient way.

## Getting Started

Add `pop-drink` crate to your contract `Cargo.toml`:

```toml
drink = { version = "1.0.0",  package = "pop-drink" }
```

Set up your pop-drink test environment and write your first test.

```rs
use drink::{
    devnet::{AccountId, Balance, Runtime},
    TestExternalities,
    deploy, call,
};


// Builds your contract(s).
#[drink::contract_bundle_provider]
enum BundleProvider {}

/// Sandbox environment for Pop Devnet Runtime.
pub struct Pop {
    ext: TestExternalities,
}

// Initialising genesis state.
impl Default for Pop {
    fn default() -> Self {
        let balances: Vec<(AccountId, Balance)> = vec![(ALICE, 100u128)];
        let ext = BlockBuilder::<Runtime>::new_ext(balances);
        Self { ext }
    }
}

// Implement core functionalities for the `Pop` sandbox.
drink::impl_sandbox!(Pop, Runtime, ALICE);

// Write your first pop-drink test!
#[drink::test(sandbox = Pop)]
fn test(mut session: Session) { ... }
```

Important: run your tests with `--release`
```
cargo test --release
```

## Learn more

Dive into the ["quick start with drink!"](/crates/drink/examples/quick-start-with-drink/README.md) to learn more about drink. Also check out the other examples provided by drink.

If you are interested in learning more about testing contracts using the Pop API, check out the [example contracts](https://github.com/r0gue-io/pop-node/tree/main/pop-api/examples) tested with pop-drink!

## Support

Be part of our passionate community of Web3 builders. [Join our Telegram](https://t.me/onpopio)!

Feel free to contribute to `drink` or `pop-drink` to help improve testing ink! smart contracts! 

For any questions related to ink! you can also go to [Polkadot Stack Exchange](https://polkadot.stackexchange.com/) or ask the [ink! community](https://t.me/inkathon/1).