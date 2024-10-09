pub use drink::*;
pub use frame_support::{self, assert_ok};
pub use ink_sandbox::api::assets_api::AssetsAPI;
use ink_sandbox::{AccountIdFor, BalanceFor};
use scale::Decode;
pub use session::{error::SessionError, ContractBundle, Session, NO_SALT};
pub use sp_io::TestExternalities;

pub mod error;
#[cfg(test)]
pub mod mock;

pub mod devnet {
	pub use pop_runtime_devnet::Runtime;

	use super::*;

	/// A set of primitive runtime errors and versioned runtime errors used for testing.
	pub mod error {
		pub use pop_runtime_devnet::RuntimeError::*;

		pub use crate::error::*;

		/// A collection of error types from the `v0` module used for smart contract testing in the
		/// `devnet` environment.
		pub mod v0 {
			pub use pop_api::primitives::v0::{Error as ApiError, *};

			/// Error type for testing contracts using the API V0.
			pub type Error = crate::error::Error<pop_runtime_devnet::RuntimeError, ApiError, 3>;
		}
	}

	// Types used in the pop runtime.
	pub type Balance = BalanceFor<Runtime>;
	pub type AccountId = AccountIdFor<Runtime>;

	/// This is used to resolve type mismatches between the `AccountId` in the quasi testing
	/// environment and the contract environment.
	pub fn account_id_from_slice(s: &AccountId) -> pop_api::primitives::AccountId {
		let account: [u8; 32] = s.clone().into();
		super::account_id_from_slice(&account)
	}
}

/// Deploys a contract with a given constructo methodr, arguments, salt and initial value. In case
/// of success, returns the address of the deployed contract.
///
/// # Generic Parameters:
///
/// - `S`: Sandbox defines the API of a sandboxed runtime.
/// - `E`: Decodable error type returned if the contract deployment fails.
///
/// # Parameters:
///
/// - `session` - Wrapper around `Sandbox`` that provides a convenient API for interacting with
///   multiple contracts.
/// - `bundle` - A struct representing the result of parsing a `.contract` bundle file.
/// - `method` - The name of the constructor method.
/// - `input` - Arguments passed to the constructor method.
/// - `salt` - Additional data used to influence the contract deployment address.
/// - `init_value` - Initial funds allocated for the contract.
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

/// Call a contract method and decode the returned data.
///
/// # Generic Parameters:
///
/// - `S` - Sandbox defines the API of a sandboxed runtime.
/// - `O` - Decodable output type returned if the contract deployment succeeds.
/// - `E` - Decodable error type returned if the contract deployment fails.
///
/// # Parameters:
///
/// - `session` - Wrapper around `Sandbox`` that provides a convenient API for interacting with
///   multiple contracts.
/// - `func_name` - The name of the contract method.
/// - `input` - Arguments passed to the contract method.
/// - `endowment` - Funds allocated for the contract execution.
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
		Err(SessionError::CallReverted(error)) => {
			Err(E::decode(&mut &error[2..]).expect("Decoding failed"))
		},
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

fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
	pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
}

/// Get the last event from pallet contracts.
///
/// # Generic Parameters:
///
/// - `S` - Sandbox defines the API of a sandboxed runtime.
///
/// # Parameters:
///
/// - `session` - Wrapper around `Sandbox`` that provides a convenient API for interacting with
///   multiple contracts.
pub fn last_contract_event<S>(session: &Session<S>) -> Option<Vec<u8>>
where
	S: Sandbox,
	S::Runtime: pallet_contracts::Config,
	<S::Runtime as frame_system::Config>::RuntimeEvent:
		TryInto<pallet_contracts::Event<S::Runtime>>,
{
	session.record().last_event_batch().contract_events().last().cloned()
}
