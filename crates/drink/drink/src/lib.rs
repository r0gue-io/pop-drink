//! The drink crate provides a sandboxed runtime for testing smart contracts without a need for
//! a running node.

pub mod errors;
pub mod pallet_contracts_debugging;
/// Necessary exports in ink_e2e_sandbox
#[cfg(feature = "session")]
pub mod session;

#[cfg(feature = "macros")]
pub use drink_test_macro::{contract_bundle_provider, test};
pub use errors::Error;
pub use frame_support;
pub use ink_sandbox::{
	self, api as sandbox_api, create_sandbox, impl_sandbox, pallet_assets, pallet_balances,
	pallet_contracts, pallet_timestamp, sp_externalities, AccountId32, DispatchError, Sandbox,
	Ss58Codec, Weight,
};
#[cfg(feature = "session")]
pub use session::mock::{mock_message, ContractMock, MessageMock, MockedCallResult, Selector};

/// Main result type for the drink crate.
pub type DrinkResult<T> = std::result::Result<T, Error>;

/// Minimal Sandbox runtime used for testing contracts with drink!.
pub mod minimal {
	use ink_sandbox::create_sandbox;

	// create_sandbox!(MinimalSandbox);
	create_sandbox!(MinimalSandbox, (), crate::pallet_contracts_debugging::DrinkDebug);
}
