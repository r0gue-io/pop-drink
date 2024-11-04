//! A library for testing smart contracts on Pop Network.

pub use drink::*;
pub use frame_support::{self, assert_ok};
pub use ink_sandbox::api::assets_api::AssetsAPI;
use ink_sandbox::{AccountIdFor, BalanceFor};
use scale::Decode;
pub use session::{error::SessionError, ContractBundle, Session, NO_SALT};
pub use sp_io::TestExternalities;

/// Error type and utilities for testing contracts using the Pop API.
pub mod error;
/// Collection of macros for testing contracts using the Pop API.
pub mod macros;
#[cfg(test)]
mod mock;

/// Types and utilities for testing smart contracts interacting with Pop Network Devnet via the pop
/// api.
pub mod devnet {
	pub use pop_runtime_devnet::Runtime;

	use super::*;
	pub use crate::error::*;

	/// Error related utilities for smart contracts using pop api.
	pub mod error {
		pub use pop_runtime_devnet::RuntimeError::*;

		pub use crate::error::*;

		pub mod v0 {
			pub use pop_api::primitives::v0::{self, Error as ApiError, *};

			/// Error type for writing tests (see `error` module).
			pub type Error = crate::error::Error<v0::Error, pop_runtime_devnet::RuntimeError, 3>;
		}
	}

	// Types used in the pop runtime.
	pub type Balance = BalanceFor<Runtime>;
	pub type AccountId = AccountIdFor<Runtime>;

	/// Converts an AccountId from Pop's runtime to the account ID used in the contract environment.
	pub fn account_id_from_slice(s: &AccountId) -> pop_api::primitives::AccountId {
		let account: [u8; 32] = s.clone().into();
		super::account_id_from_slice(&account)
	}
}

/// Deploy a contract with a given constructor, arguments, salt and an initial value. In
/// case of success, returns the address of the deployed contract.
///
/// # Generic Parameters:
/// - `S` - Sandbox environment.
/// - `E` - `Err()` type returned by the contract.
///
/// # Parameters:
/// - `session` - The session for interacting with contracts.
/// - `bundle` - The contract bundle.
/// - `method` - The name of the constructor method.
/// - `input` - The input arguments.
/// - `salt` - Optional deployment salt.
/// - `init_value` - Initial balance to transfer during the contract creation. Requires the contract
///   method to be `payable`.
///
/// # Example:
/// ```rs
/// #[drink::test(sandbox = Pop)]
/// fn test_constructor_works(mut session: Session) {
/// 	let bundle = BundleProvider::local().unwrap();
///
/// 	// Deploy contract.
/// 	//
/// 	// `ContractError` is the error type used by the contract.
/// 	assert_ok!(deploy<Pop, ContractError>(&mut session, bundle, "new", input, salt, init_value));
/// }
/// ```
pub fn deploy<S, E>(
	session: &mut Session<S>,
	bundle: ContractBundle,
	method: &str,
	input: Vec<String>,
	salt: Vec<u8>,
	init_value: Option<BalanceFor<S::Runtime>>,
) -> Result<AccountIdFor<S::Runtime>, E>
where
	S: Sandbox,
	S::Runtime: pallet_contracts::Config,
	E: Decode,
{
	let result = session.deploy_bundle(bundle, method, &input, salt, init_value);
	if result.is_err() {
		let deployment_result = session.record().last_deploy_result().result.clone();
		let error = deployment_result.unwrap().result.data;
		return Err(E::decode(&mut &error[2..]).unwrap());
	}
	Ok(result.unwrap())
}

/// Call a method and decode the returned data.
///
/// # Generic Parameters:
/// - `S` - Sandbox environment.
/// - `O` - `Ok()` type returned by the contract.
/// - `E` - `Err()` type returned by the contract.
///
/// # Parameters:
/// - `session` - The session for interacting with contracts.
/// - `func_name`: The name of the contract method.
/// - `input` - The input arguments.
/// - `init_value` - Balance to transfer during the call. Requires the contract method to be
///   `payable`.
///
/// # Example:
/// ```rs
/// #[drink::test(sandbox = Pop)]
/// fn call_works(mut session: Session) {
/// 	let bundle = BundleProvider::local().unwrap();
/// 	assert_ok!(deploy<Pop, ContractError>(&mut session, bundle, "new", input, salt, init_value));
///
/// 	// Call contract.
/// 	//
/// 	// `()` is the successful result type used by the contract.
/// 	// `ContractError` is the error type used by the contract.
/// 	call::<Pop, (), ContractError>(
/// 		session,
/// 		"transfer",
/// 		input,
/// 		init_value,
/// 	)
/// }
/// ```
pub fn call<S, O, E>(
	session: &mut Session<S>,
	func_name: &str,
	input: Vec<String>,
	endowment: Option<BalanceFor<S::Runtime>>,
) -> Result<O, E>
where
	S: Sandbox,
	S::Runtime: pallet_contracts::Config,
	O: Decode,
	E: Decode,
{
	match session.call::<String, ()>(func_name, &input, endowment) {
		// If the call is reverted, decode the error into the specified error type.
		Err(SessionError::CallReverted(error)) =>
			Err(E::decode(&mut &error[2..]).expect("Decoding failed")),
		// If the call is successful, decode the last returned value.
		Ok(_) => Ok(session
			.record()
			.last_call_return_decoded::<O>()
			.expect("Expected a return value")
			.expect("Decoding failed")),
		// Catch-all for unexpected results.
		_ => panic!("Expected call to revert or return a value"),
	}
}

/// Get the last contract event.
///
/// # Generic Parameters:
/// - `S` - Sandbox environment.
///
/// # Parameters:
/// - `session` - The session for interacting with contracts.
///
/// # Example:
/// ```rs
/// use drink::last_contract_event;
///
/// assert_eq!(
/// 	last_contract_event::<Pop>(&session).unwrap(),
/// 	ContractEvent {
/// 		value: 42,
/// 	}
/// 	.encode()
/// 	.as_slice()
/// );
/// ```
pub fn last_contract_event<S>(session: &Session<S>) -> Option<Vec<u8>>
where
	S: Sandbox,
	S::Runtime: pallet_contracts::Config,
	<S::Runtime as frame_system::Config>::RuntimeEvent:
		TryInto<pallet_contracts::Event<S::Runtime>>,
{
	session.record().last_event_batch().contract_events().last().cloned()
}

fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
	pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
}
