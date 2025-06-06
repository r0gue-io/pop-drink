use core::any::Any;

pub mod api;
pub mod macros;

pub use frame_metadata::RuntimeMetadataPrefixed;
pub use frame_support::weights::Weight;
use frame_support::{
	sp_runtime::traits::{Dispatchable, StaticLookup},
	traits::fungible::Inspect,
};
use frame_system::{pallet_prelude::BlockNumberFor, EventRecord};
pub use macros::{BlockBuilder, DefaultSandbox};
use pallet_contracts::{ContractExecResult, ContractInstantiateResult};
/// Export pallets that are used in [`crate::create_sandbox`]
pub use {
	frame_support::{
		self,
		sp_runtime::{AccountId32, DispatchError},
	},
	frame_system, pallet_assets, pallet_balances, pallet_contracts, pallet_nfts, pallet_timestamp,
	paste,
	sp_core::crypto::Ss58Codec,
	sp_externalities::{self, Extension},
	sp_io::TestExternalities,
	sp_runtime_interface::{self},
};

/// Alias for the account ID lookup source.
pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

/// Alias for the account ID type.
pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;

/// Alias for the balance type.
pub type BalanceFor<R> =
	<<R as pallet_contracts::Config>::Currency as Inspect<AccountIdFor<R>>>::Balance;

/// Alias for the contract exec result.
pub type ContractExecResultFor<Runtime> =
	ContractExecResult<BalanceFor<Runtime>, EventRecordOf<Runtime>>;

/// Alias for the contract instantiate result.
pub type ContractInstantiateResultFor<Runtime> =
	ContractInstantiateResult<AccountIdFor<Runtime>, BalanceFor<Runtime>, EventRecordOf<Runtime>>;

/// Alias for the event record type.
pub type EventRecordOf<Runtime> = EventRecord<
	<Runtime as frame_system::Config>::RuntimeEvent,
	<Runtime as frame_system::Config>::Hash,
>;

/// Alias for the runtime origin.
type OriginFor<T> = <RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin;

/// Alias for the runtime call type.
pub type RuntimeCall<R> = <R as frame_system::Config>::RuntimeCall;

/// Alias for the runtime event of a sandbox.
pub type RuntimeEventOf<S> = <RuntimeOf<S> as frame_system::Config>::RuntimeEvent;

/// Alias for the runtime of a sandbox.
pub type RuntimeOf<S> = <S as Sandbox>::Runtime;

/// Sandbox defines the API of a sandboxed runtime.
pub trait Sandbox {
	/// The runtime associated with the sandbox.
	type Runtime: frame_system::Config;

	/// Execute the given externalities.
	fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T;

	/// Dry run an action without modifying the storage.
	fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T;

	/// Register an extension.
	fn register_extension<E: Any + Extension>(&mut self, ext: E);

	/// Initialize a new block at particular height.
	fn initialize_block(
		_height: BlockNumberFor<Self::Runtime>,
		_parent_hash: <Self::Runtime as frame_system::Config>::Hash,
	) {
	}

	/// Finalize a block at particular height.
	fn finalize_block(
		_height: BlockNumberFor<Self::Runtime>,
	) -> <Self::Runtime as frame_system::Config>::Hash {
		Default::default()
	}

	/// Default actor for the sandbox.
	fn default_actor() -> AccountIdFor<Self::Runtime>;

	fn default_gas_limit() -> Weight {
		Weight::from_parts(100_000_000_000, 3 * 1024 * 1024)
	}

	/// Metadata of the runtime.
	fn get_metadata() -> RuntimeMetadataPrefixed;

	/// Convert an account to an call origin.
	fn convert_account_to_origin(
		account: AccountIdFor<Self::Runtime>,
	) -> <<Self::Runtime as frame_system::Config>::RuntimeCall as Dispatchable>::RuntimeOrigin;
}
