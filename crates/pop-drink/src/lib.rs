pub use drink::*;
pub use frame_support::{self, assert_ok};
pub use ink_sandbox::api::assets_api::AssetsAPI;
use ink_sandbox::{AccountIdFor, BalanceFor};
pub use session::{error::SessionError, ContractBundle, Session, NO_SALT};
pub use sp_io::TestExternalities;

pub mod errors;
#[cfg(test)]
pub mod mock;
pub mod utils;

pub mod devnet {
	pub use pop_runtime_devnet::Runtime;

	use super::*;

	/// A set of primitive runtime errors and versioned runtime errors used for testing.
	pub mod error {
		pub use pop_runtime_devnet::RuntimeError::{self as PopRuntimeError, *};

		pub use crate::errors::*;
		/// A collection of error types from the `v0` module used for smart contract testing in the
		/// `devnet` environment.
		pub mod v0 {
			use super::*;
			pub use crate::errors::v0::*;
			/// Runtime error type used for testing smart contract methods using Pop API `v0`.
			pub type RuntimeError = crate::errors::v0::RuntimeError<PopRuntimeError>;
		}
	}

	// Types used in the pop runtime.
	pub type Balance = BalanceFor<Runtime>;
	pub type AccountId = AccountIdFor<Runtime>;

	// This is used to resolve type mismatches between the `AccountId` in the quasi testing
	// environment and the contract environment.
	pub fn account_id_from_slice(s: &AccountId) -> pop_api::primitives::AccountId {
		let account: [u8; 32] = s.clone().into();
		utils::account_id_from_slice(&account)
	}
}
