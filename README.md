<div align="center"> 
<h1>Pop DRink!</h1>

<a href="r0gue.io"><img width="100px" style="border-radius:10px;" src="https://github.com/user-attachments/assets/96830651-c3db-412a-9cb4-6fcd8ea6231b" alt="R0GUE Logo" /></a>

[![Twitter URL](https://img.shields.io/twitter/follow/Pop?style=social)](https://x.com/onpopio/)
[![Twitter URL](https://img.shields.io/twitter/follow/R0GUE?style=social)](https://twitter.com/gor0gue)
[![Telegram](https://img.shields.io/badge/Telegram-gray?logo=telegram)](https://t.me/onpopio)

Forked version of [inkdevhub/drink](https://github.com/inkdevhub/drink) for E2E testing smart contract using [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api) with [Pop Network runtimes](https://github.com/r0gue-io/pop-node/tree/main/runtime).

</div>

## Overview

About the repository folder structure:

- [pop-drink](/crates/pop-drink): Provides utility methods for testing contract methods with DRink! and Pop Network runtimes.
- [examples](/crates/drink/examples): A collection of example contracts tested with DRink!
- [drink](/crates/drink/drink): DRink! is a toolbox for ink! developers that allows for a fully functional ink! contract development without any running node.
- [ink-sandbox](/crates/ink-sandbox): Sandbox refers to an isolated runtime environment that simulates the behavior of a full node, without requiring an actual node.
- [drink-cli](/crates/drink/drink-cli): Simple command line tool to help you play with your local contracts in a convenient way.

## Getting Started

Add `pop-drink` crate to your contract `Cargo.toml`:

```toml
drink = { version = "1.0.0",  package = "pop-drink" }
```

### Setup a testing environment

Please see ["Quick start with DRink!"](/crates/drink/examples/quick-start-with-drink/README.md) for a detailed explanation.

Add the below code at the top of your contract test file to setup [Sandbox](TODO) for the [**Pop Network Devnet**](https://github.com/r0gue-io/pop-node/tree/main/runtime/devnet) runtime.

```rs
#[derive(Default)]
struct Sandbox;

// Implement `Sandbox` environment for the Pop Network Devnet runtime.
drink::impl_sandbox!(Sandbox, drink::devnet::Runtime, ALICE);
```

## Writing tests

### Writing tests

Your typical test module will look like (See ["Quick start with DRink!"](/crates/drink/examples/quick-start-with-drink/README.md) example tests):

```rust
#[cfg(test)]
mod tests {
    use drink::session::{Session, NO_ARGS, None, NO_SALT};

    #[drink::contract_bundle_provider]
    enum BundleProvider {}

    #[drink::test]
    fn deploy_and_call_a_contract(mut session: Session) -> Result<(), Box<dyn Error>> {
        let result: bool = session
            .deploy_bundle_and(BundleProvider::local(), "new", &["true"], NO_SALT, None)?
            .call_and("flip", NO_ARGS, None)?
            .call_and("flip", NO_ARGS, None)?
            .call_and("flip", NO_ARGS, None)?
            .call("get", NO_ARGS, None)??;
        assert_eq!(result, false);
    }
}
```

So, firstly, you declare a bundle provider like:

```rust
#[drink::contract_bundle_provider]
enum BundleProvider {}
```

It will take care of building all contract dependencies in the compilation phase and gather all contract bundles into a single registry.
Then, you will be able to get a contract bundle by calling:

```rust
let bundle = BundleProvider::local()?; // for the contract from the current crate
let bundle = BundleProvider::Flipper.bundle()?; // for the contract from the `flipper` crate
```

We mark each testcase with `#[drink::test]` attribute and declare return type as `Result` so that we can use the `?` operator:

```rust
#[drink::test]
fn testcase() -> Result<(), Box<dyn Error>> {
    // ...
}
```

Then, we can use the `Session` API to interact with both contracts and the whole runtime.
For details, check out testcases in [lib.rs](lib.rs).

### Writing tests for methods using [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api)

Pop DRink! also provides [utilitiy methods](/crates/pop-drink/src/lib.rs) that you can use to test your contracts. This example interacts with a [PSP22 example contract](https://github.com/r0gue-io/pop-node/blob/main/pop-api/examples/fungibles/lib.rs) that uses [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api). The contract method returns [`PSP22Error`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/v0/fungibles/errors.rs#L73C1-L73C22) custom error which is provided by Pop API library.

```rs
// Import required methods and types.
use drink::{call, session::Session};
use pop_api::v0::fungibles::PSP22Error;

// Interact with `transfer` method of the `Psp22` contract.
fn transfer(session: &mut Session<Pop>, to: AccountId, amount: Balance) -> Result<(), PSP22Error> {
 let empty_array = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
 let input = vec![to.to_string(), amount.to_string(), empty_array];

 call::<Pop, (), PSP22Error>(session, "Psp22::transfer", input, None)
}
```

Asserts the returned error to an [`Error`](TODO) type using [`assert_err!`](TODO) to test errors of a runtime call.

```rs
use drink::{assert_err, v0::Error, Assets, AssetsError::NoAccount};

#[drink::test(sandbox = Pop)]
fn test_transfer_to_no_account() {
  // Test the contract call if a custom error is the runtime module error.
  assert_err!(
    transfer(&mut session, ALICE, AMOUNT),
    Error::Module(Assets(NoAccount))
  );
}
```

We need to specify the sandbox of Pop Network runtime for a testcase if the contract is using Pop API.

```rs
#[drink::test(sandbox = Pop)]
```

## Development Guide

To run the `examples` contracts for DRink!

```
cargo test --release
```

### Support

- Be part of our passionate community of Web3 pioneers. [Join our Telegram](https://t.me/onpopio)!
- Additionally, there are [GitHub issues](https://github.com/r0gue-io/pop-drink/issues) and
  [Polkadot Stack Exchange](https://polkadot.stackexchange.com/).
