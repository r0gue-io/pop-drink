pub use drink::*;
pub use ink_sandbox::api::{assets_api::AssetsAPI};
pub use sp_io::TestExternalities;
pub use frame_support::{self, sp_runtime::DispatchError, assert_ok};
pub mod devnet {
    pub use pop_runtime_devnet::{BuildStorage, Runtime};
    use ink_sandbox::{AccountIdFor, BalanceFor};

    /// Balance type used in the pop runtime.
    pub type Balance = BalanceFor<Runtime>;
    pub type AccountId = AccountIdFor<Runtime>;
}

