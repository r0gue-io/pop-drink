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
        let mut storage = frame_system::GenesisConfig::<T>::default()
            .build_storage()
            .unwrap();

        pallet_balances::GenesisConfig::<T> { balances }
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

#[macro_export]
macro_rules! create_sandbox {
    ($sandbox:ident, $runtime:ident) => {
        use $crate::frame_support::sp_runtime::AccountId32;

        // Implement `crate::Sandbox` trait

        /// Default initial balance for the default account.
        pub const UNIT: u128 = 10_000_000_000;
        pub const INIT_AMOUNT: u128 = 100_000_000 * UNIT;
        pub const DEFAULT_ACCOUNT: AccountId32 = AccountId32::new([1u8; 32]);

        pub struct $sandbox {
            ext: $crate::TestExternalities,
        }

        impl ::std::default::Default for $sandbox {
            fn default() -> Self {
                let ext = $crate::macros::BlockBuilder::<$runtime>::new_ext(vec![(
                    DEFAULT_ACCOUNT,
                    INIT_AMOUNT,
                )]);
                Self { ext }
            }
        }

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
                $crate::macros::BlockBuilder::<Self::Runtime>::initialize_block(height, parent_hash)
            }

            fn finalize_block(
                height: $crate::frame_system::pallet_prelude::BlockNumberFor<Self::Runtime>,
            ) -> <Self::Runtime as $crate::frame_system::Config>::Hash {
                $crate::macros::BlockBuilder::<Self::Runtime>::finalize_block(height)
            }

            fn default_actor() -> $crate::AccountIdFor<Self::Runtime> {
                DEFAULT_ACCOUNT
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
macro_rules! create_sandbox_with_runtime {
    ($name:ident) => {
        $crate::paste::paste! {
            $crate::create_sandbox_with_runtime!($crate, $name, [<$name Runtime>], (), (), {});
        }
    };
    ($name:ident, $chain_extension: ty, $debug: ty) => {
        $crate::paste::paste! {
            $crate::create_sandbox_with_runtime!($crate, $name, [<$name Runtime>], $chain_extension, $debug, {});
        }
    };
    ($name:ident, $chain_extension: ty, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {
        $crate::paste::paste! {
            $crate::create_sandbox_with_runtime!($crate, $name, [<$name Runtime>], $chain_extension, $debug, {
                $(
                    $pallet_name : $pallet,
                )*
            });
        }
    };
    ($module_path:ident, $name:ident, $chain_extension: ty, $debug: ty) => {
        $crate::paste::paste! {
            $crate::create_sandbox_with_runtime!($module_path, $name, [<$name Runtime>], $chain_extension, $debug, {});
        }
    };
    ($module_path:ident, $sandbox:ident, $runtime:ident, $chain_extension: ty, $debug: ty, { $( $pallet_name:tt : $pallet:ident ),* $(,)? }) => {


// Put all the boilerplate into an auxiliary module
mod construct_runtime {

    // Bring some common types into the scope
    use $module_path::frame_support::{
        construct_runtime,
        derive_impl,
        parameter_types,
        sp_runtime::{
            testing::H256,
            traits::Convert,
             Perbill,
        },
        traits::{ConstBool, ConstU128, ConstU32, ConstU64, Currency, Randomness},
        weights::Weight,
    };
    use $module_path::frame_system::EnsureSigned;

    // Define the runtime type as a collection of pallets
    construct_runtime!(
        pub enum $runtime {
            System: $module_path::frame_system,
            Balances: $module_path::pallet_balances,
            Timestamp: $module_path::pallet_timestamp,
            Contracts: $module_path::pallet_contracts,
            $(
                $pallet_name: $pallet,
            )*
        }
    );

    // Configure pallet system
    #[derive_impl($module_path::frame_system::config_preludes::SolochainDefaultConfig as $module_path::frame_system::DefaultConfig)]
    impl $module_path::frame_system::Config for $runtime {
        type Block = $module_path::frame_system::mocking::MockBlockU32<$runtime>;
        type Version = ();
        type BlockHashCount = ConstU32<250>;
        type AccountData = $module_path::pallet_balances::AccountData<<$runtime as $module_path::pallet_balances::Config>::Balance>;
    }

    // Configure pallet balances
    impl $module_path::pallet_balances::Config for $runtime {
        type RuntimeEvent = RuntimeEvent;
        type WeightInfo = ();
        type Balance = u128;
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
    impl $module_path::pallet_timestamp::Config for $runtime {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = ConstU64<1>;
        type WeightInfo = ();
    }

    // Configure pallet contracts
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
        pub SandboxSchedule: $module_path::pallet_contracts::Schedule<$runtime> = {
            <$module_path::pallet_contracts::Schedule<$runtime>>::default()
        };
        pub DeletionWeightLimit: Weight = Weight::zero();
        pub DefaultDepositLimit: BalanceOf = 10_000_000;
        pub CodeHashLockupDepositPercent: Perbill = Perbill::from_percent(0);
        pub MaxDelegateDependencies: u32 = 32;
    }

    impl $module_path::pallet_contracts::Config for $runtime {
        type AddressGenerator = $module_path::pallet_contracts::DefaultAddressGenerator;
        type ApiVersion = ();
        type CallFilter = ();
        // TestFilter;
        type CallStack = [$module_path::pallet_contracts::Frame<Self>; 5];
        type ChainExtension = $chain_extension;
        type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
        type Currency = Balances;
        type Debug = $debug;
        // TestDebug;
        type DefaultDepositLimit = DefaultDepositLimit;
        type DepositPerByte = ConstU128<1>;
        type DepositPerItem = ConstU128<1>;
        type Environment = ();
        type InstantiateOrigin = EnsureSigned<Self::AccountId>;
        type MaxCodeLen = ConstU32<{ 123 * 1024 }>;
        type MaxDebugBufferLen = ConstU32<{ 2 * 1024 * 1024 }>;
        type MaxDelegateDependencies = MaxDelegateDependencies;
        type MaxStorageKeyLen = ConstU32<128>;
        type Migrations = ();
        // crate::migration::codegen::BenchMigrations;
        type Randomness = SandboxRandomness;
        type RuntimeCall = RuntimeCall;
        type RuntimeEvent = RuntimeEvent;
        type RuntimeHoldReason = RuntimeHoldReason;
        type Schedule = SandboxSchedule;
        type Time = Timestamp;
        type UnsafeUnstableInterface = ConstBool<false>;
        // UnstableInterface;
        type UploadOrigin = EnsureSigned<Self::AccountId>;
        type WeightInfo = ();
        type WeightPrice = ();
        // Self;
        type Xcm = ();
    }

    $crate::create_sandbox!($sandbox, $runtime);
}

// Export runtime type itself, pallets and useful types from the auxiliary module
pub use construct_runtime::{
    $sandbox, $runtime, Balances, Contracts, PalletInfo, RuntimeCall, RuntimeEvent, RuntimeHoldReason,
    RuntimeOrigin, System, Timestamp,
};
    };
}

create_sandbox_with_runtime!(DefaultSandbox);
