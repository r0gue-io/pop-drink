use std::time::SystemTime;

pub use frame_metadata::RuntimeMetadataPrefixed;
use frame_support::{
    sp_runtime::{
        traits::{Header, One},
        BuildStorage,
    },
    traits::Hooks,
};
use frame_system::pallet_prelude::BlockNumberFor;
use ink_sandbox::TestExternalities;
pub use {
    frame_support::{
        self,
        sp_runtime::{AccountId32, DispatchError},
    },
    frame_system, pallet_balances, pallet_contracts, pallet_timestamp,
    sp_externalities::Extension,
};

/// Alias for the account ID type.
pub type AccountIdFor<R> = <R as frame_system::Config>::AccountId;

/// Used for handle block events.
pub struct BlockBuilder<T>(std::marker::PhantomData<T>);

impl<
        T: pallet_balances::Config + pallet_timestamp::Config<Moment = u64> + pallet_contracts::Config,
    > BlockBuilder<T>
{
    /// Create a new externalities with the given balances.
    pub fn new_ext(
        balances: Vec<(<T as frame_system::Config>::AccountId, T::Balance)>,
    ) -> TestExternalities {
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
