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

- `crates/drink`
  - `drink`: DRink! is a toolbox for ink! developers that allows for a fully functional ink! contract development without any running node.
  - `drink-cli`: Simple command line tool to help you play with your local contracts in a convenient way.
  - `examples`: A collection of example contracts tested with DRink!
- `crates/ink-sandbox`: In the context of quasi-testing with pop-drink, a sandbox refers to an isolated runtime environment that simulates the behavior of a full node, without requiring an actual node.
- `crates/pop-drink`: Provides utility methods for testing contract methods with DRink! and Pop Network runtimes.

## Getting Started

- Add `pop-drink` crate to your contract `Cargo.toml`:

```toml
drink = { version = "1.0.0",  package = "pop-drink" }
```

### 1. Setup a testing environment

- Import required methods and types:

```rs
use drink::{
 devnet::{AccounId, Balance},
 TestExternalities
};
```

- Add the below code at the top of your contract test file to setup sandbox and contract bundle provider for the **Pop Network Devnet** runtime:

```rs
// Initialising useful variables for testing the contract.
const UNIT: Balance = 10_000_000_000;
const INIT_AMOUNT: Balance = 100_000_000 * UNIT;
const INIT_VALUE: Balance = 100 * UNIT;
const ALICE: AccountId = AccountId::new([1u8; 32]);
const BOB: AccountId = AccountId::new([2_u8; 32]);
const CHARLIE: AccountId = AccountId::new([3_u8; 32]);

// The contract bundle provider.
//
// Reference: https://github.com/r0gue-io/pop-drink/blob/main/crates/drink/drink/test-macro/src/lib.rs
#[drink::contract_bundle_provider]
enum BundleProvider {}

// Sandbox environment for Pop Devnet Runtime.
struct Pop {
	ext: TestExternalities,
}

impl Default for Pop {
	fn default() -> Self {
		// Initialising genesis state, providing accounts with an initial balance.
		let balances: Vec<(AccountId, u128)> =
			vec![(ALICE, INIT_AMOUNT), (BOB, INIT_AMOUNT), (CHARLIE, INIT_AMOUNT)];
		let ext = BlockBuilder::<Runtime>::new_ext(balances);
		Self { ext }
	}
}

// Implement `Sandbox` environment for the Pop Network Devnet runtime.
drink::impl_sandbox!(Pop, Runtime, ALICE);
```

### 2. Deploy a new contract

- Import required methods and types:

```rs
use drink::{assert_ok, deploy};
```

- Instantiate and deploy a local contract with `new` constructor method:

```rs
let local_contract = BundleProvider::local().unwrap();
// Contract address is returned if a deployment succeeds.
assert_ok!(deploy(
 &mut session,
 local_contract,
 "new",
 vec![TOKEN.to_string(), MIN_BALANCE.to_string()],
 vec![],
 None
));
```

### 3. Test contract method that uses [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api)

This example interacts with a [PSP22](https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md) example contract that uses [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api). The contract method returns [`PSP22Error`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/v0/fungibles/errors.rs#L73C1-L73C22) which is provided by Pop API library.

Learn more in the [PSP22 example contract](https://github.com/r0gue-io/pop-node/blob/main/pop-api/examples/fungibles/lib.rs).

- Import required methods and types:

```rs
use drink::{call, session::Session};
use pop_api::v0::fungibles::PSP22Error;
```

- Example of interacting with the `transfer` method of the `Psp22` contract:

```rs
// Example of a method to interact with contract via pop-drink.
fn transfer(session: &mut Session<Pop>, to: AccountId, amount: Balance) -> Result<(), PSP22Error> {
 // Initialising a string value of an empty array.
 let empty_array = serde_json::to_string::<[u8; 0]>(&[]).unwrap();
 // Call to a contract method.
 call::<Pop, (), PSP22Error>(
  session,
  "Psp22::transfer",
  vec![to.to_string(), amount.to_string(), empty_array],
  None,
 )
}
```

#### If the contract throws a non-custom error:

Non-custom errors are fixed variants of the provided [`PSP22Error`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/v0/fungibles/errors.rs#L73C1-L73C22).

```rs
// Not enough balance. Failed with `InsufficientBalance`.
assert_eq!(
 transfer_from(&mut session, ALICE, contract.clone(), AMOUNT + 1),
 Err(PSP22Error::InsufficientBalance)
);
```

#### If the contract throws a custom error:

A custom error is returned if the error doesn't conform to the [`PSP22Error`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/v0/fungibles/errors.rs#L73C1-L73C22) standard.
The custom error is represented by a [`StatusCode`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/lib.rs#L33), which encapsulates a `u32` value indicating the success or failure of a runtime call via the Pop API.

Pop DRink! provides an error type and a [macro](https://doc.rust-lang.org/book/ch19-06-macros.html) to simplify testing both runtime module errors and API errors.

- `drink::v0::Error`: Runtime error type for testing API V0.
- `assert_err`: Asserts that a `Result` with an error type convertible to `u32` matches the expected `Error` from pop-drink.

Test the contract call if a custom error is the runtime module error:

```rs
use drink::{assert_err, v0::Error, Assets, AssetsError::NoAccount};

assert_err!(
 transfer(&mut session, ALICE, AMOUNT),
 Error::Module(Assets(NoAccount))
);
```

Test the contract call if a custom error is the API error:

```rs
use drink::{assert_err, v0::{Error, Arithmetic, ArithmeticError::Overflow}};

assert_err!(
 transfer(&mut session, ALICE, AMOUNT),
 Error::Api(Arithmetic(Overflow))
);
```

## Development Guide

To run the `examples` contracts for DRink!

```
cargo test --release
```

## Terminology

- `crates/drink`

  - **[Session](https://github.com/r0gue-io/pop-drink/blob/main/crates/drink/drink/src/session.rs)**: Wrapper around [`Sandbox`](https://github.com/r0gue-io/pop-drink/blob/main/crates/ink-sandbox/src/lib.rs) that provides methods to interact with multiple contracts.

  - **[Sandbox](https://github.com/r0gue-io/pop-drink/blob/main/crates/ink-sandbox/src/lib.rs)**: In the context of quasi-testing with pop-drink, a sandbox refers to an isolated runtime environment that simulates the behavior of a full node, without requiring an actual node.

- `crates/pop-drink`

  - Mentions of `API` in `crates/pop_drink` refer to `pop_api::Error`.

  - **Module Errors**: Errors returned by the Substrate runtime modules (or pallets).

  - **API Errors**: Errors returned by the Substrate runtime modules (or pallets).

### Support

- Be part of our passionate community of Web3 pioneers. [Join our Telegram](https://t.me/onpopio)!
- Additionally, there are [GitHub issues](https://github.com/r0gue-io/pop-drink/issues) and
  [Polkadot Stack Exchange](https://polkadot.stackexchange.com/).
