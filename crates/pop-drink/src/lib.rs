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

	// Types used in the pop runtime.
	pub type Balance = BalanceFor<Runtime>;
	pub type AccountId = AccountIdFor<Runtime>;
	// Error type aliases for the pop runtime modules.
	type AssetsInstance<T> = <T as pallet_api::fungibles::pallet::Config>::AssetsInstance;
	pub type AssetsError = pallet_assets::Error<Runtime, AssetsInstance<Runtime>>;
	pub type BalancesError = pallet_balances::Error<Runtime>;
	pub type ContractsError = pallet_contracts::Error<Runtime>;

	// This is used to resolve type mismatches between the `AccountId` in the quasi testing
	// environment and the contract environment.
	pub fn account_id_from_slice(s: &AccountId) -> pop_api::primitives::AccountId {
		let account: [u8; 32] = s.clone().into();
		utils::account_id_from_slice(&account)
	}

	#[derive(Debug, PartialEq)]
	pub struct RuntimeError(pub DispatchError);

	impl Into<u32> for RuntimeError {
		fn into(self) -> u32 {
			V0Error::from(self.0).into()
		}
	}

	#[cfg(test)]
	mod test {
		use crate::devnet::{AssetsError, BalancesError, RuntimeError};
		use frame_support::sp_runtime::ArithmeticError;
		use pop_api::primitives::{ArithmeticError::Overflow, Error as PopApiError};

		#[test]
		fn runtime_error_to_primitives_error_conversion_works() {
			vec![
				(RuntimeError(ArithmeticError::Overflow.into()), PopApiError::Arithmetic(Overflow)),
				(
					RuntimeError(AssetsError::BalanceLow.into()),
					PopApiError::Module { index: 52, error: [0, 0] },
				),
				(
					RuntimeError(AssetsError::NoAccount.into()),
					PopApiError::Module { index: 52, error: [1, 0] },
				),
				(
					RuntimeError(AssetsError::NoPermission.into()),
					PopApiError::Module { index: 52, error: [2, 0] },
				),
				(
					RuntimeError(BalancesError::VestingBalance.into()),
					PopApiError::Module { index: 10, error: [0, 0] },
				),
			]
			.into_iter()
			.for_each(|t| {
				println!("t");
				let runtime_error: u32 = t.0.into();
				let pop_api_error: u32 = t.1.into();
				// `u32` assertion.
				assert_eq!(runtime_error, pop_api_error);
			});
		}
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
