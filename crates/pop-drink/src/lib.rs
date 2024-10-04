pub use drink::*;
pub use frame_support::{self, assert_ok, sp_runtime::DispatchError, traits::PalletInfoAccess};
pub use ink_sandbox::api::assets_api::AssetsAPI;
use ink_sandbox::{AccountIdFor, BalanceFor};
use scale::Decode;
pub use session::{error::SessionError, ContractBundle, Session, NO_SALT};
pub use sp_io::TestExternalities;

pub mod devnet {
	use super::*;
	pub use frame_support::sp_runtime::{
		ArithmeticError, DispatchError, ModuleError, TokenError, TransactionalError,
	};
	pub use pop_runtime_devnet::{
		config::api::versioning::V0Error, Assets, Balances, BuildStorage, Contracts, Runtime,
	};
	use scale::Encode;

	// Types used in the pop runtime.
	pub type Balance = BalanceFor<Runtime>;
	pub type AccountId = AccountIdFor<Runtime>;
	// Error type aliases for the pop runtime modules.
	type AssetsInstance<T> = <T as pallet_api::fungibles::pallet::Config>::AssetsInstance;
	pub type AssetsError = pallet_assets::Error<Runtime, AssetsInstance<Runtime>>;
	pub type BalancesError = pallet_balances::Error<Runtime>;
	pub type ContractsError = pallet_contracts::Error<Runtime>;

	#[derive(Encode, Decode, Debug)]
	pub enum RuntimeError {
		Raw(DispatchError),
		#[codec(index = 52)]
		Assets(AssetsError),
		#[codec(index = 10)]
		Balances(BalancesError),
		#[codec(index = 40)]
		Contracts(ContractsError),
	}

	impl Into<u32> for RuntimeError {
		fn into(self) -> u32 {
			let dispatch_error = match self {
				RuntimeError::Raw(dispatch_error) => dispatch_error,
				RuntimeError::Assets(error) => error.into(),
				RuntimeError::Balances(error) => error.into(),
				RuntimeError::Contracts(error) => error.into(),
			};
			V0Error::from(dispatch_error).into()
		}
	}

	impl From<u32> for RuntimeError {
		fn from(value: u32) -> Self {
			let encoded = value.to_le_bytes();
			match encoded {
				[3, module_error @ ..] => {
					RuntimeError::decode(&mut &module_error[..]).expect("Decoding failed")
				},
				_ => RuntimeError::Raw(
					DispatchError::decode(&mut &encoded[..]).expect("Decoding failed"),
				),
			}
		}
	}

	// This is used to resolve type mismatches between the `AccountId` in the quasi testing
	// environment and the contract environment.
	pub fn account_id_from_slice(s: &AccountId) -> pop_api::primitives::AccountId {
		let account: [u8; 32] = s.clone().into();
		utils::account_id_from_slice(&account)
	}

	#[cfg(test)]
	mod test {
		use crate::devnet::{AssetsError, BalancesError, RuntimeError};
		use frame_support::sp_runtime::ArithmeticError;
		use pop_api::primitives::{ArithmeticError::Overflow, Error as PopApiError};

		#[test]
		fn runtime_error_to_primitives_error_conversion_works() {
			vec![
				(
					RuntimeError::Raw(ArithmeticError::Overflow.into()),
					PopApiError::Arithmetic(Overflow),
				),
				(
					RuntimeError::Assets(AssetsError::BalanceLow),
					PopApiError::Module { index: 52, error: [0, 0] },
				),
				(
					RuntimeError::Assets(AssetsError::NoAccount),
					PopApiError::Module { index: 52, error: [1, 0] },
				),
				(
					RuntimeError::Assets(AssetsError::NoPermission),
					PopApiError::Module { index: 52, error: [2, 0] },
				),
				(
					RuntimeError::Balances(BalancesError::VestingBalance),
					PopApiError::Module { index: 10, error: [0, 0] },
				),
			]
			.into_iter()
			.for_each(|t| {
				let runtime_error: u32 = t.0.into();
				let pop_api_error: u32 = t.1.into();
				// `u32` assertion.
				assert_eq!(runtime_error, pop_api_error);
			});
		}
	}
}

pub mod error {
	use crate::devnet::RuntimeError;

	#[track_caller]
	pub fn assert_runtime_err_inner<R, E: Into<u32>>(
		result: Result<R, E>,
		expected_error: RuntimeError,
	) {
		let expected_code: u32 = expected_error.into();
		if let Err(error) = result {
			let error_code: u32 = error.into();
			if error_code != expected_code {
				panic!(
					r#"assertion `left == right` failed
  left: {:?}
 right: {:?}"#,
					RuntimeError::from(error_code),
					RuntimeError::from(expected_code),
				);
			}
		} else {
			panic!(
				r#"assertion `left == right` failed
  left: Ok()
 right: {:?}"#,
				RuntimeError::from(expected_code),
			);
		}
	}

	#[macro_export]
	macro_rules! assert_runtime_err {
		($result:expr, $error:expr $(,)?) => {
			$crate::error::assert_runtime_err_inner($result, $error);
		};
	}
}

pub mod utils {
	use super::*;
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

	// Call a contract method and decode the returned data.
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
			// If the call is reverted, decode the error into `PSP22Error`.
			Err(SessionError::CallReverted(error)) => {
				Err(E::decode(&mut &error[2..]).unwrap_or_else(|_| panic!("Decoding failed")))
			},
			// If the call is successful, decode the last returned value.
			Ok(_) => Ok(session
				.record()
				.last_call_return_decoded::<O>()
				.unwrap_or_else(|_| panic!("Expected a return value"))
				.unwrap_or_else(|_| panic!("Decoding failed"))),
			// Catch-all for unexpected results.
			_ => panic!("Expected call to revert or return a value"),
		}
	}

	// This is used to resolve type mismatches between the `AccountId` in the quasi testing
	// environment and the contract environment.
	pub fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
		pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
	}

	// Get the last event from pallet contracts.
	pub fn last_contract_event<S>(session: &Session<S>) -> Option<Vec<u8>>
	where
		S: Sandbox,
		S::Runtime: pallet_contracts::Config,
		<S::Runtime as frame_system::Config>::RuntimeEvent:
			TryInto<pallet_contracts::Event<S::Runtime>>,
	{
		session.record().last_event_batch().contract_events().last().cloned()
	}
}
