use std::time::SystemTime;

use frame_support::{
	sp_runtime::{
		traits::{Header, One},
		BuildStorage,
	},
	traits::Hooks,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_io::TestExternalities;

/// A helper struct for initializing and finalizing blocks.
pub struct BlockBuilder<T>(std::marker::PhantomData<T>);

impl<
		T: pallet_balances::Config + pallet_timestamp::Config<Moment = u64> + pallet_contracts::Config,
	> BlockBuilder<T>
{
	/// Create a new externalities with the given balances.
	pub fn new_ext(balances: Vec<(T::AccountId, T::Balance)>) -> TestExternalities {
		let mut storage = frame_system::GenesisConfig::<T>::default().build_storage().unwrap();

		pallet_balances::GenesisConfig::<T> { balances, dev_accounts: None }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext = TestExternalities::new(storage);

		ext.execute_with(|| Self::initialize_block(BlockNumberFor::<T>::one(), Default::default()));
		ext
	}

	/// Initialize a new block at particular height.
	pub fn initialize_block(
		height: frame_system::pallet_prelude::BlockNumberFor<T>,
		parent_hash: <T as frame_system::Config>::Hash,
	) {
		frame_system::Pallet::<T>::reset_events();
		frame_system::Pallet::<T>::initialize(&height, &parent_hash, &Default::default());
		pallet_balances::Pallet::<T>::on_initialize(height);
		pallet_timestamp::Pallet::<T>::set_timestamp(
			SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.expect("Time went backwards")
				.as_secs(),
		);
		pallet_timestamp::Pallet::<T>::on_initialize(height);
		pallet_contracts::Pallet::<T>::on_initialize(height);
		frame_system::Pallet::<T>::note_finished_initialize();
	}

	/// Finalize a block at particular height.
	pub fn finalize_block(
		height: frame_system::pallet_prelude::BlockNumberFor<T>,
	) -> <T as frame_system::Config>::Hash {
		pallet_contracts::Pallet::<T>::on_finalize(height);
		pallet_timestamp::Pallet::<T>::on_finalize(height);
		pallet_balances::Pallet::<T>::on_finalize(height);
		frame_system::Pallet::<T>::finalize().hash()
	}
}

// Macro that implements the sandbox trait on the provided runtime.
#[macro_export]
macro_rules! impl_sandbox {
    ($sandbox:ident, $runtime:ident, $account:ident) => {
        use $crate::macros::BlockBuilder;

        impl $crate::Sandbox for $sandbox {
            type Runtime = $runtime;

            fn execute_with<T>(&mut self, execute: impl FnOnce() -> T) -> T {
                self.ext.execute_with(execute)
            }

            fn dry_run<T>(&mut self, action: impl FnOnce(&mut Self) -> T) -> T {
                // Make a backup of the backend.
                let backend_backup = self.ext.as_backend();
                // Run the action, potentially modifying storage. Ensure, that there are no pending changes
                // that would affect the reverted backend.
                let result = action(self);
                self.ext.commit_all().expect("Failed to commit changes");

                // Restore the backend.
                self.ext.backend = backend_backup;
                result
            }

            fn register_extension<E: ::core::any::Any + $crate::Extension>(&mut self, ext: E) {
                self.ext.register_extension(ext);
            }

            fn initialize_block(
                height: $crate::frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
                parent_hash: <Self::Runtime as $crate::frame_system::Config>::Hash,
            ) {
                BlockBuilder::<Self::Runtime>::initialize_block(height, parent_hash)
            }

            fn finalize_block(
                height: $crate::frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
            ) -> <Self::Runtime as $crate::frame_system::Config>::Hash {
                BlockBuilder::<Self::Runtime>::finalize_block(height)
            }

            fn default_actor() -> $crate::AccountIdFor<Self::Runtime> {
                $account
            }

            fn get_metadata() -> $crate::RuntimeMetadataPrefixed {
                Self::Runtime::metadata()
            }

            fn convert_account_to_origin(
                account: $crate::AccountIdFor<Self::Runtime>,
            ) -> <<Self::Runtime as $crate::frame_system::Config>::RuntimeCall as $crate::frame_support::sp_runtime::traits::Dispatchable>::RuntimeOrigin {
                Some(account).into()
            }
        }
    };
}

/// Macro creating a minimal runtime with the given name. Optionally can take a chain
/// extension type as a second argument.
///
/// The new macro will automatically implement `crate::Sandbox`.
#[macro_export]
macro_rules! create_sandbox {
    ($name:ident) => {
        $crate::paste::paste! {
            $crate::create_sandbox!($name, [<$name Runtime>], (), (), {});
        }
    };
    ($name:ident, $chain_extension: ty, $debug: ty) => {
        $crate::paste::paste! {
            $crate::create_sandbox!($name, [<$name Runtime>], $chain_extension, $debug, {});
        }
    };
    ($name:ident, $chain_extension: ty, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {
        $crate::paste::paste! {
            $crate::create_sandbox!($name, [<$name Runtime>], $chain_extension, $debug, {
                $(
                    $pallet_name : $pallet,
                )*
            });
        }
    };
    ($sandbox:ident, $runtime:ident, $chain_extension: ty, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {


// Put all the boilerplate into an auxiliary module
mod construct_runtime {

    // Bring some common types into the scope
    use $crate::frame_support::{
        construct_runtime,
        derive_impl,
        parameter_types,
        sp_runtime::{
            testing::H256,
            traits::Convert,
            AccountId32, Perbill,
        },
        traits::{AsEnsureOriginWithArg, ConstBool, ConstU128, ConstU32, ConstU64, Currency, Randomness},
        weights::Weight,
    };
    use $crate::frame_system::EnsureSigned;
    use $crate::pallet_assets::Instance1;

    // Define the runtime type as a collection of pallets
    construct_runtime!(
        pub enum $runtime {
            System: $crate::frame_system,
            Assets: $crate::pallet_assets::<Instance1>,
            Balances: $crate::pallet_balances,
            Timestamp: $crate::pallet_timestamp,
            Contracts: $crate::pallet_contracts,
            $(
                $pallet_name: $pallet,
            )*
        }
    );

    // Configure pallet system
    #[derive_impl($crate::frame_system::config_preludes::SolochainDefaultConfig as $crate::frame_system::DefaultConfig)]
    impl $crate::frame_system::Config for $runtime {
        type Block = $crate::frame_system::mocking::MockBlockU32<$runtime>;
        type Version = ();
        type BlockHashCount = ConstU32<250>;
        type AccountData = $crate::pallet_balances::AccountData<<$runtime as $crate::pallet_balances::Config>::Balance>;
    }

    // Configure pallet assets
    impl $crate::pallet_assets::Config<Instance1> for $runtime {
        type ApprovalDeposit = ConstU128<1>;
        type AssetAccountDeposit = ConstU128<10>;
        type AssetDeposit = ConstU128<1>;
        type AssetId = u32;
        type AssetIdParameter = u32;
        type Balance = u128;
        type CallbackHandle = ();
        type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<Self::AccountId>>;
        type Currency = Balances;
        type Extra = ();
        type ForceOrigin = EnsureSigned<Self::AccountId>;
        type Freezer = ();
        type Holder = ();
        type MetadataDepositBase = ConstU128<1>;
        type MetadataDepositPerByte = ConstU128<1>;
        type RemoveItemsLimit = ConstU32<5>;
        type RuntimeEvent = RuntimeEvent;
        type StringLimit = ConstU32<50>;
        type WeightInfo = ();
    }

    // Configure pallet balances
    impl $crate::pallet_balances::Config for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = ();
        type Balance = u128;
        type DoneSlashHandler = ();
        type DustRemoval = ();
        type ExistentialDeposit = ConstU128<1>;
        type AccountStore = System;
        type ReserveIdentifier = [u8; 8];
        type FreezeIdentifier = ();
        type MaxLocks = ();
        type MaxReserves = ();
        type MaxFreezes = ();
        type RuntimeHoldReason = RuntimeHoldReason;
        type RuntimeFreezeReason = RuntimeFreezeReason;
    }

    // Configure pallet timestamp
    impl $crate::pallet_timestamp::Config for $runtime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ConstU64<1>;
        type WeightInfo = ();
    }

    pub enum SandboxRandomness {}
    impl Randomness<H256, u32> for SandboxRandomness {
        fn random(_subject: &[u8]) -> (H256, u32) {
            unreachable!("No randomness")
        }
    }

    type BalanceOf = <Balances as Currency<AccountId32>>::Balance;
    impl Convert<Weight, BalanceOf> for $runtime {
        fn convert(w: Weight) -> BalanceOf {
            w.ref_time().into()
        }
    }

    parameter_types! {
        pub SandboxSchedule: $crate::pallet_contracts::Schedule<$runtime> = {
            <$crate::pallet_contracts::Schedule<$runtime>>::default()
        };
        pub DeletionWeightLimit: Weight = Weight::zero();
        pub DefaultDepositLimit: BalanceOf = 10_000_000;
        pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
        pub MaxDelegateDependencies: u32 = 32;
    }

    // Configure pallet contracts
    impl $crate::pallet_contracts::Config for $runtime {
        type Time = Timestamp;
        type Randomness = SandboxRandomness;
        type Currency = Balances;
        type RuntimeEvent = RuntimeEvent;
        type RuntimeCall = RuntimeCall;
        type CallFilter = ();
        type WeightPrice = Self;
        type WeightInfo = ();
        type ChainExtension = $chain_extension;
        type Schedule = SandboxSchedule;
        type CallStack = [$crate::pallet_contracts::Frame<Self>; 5];
        type DepositPerByte = ConstU128<1>;
        type DepositPerItem = ConstU128<1>;
        type AddressGenerator = $crate::pallet_contracts::DefaultAddressGenerator;
        type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
        type MaxStorageKeyLen = ConstU32<128>;
        type MaxTransientStorageSize = ConstU32<{ 1024 * 1024 }>;
        type UnsafeUnstableInterface = ConstBool<false>;
        type UploadOrigin = $crate::frame_system::EnsureSigned<Self::AccountId>;
        type InstantiateOrigin = $crate::frame_system::EnsureSigned<Self::AccountId>;
        type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
        type Migrations = ();
        type DefaultDepositLimit = DefaultDepositLimit;
        type Debug = $debug;
        type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
        type MaxDelegateDependencies = MaxDelegateDependencies;
        type RuntimeHoldReason = RuntimeHoldReason;
        type Environment = ();
        type Xcm = ();
        type ApiVersion = ();
    }

    /// Unit base for balances.
    pub const UNIT: u128 = 10_000_000_000;
    /// Default initial balance for the default account.
    pub const INIT_AMOUNT: u128 = 100_000_000 * UNIT;
    /// Default account.
    pub const DEFAULT_ACCOUNT: AccountId32 = AccountId32::new([1u8; 32]);

    /// The sandbox.
    pub struct $sandbox {
        ext: $crate::TestExternalities,
    }

    impl ::std::default::Default for $sandbox {
        fn default() -> Self {
            let ext = BlockBuilder::<$runtime>::new_ext(vec![(DEFAULT_ACCOUNT, INIT_AMOUNT)]);
            Self { ext }
        }
    }

    // Implement `Sandbox` trait.
    $crate::impl_sandbox!($sandbox, $runtime, DEFAULT_ACCOUNT);

}

// Export runtime type itself, pallets and useful types from the auxiliary module
pub use construct_runtime::{
    $sandbox, $runtime, Assets, Balances, Contracts, PalletInfo, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
    RuntimeOrigin, System, Timestamp,
};
    };
}

create_sandbox!(DefaultSandbox);
