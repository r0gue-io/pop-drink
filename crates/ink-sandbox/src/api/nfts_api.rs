use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	sp_runtime::{traits::Dispatchable, DispatchError},
	traits::{nonfungibles_v2::Inspect, Incrementable},
};
use pallet_nfts::{
	AccountBalance, Collection, CollectionConfigFor, CollectionDetailsFor, DepositBalanceOf,
	DestroyWitness, Item, ItemDetailsFor, MintWitness, NextCollectionId,
};

use crate::{AccountIdFor, AccountIdLookupOf, RuntimeCall, Sandbox};

type CollectionIdOf<T, I = ()> =
	<NftsOf<T, I> as Inspect<<T as frame_system::Config>::AccountId>>::CollectionId;
type ItemIdOf<T, I = ()> =
	<NftsOf<T, I> as Inspect<<T as frame_system::Config>::AccountId>>::ItemId;
type NftsOf<T, I = ()> = pallet_nfts::Pallet<T, I>;

type MintWitnessData<T, I = ()> = MintWitness<ItemIdOf<T, I>, DepositBalanceOf<T, I>>;

/// Nfts API for the sandbox.
pub trait NftsAPI<T: Sandbox, I: 'static = ()>
where
	T: Sandbox,
	T::Runtime: pallet_nfts::Config<I>,
{
	/// Creates an NFT collection.
	///
	/// # Arguments
	/// * `admin` - The admin account of the collection.
	/// * `config` - Settings and config to be set for the new collection.
	fn create<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		admin: &AccountIdLookupOf<T::Runtime>,
		config: CollectionConfigFor<T::Runtime, I>,
	) -> Result<(), DispatchError>;

	/// Destroy an NFT collection.
	///
	/// # Arguments
	/// * `collection` - The collection to be destroyed.
	/// * `witness` - Information on the items minted in the `collection`. This must be correct.
	fn destroy<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime, I>,
		witness: DestroyWitness,
	) -> DispatchResultWithPostInfo;

	/// Mints an item to the specified recipient account.
	///
	/// # Arguments
	/// * `collection` - The collection.
	/// * `item` - The identifier for the new item.
	/// * `mint_to` - The recipient account.
	/// * `witness_data`- Information on the items minted in the `collection`. This must be correct.
	///
	/// Note: The deposit will be taken from the `origin` and not the `owner` of the `item`.
	fn mint<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime, I>,
		item: ItemIdOf<T::Runtime, I>,
		mint_to: AccountIdLookupOf<T::Runtime>,
		witness_data: Option<MintWitnessData<T::Runtime, I>>,
	) -> Result<(), DispatchError>;

	/// Destroys the specified item. Clearing the corresponding approvals.
	///
	/// # Arguments
	/// * `collection` - The collection.
	/// * `item` - The item to burn.
	fn burn<Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime, I>,
		item: ItemIdOf<T::Runtime, I>,
	) -> Result<(), DispatchError>;

	/// Transfers an owned or approved item to the specified recipient.
	///
	/// Origin must be either the item's owner or an account approved by the owner to
	/// transfer the item.
	///
	/// # Arguments
	/// * `collection` - The collection.
	/// * `item` - The item.
	/// * `dest` - The recipient account.
	fn transfer<
		Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>,
	>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime, I>,
		item: ItemIdOf<T::Runtime, I>,
		dest: AccountIdLookupOf<T::Runtime>,
	) -> Result<(), DispatchError>;

	/// Returns the next collection identifier, if any.
	fn next_collection_id(&mut self) -> Option<CollectionIdOf<T::Runtime, I>>;

	/// Returns the collection, if any.
	///
	/// # Arguments
	/// * `id` - The collection.
	fn collection(
		&mut self,
		id: &CollectionIdOf<T::Runtime, I>,
	) -> Option<CollectionDetailsFor<T::Runtime, I>>;

	/// Returns the collection item, if any.
	///
	/// # Arguments
	/// * `collection` - The collection.
	/// * `id` - The item ID.
	fn item(
		&mut self,
		collection: &CollectionIdOf<T::Runtime, I>,
		id: &ItemIdOf<T::Runtime, I>,
	) -> Option<ItemDetailsFor<T::Runtime, I>>;

	/// Returns the owner of a collection, if any.
	///
	/// # Arguments
	/// * `collection` - The collection.
	fn collection_owner(
		&mut self,
		collection: &CollectionIdOf<T::Runtime, I>,
	) -> Option<AccountIdFor<T::Runtime>>;

	/// Returns the number of items the owner has within a collection.
	///
	/// # Arguments
	/// * `collection` - The collection.
	/// * `account` - The account that owns items in the collection.
	fn balance_of(
		&mut self,
		collection: &CollectionIdOf<T::Runtime, I>,
		account: &AccountIdFor<T::Runtime>,
	) -> u32;

	/// Returns the total supply of a collection.
	///
	/// # Arguments
	/// * `collection` - The collection.
	fn total_supply(&mut self, collection: CollectionIdOf<T::Runtime, I>) -> u32;

	/// Returns the owner of an item within a specified collection, if any.
	///
	/// # Arguments
	/// * `collection` - The collection.
	/// * `item` - The item.
	fn owner(
		&mut self,
		collection: &CollectionIdOf<T::Runtime, I>,
		item: &ItemIdOf<T::Runtime, I>,
	) -> Option<AccountIdFor<T::Runtime>>;
}

