use frame_support::{
	sp_runtime::DispatchError,
	traits::fungibles::{
		approvals::{Inspect as _, Mutate as _},
		Create, Destroy, Inspect, Mutate,
	},
};

use crate::{AccountIdFor, OriginFor, Sandbox};

type AssetIdOf<T, I> = <AssetsOf<T, I> as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type AssetsOf<T, I> = pallet_assets::Pallet<T, I>;
type BalanceOf<T, I> = <AssetsOf<T, I> as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Assets API for the sandbox.
pub trait AssetsAPI<T: Sandbox, I: 'static = ()>
where
	T: Sandbox,
	T::Runtime: pallet_assets::Config<I>,
{
	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
	///
	/// # Arguments
	/// * `id` - ID of the new asset to be created.
	/// * `owner` - The owner of the created asset.
	/// * `min_balance` - The asset amount one account need at least.
	fn create(
		&mut self,
		id: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
		min_balance: BalanceOf<T::Runtime, I>,
	) -> Result<(), DispatchError>;

	/// Start the destruction an existing fungible asset.
	///
	/// # Arguments
	/// * `asset` - ID of the asset.
	fn start_destroy(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> Result<(), DispatchError>;

	/// Start the destruction an existing fungible asset.
	///
	/// # Arguments
	/// * `asset` - ID of the asset.
	/// * `name` - Token name.
	/// * `symbol` - Token symbol.
	/// * `decimals` - Token decimals.
	fn set_metadata(
		&mut self,
		origin: impl Into<OriginFor<T>>,
		asset: &AssetIdOf<T::Runtime, I>,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> Result<(), DispatchError>;

	/// Approves `spender` to spend `value` amount of tokens on behalf of the caller.
	///
	/// Successive calls of this method overwrite previous values.
	///
	/// # Arguments
	/// * `asset` - ID of the asset.
	/// * `spender` - The account that is allowed to spend the tokens.
	/// * `value` - The number of tokens to approve.
	fn approve(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
		delegate: &AccountIdFor<T::Runtime>,
		amount: BalanceOf<T::Runtime, I>,
	) -> Result<(), DispatchError>;

	/// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
	///
	/// # Arguments
	/// * `asset` - ID of the asset.
	/// * `account` - The account to be credited with the created tokens.
	/// * `value` - The number of tokens to mint.
	fn mint_into(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		account: &AccountIdFor<T::Runtime>,
		value: BalanceOf<T::Runtime, I>,
	) -> Result<BalanceOf<T::Runtime, I>, DispatchError>;

	/// Returns the account balance for the specified `owner`.
	///
	/// # Arguments
	/// * `owner` - The account whose balance is being queried.
	fn balance_of(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
	) -> BalanceOf<T::Runtime, I>;

	/// Returns the total supply of the `asset`.
	///
	/// # Arguments
	/// * `asset` - ID of the asset.
	fn total_supply(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> BalanceOf<T::Runtime, I>;

	/// Returns the allowance for a `spender` approved by an `owner`.
	///
	/// # Arguments
	/// * `asset` - ID of the asset.
	/// * `owner` - The account that owns the tokens.
	/// * `spender` - The account that is allowed to spend the tokens.
	fn allowance(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
		delegate: &AccountIdFor<T::Runtime>,
	) -> BalanceOf<T::Runtime, I>;

	/// Check if the asset exists.
	///
	/// # Arguments
	/// * `asset` - ID of the asset.
	fn asset_exists(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> bool;
}

impl<T, I> AssetsAPI<T, I> for T
where
	T: Sandbox,
	T::Runtime: pallet_assets::Config<I>,
	I: 'static,
{
	fn create(
		&mut self,
		id: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
		min_balance: BalanceOf<T::Runtime, I>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			<pallet_assets::Pallet<T::Runtime, I> as Create<AccountIdFor<T::Runtime>>>::create(
				id.clone(),
				owner.clone(),
				true,
				min_balance,
			)
		})
	}

	fn start_destroy(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> Result<(), DispatchError> {
		self.execute_with(|| <pallet_assets::Pallet::<T::Runtime, I> as Destroy<AccountIdFor<T::Runtime>>>::start_destroy(asset.clone(), None))
	}

	fn set_metadata(
		&mut self,
		origin: impl Into<OriginFor<T>>,
		asset: &AssetIdOf<T::Runtime, I>,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			pallet_assets::Pallet::<T::Runtime, I>::set_metadata(
				origin.into(),
				asset.clone().into(),
				name,
				symbol,
				decimals,
			)
		})
	}

	fn mint_into(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		account: &AccountIdFor<T::Runtime>,
		value: BalanceOf<T::Runtime, I>,
	) -> Result<BalanceOf<T::Runtime, I>, DispatchError> {
		self.execute_with(|| {
			pallet_assets::Pallet::<T::Runtime, I>::mint_into(asset.clone(), account, value)
		})
	}

	fn approve(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
		delegate: &AccountIdFor<T::Runtime>,
		amount: BalanceOf<T::Runtime, I>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| {
			pallet_assets::Pallet::<T::Runtime, I>::approve(asset.clone(), owner, delegate, amount)
		})
	}

	fn balance_of(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
	) -> BalanceOf<T::Runtime, I> {
		self.execute_with(|| pallet_assets::Pallet::<T::Runtime, I>::balance(asset.clone(), owner))
	}

	fn total_supply(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> BalanceOf<T::Runtime, I> {
		self.execute_with(|| pallet_assets::Pallet::<T::Runtime, I>::total_supply(asset.clone()))
	}

	fn allowance(
		&mut self,
		asset: &AssetIdOf<T::Runtime, I>,
		owner: &AccountIdFor<T::Runtime>,
		delegate: &AccountIdFor<T::Runtime>,
	) -> BalanceOf<T::Runtime, I> {
		self.execute_with(|| {
			pallet_assets::Pallet::<T::Runtime, I>::allowance(asset.clone(), owner, delegate)
		})
	}

	fn asset_exists(&mut self, asset: &AssetIdOf<T::Runtime, I>) -> bool {
		self.execute_with(|| pallet_assets::Pallet::<T::Runtime, I>::asset_exists(asset.clone()))
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::DefaultSandbox;
	#[test]
	fn api_works() {
		let mut sandbox = DefaultSandbox::default();
		let token = 1;
		let actor = DefaultSandbox::default_actor();
		let balance = sandbox.balance_of(&token, &actor);

		sandbox.create(&token, &actor, 1).unwrap();
		sandbox.mint_into(&token, &actor, 100).unwrap();
		assert_eq!(sandbox.balance_of(&token, &actor), balance + 100);

		assert!(sandbox.asset_exists(&token));
	}
}
