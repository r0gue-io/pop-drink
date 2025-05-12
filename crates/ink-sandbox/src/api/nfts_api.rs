use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	sp_runtime::{
		traits::{Dispatchable, StaticLookup},
		DispatchError,
	},
	traits::{nonfungibles_v2::Inspect, Incrementable},
};
use pallet_nfts::{DestroyWitness, Instance1, MintWitness};

use crate::{AccountIdFor, RuntimeCall, Sandbox};

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
type AccountBalanceOf<T> = pallet_nfts::AccountBalance<T, Instance1>;
type CollectionIdOf<T> =
	<NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
type ItemIdOf<T> = <NftsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
type CollectionConfigFor<T> = pallet_nfts::CollectionConfigFor<T, Instance1>;
type DepositBalanceOf<T> = pallet_nfts::DepositBalanceOf<T, Instance1>;
type NextCollectionIdOf<T> = pallet_nfts::NextCollectionId<T, Instance1>;
type NftsOf<T> = pallet_nfts::Pallet<T, Instance1>;

/// Assets API for the sandbox.
pub trait NftsAPI<T: Sandbox>
where
	T: Sandbox,
	T::Runtime: pallet_nfts::Config<Instance1>,
{
	fn create<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		admin: &AccountIdLookupOf<T::Runtime>,
		config: CollectionConfigFor<T::Runtime>,
	) -> Result<(), DispatchError>;

	fn destroy<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime>,
		witness: DestroyWitness,
	) -> DispatchResultWithPostInfo;

	fn mint<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime>,
		item: ItemIdOf<T::Runtime>,
		mint_to: AccountIdLookupOf<T::Runtime>,
		witness_data: Option<MintWitness<ItemIdOf<T::Runtime>, DepositBalanceOf<T::Runtime>>>,
	) -> Result<(), DispatchError>;

	fn burn<Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime>,
		item: ItemIdOf<T::Runtime>,
	) -> Result<(), DispatchError>;

	fn transfer<
		Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>,
	>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime>,
		item: ItemIdOf<T::Runtime>,
		dest: AccountIdLookupOf<T::Runtime>,
	) -> Result<(), DispatchError>;

	fn next_collection_id(&mut self) -> Option<CollectionIdOf<T::Runtime>>;

	fn collection_owner(
		&mut self,
		collection: &CollectionIdOf<T::Runtime>,
	) -> Option<AccountIdFor<T::Runtime>>;

	fn balance_of(
		&mut self,
		collection: &CollectionIdOf<T::Runtime>,
		account: &AccountIdFor<T::Runtime>,
	) -> u32;

	fn owner(
		&mut self,
		collection: &CollectionIdOf<T::Runtime>,
		item: &ItemIdOf<T::Runtime>,
	) -> Option<AccountIdFor<T::Runtime>>;
}

impl<T> NftsAPI<T> for T
where
	T: Sandbox,
	T::Runtime: pallet_nfts::Config<Instance1>,
{
	fn create<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		admin: &AccountIdLookupOf<T::Runtime>,
		config: CollectionConfigFor<T::Runtime>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, Instance1>>::create(
				origin.into(),
				admin.clone(),
				config,
			)
		})
	}

	fn destroy<
		Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>,
	>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<<T as Sandbox>::Runtime>,
		witness: DestroyWitness,
	) -> DispatchResultWithPostInfo {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, Instance1>>::destroy(
				origin.into(),
				collection,
				witness,
			)
		})
	}

	fn mint<Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<<T as Sandbox>::Runtime>,
		item: ItemIdOf<<T as Sandbox>::Runtime>,
		mint_to: AccountIdLookupOf<<T as Sandbox>::Runtime>,
		witness_data: Option<
			MintWitness<
				ItemIdOf<<T as Sandbox>::Runtime>,
				DepositBalanceOf<<T as Sandbox>::Runtime>,
			>,
		>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, Instance1>>::mint(
				origin.into(),
				collection,
				item,
				mint_to,
				witness_data,
			)
		})
	}

	fn burn<Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime>,
		item: ItemIdOf<T::Runtime>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, Instance1>>::burn(origin.into(), collection, item)
		})
	}

	fn transfer<
		Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>,
	>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<<T as Sandbox>::Runtime>,
		item: ItemIdOf<<T as Sandbox>::Runtime>,
		dest: AccountIdLookupOf<<T as Sandbox>::Runtime>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, Instance1>>::transfer(
				origin.into(),
				collection,
				item,
				dest,
			)
		})
	}

	fn next_collection_id(&mut self) -> Option<CollectionIdOf<<T as Sandbox>::Runtime>> {
		self.execute_with(|| {
			NextCollectionIdOf::<T::Runtime>::get()
				.or(CollectionIdOf::<T::Runtime>::initial_value())
		})
	}

	fn collection_owner(
		&mut self,
		collection: &CollectionIdOf<<T as Sandbox>::Runtime>,
	) -> Option<AccountIdFor<<T as Sandbox>::Runtime>> {
		self.execute_with(|| {
    		<pallet_nfts::Pallet<T::Runtime, Instance1> as Inspect<AccountIdFor<T::Runtime>>>::collection_owner(
    			collection,
    		)
    	})
	}

	fn balance_of(
		&mut self,
		collection: &CollectionIdOf<T::Runtime>,
		account: &AccountIdFor<T::Runtime>,
	) -> u32 {
		self.execute_with(|| {
			AccountBalanceOf::<T::Runtime>::get(collection, account)
				.map(|(balance, _)| balance)
				.unwrap_or_default()
		})
	}

	fn owner(
		&mut self,
		collection: &CollectionIdOf<T::Runtime>,
		item: &ItemIdOf<T::Runtime>,
	) -> Option<AccountIdFor<T::Runtime>> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, Instance1> as Inspect<AccountIdFor<T::Runtime>>>::owner(
				collection, item,
			)
		})
	}
}

#[cfg(test)]
mod test {
	use pallet_contracts::test_utils::{ALICE, BOB};
	use pallet_nfts::{CollectionConfig, CollectionSettings, MintSettings};

	use super::*;
	use crate::{api::prelude::NftsAPI, DefaultSandbox};

	#[test]
	fn api_works() -> Result<(), DispatchError> {
		let mut sandbox = DefaultSandbox::default();
		let actor = DefaultSandbox::default_actor();
		let collection = sandbox.next_collection_id().unwrap_or_default();
		let item = 1;

		let config = CollectionConfig {
			settings: CollectionSettings::all_enabled(),
			mint_settings: MintSettings::default(),
			max_supply: None,
		};
		sandbox.create(Some(actor.clone()), &ALICE.into(), config)?;
		assert_eq!(sandbox.collection_owner(&collection), Some(actor.clone()));

		sandbox.mint(Some(actor.clone()), collection, item, actor.clone().into(), None)?;
		assert_eq!(sandbox.balance_of(&collection, &actor), 1);
		assert_eq!(sandbox.owner(&collection, &item), Some(actor.clone()));

		sandbox.transfer(Some(actor.clone()), collection, item, BOB.into())?;
		assert_eq!(sandbox.balance_of(&collection, &BOB), 1);
		assert_eq!(sandbox.owner(&collection, &item), Some(BOB));

		Ok(())
	}
}