impl<T, I> NftsAPI<T, I> for T
where
	T: Sandbox,
	T::Runtime: pallet_nfts::Config<I>,
	I: 'static,
{
	fn create<Origin: Into<<RuntimeCall<T::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		admin: &AccountIdLookupOf<T::Runtime>,
		config: CollectionConfigFor<T::Runtime, I>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, I>>::create(origin.into(), admin.clone(), config)
		})
	}

	fn destroy<
		Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>,
	>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<T::Runtime, I>,
		witness: DestroyWitness,
	) -> DispatchResultWithPostInfo {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, I>>::destroy(origin.into(), collection, witness)
		})
	}

	fn mint<Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<<T as Sandbox>::Runtime, I>,
		item: ItemIdOf<<T as Sandbox>::Runtime, I>,
		mint_to: AccountIdLookupOf<<T as Sandbox>::Runtime>,
		witness_data: Option<MintWitnessData<T::Runtime, I>>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, I>>::mint(
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
		collection: CollectionIdOf<T::Runtime, I>,
		item: ItemIdOf<T::Runtime, I>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, I>>::burn(origin.into(), collection, item)
		})
	}

	fn transfer<
		Origin: Into<<RuntimeCall<<T as Sandbox>::Runtime> as Dispatchable>::RuntimeOrigin>,
	>(
		&mut self,
		origin: Origin,
		collection: CollectionIdOf<<T as Sandbox>::Runtime, I>,
		item: ItemIdOf<<T as Sandbox>::Runtime, I>,
		dest: AccountIdLookupOf<<T as Sandbox>::Runtime>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, I>>::transfer(origin.into(), collection, item, dest)
		})
	}

	fn next_collection_id(&mut self) -> Option<CollectionIdOf<<T as Sandbox>::Runtime, I>> {
		self.execute_with(|| {
			NextCollectionId::<T::Runtime, I>::get()
				.or(CollectionIdOf::<T::Runtime, I>::initial_value())
		})
	}

	fn collection(
		&mut self,
		id: &CollectionIdOf<<T as Sandbox>::Runtime, I>,
	) -> Option<CollectionDetailsFor<<T as Sandbox>::Runtime, I>> {
		self.execute_with(|| Collection::<T::Runtime, I>::get(id))
	}

	fn item(
		&mut self,
		collection: &CollectionIdOf<<T as Sandbox>::Runtime, I>,
		id: &ItemIdOf<<T as Sandbox>::Runtime, I>,
	) -> Option<ItemDetailsFor<<T as Sandbox>::Runtime, I>> {
		self.execute_with(|| Item::<T::Runtime, I>::get(collection, id))
	}

	fn collection_owner(
		&mut self,
		collection: &CollectionIdOf<<T as Sandbox>::Runtime, I>,
	) -> Option<AccountIdFor<<T as Sandbox>::Runtime>> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, I> as Inspect<AccountIdFor<T::Runtime>>>::collection_owner(
    			collection,
    		)
		})
	}

	fn balance_of(
		&mut self,
		collection: &CollectionIdOf<T::Runtime, I>,
		account: &AccountIdFor<T::Runtime>,
	) -> u32 {
		self.execute_with(|| {
			AccountBalance::<T::Runtime, I>::get(collection, account)
				.map(|(balance, _)| balance)
				.unwrap_or_default()
		})
	}

	fn total_supply(&mut self, collection: CollectionIdOf<T::Runtime, I>) -> u32 {
		self.execute_with(|| {
			NftsOf::<T::Runtime, I>::collection_items(collection).unwrap_or_default()
		})
	}

	fn owner(
		&mut self,
		collection: &CollectionIdOf<T::Runtime, I>,
		item: &ItemIdOf<T::Runtime, I>,
	) -> Option<AccountIdFor<T::Runtime>> {
		self.execute_with(|| {
			<pallet_nfts::Pallet<T::Runtime, I> as Inspect<AccountIdFor<T::Runtime>>>::owner(
				collection, item,
			)
		})
	}
}

#[cfg(test)]
mod test {
	use pallet_contracts::test_utils::{ALICE, BOB};
	use pallet_nfts::{CollectionConfig, CollectionDetails, CollectionSettings, MintSettings};

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
		assert_eq!(sandbox.total_supply(collection), 0);
		assert_eq!(sandbox.collection_owner(&collection), Some(actor.clone()));
		assert_eq!(
			sandbox.collection(&collection),
			Some(CollectionDetails {
				owner: actor.clone(),
				item_metadatas: 0,
				items: 0,
				attributes: 0,
				item_configs: 0,
				owner_deposit: 2
			})
		);

		sandbox.mint(Some(actor.clone()), collection, item, actor.clone().into(), None)?;
		assert_eq!(sandbox.total_supply(collection), 1);
		assert_eq!(sandbox.balance_of(&collection, &actor), 1);
		assert_eq!(sandbox.owner(&collection, &item), Some(actor.clone()));
		assert_eq!(sandbox.item(&collection, &item).map(|item| item.owner), Some(actor.clone()));

		sandbox.transfer(Some(actor), collection, item, BOB.into())?;
		assert_eq!(sandbox.balance_of(&collection, &BOB), 1);
		assert_eq!(sandbox.owner(&collection, &item), Some(BOB));

		Ok(())
	}
}
