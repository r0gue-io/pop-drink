pub use drink::*;
pub use frame_support::{self, assert_ok};
pub use ink_sandbox::api::assets_api::AssetsAPI;
use ink_sandbox::{AccountIdFor, BalanceFor};
pub use session::{error::SessionError, ContractBundle, Session, NO_SALT};
pub use sp_io::TestExternalities;

pub mod error;
#[cfg(test)]
pub mod mock;
pub mod utils;

pub mod devnet {
	use error::DrinkError as DrinkErrorOf;
	pub use pop_runtime_devnet::{Runtime, RuntimeError};

	use super::*;

	// Types used in the pop runtime.
	pub type Balance = BalanceFor<Runtime>;
	pub type AccountId = AccountIdFor<Runtime>;
	pub type DrinkError = DrinkErrorOf<RuntimeError>;

	// This is used to resolve type mismatches between the `AccountId` in the quasi testing
	// environment and the contract environment.
	pub fn account_id_from_slice(s: &AccountId) -> pop_api::primitives::AccountId {
		let account: [u8; 32] = s.clone().into();
		utils::account_id_from_slice(&account)
	}
}
