use frame_support::{
	sp_runtime::DispatchError,
	traits::fungibles::{Create, Inspect, Mutate},
};
use pallet_assets::Instance1;

use crate::{AccountIdFor, Sandbox};

type AssetIdOf<T> = <AssetsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type AssetsOf<T> = pallet_assets::Pallet<T, Instance1>;
type BalanceOf<T> = <AssetsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

// TODO: api is the same with balances api. Perhaps should be changed to e.g. `create_asset`.
//  Not important for now though.
/// Assets API for the sandbox.
pub trait AssetsAPI<T: Sandbox>
where
	T: Sandbox,
	T::Runtime: pallet_assets::Config<Instance1>,
{
	/// Create a token.
	fn create(
		&mut self,
		id: &AssetIdOf<T::Runtime>,
		owner: &AccountIdFor<T::Runtime>,
		min_balance: BalanceOf<T::Runtime>,
	) -> Result<(), DispatchError>;

	/// Mint tokens to an account.
	///
	/// # Arguments
	///
	/// * `address` - The address of the account to add tokens to.
	/// * `amount` - The number of tokens to add.
	fn mint_into(
		&mut self,
		asset: &AssetIdOf<T::Runtime>,
		account: &AccountIdFor<T::Runtime>,
		value: BalanceOf<T::Runtime>,
	) -> Result<BalanceOf<T::Runtime>, DispatchError>;

	/// Return the balance of an account.
	///
	/// # Arguments
	///
	/// * `address` - The address of the account to query.
	fn balance_of(
		&mut self,
		asset: &AssetIdOf<T::Runtime>,
		owner: &AccountIdFor<T::Runtime>,
	) -> BalanceOf<T::Runtime>;

	fn asset_exists(&mut self, asset: &AssetIdOf<T::Runtime>) -> bool;
}

impl<T> AssetsAPI<T> for T
where
	T: Sandbox,
	T::Runtime: pallet_assets::Config<Instance1>,
{
	fn create(
		&mut self,
		id: &AssetIdOf<T::Runtime>,
		owner: &AccountIdFor<T::Runtime>,
		min_balance: BalanceOf<T::Runtime>,
	) -> Result<(), DispatchError> {
		self.execute_with(|| <pallet_assets::Pallet::<T::Runtime, Instance1> as Create<AccountIdFor<T::Runtime>>>::create(id.clone(), owner.clone(), true, min_balance))
	}

	fn mint_into(
		&mut self,
		asset: &AssetIdOf<T::Runtime>,
		account: &AccountIdFor<T::Runtime>,
		value: BalanceOf<T::Runtime>,
	) -> Result<BalanceOf<T::Runtime>, DispatchError> {
		self.execute_with(|| {
			pallet_assets::Pallet::<T::Runtime, Instance1>::mint_into(asset.clone(), account, value)
		})
	}

	fn balance_of(
		&mut self,
		asset: &AssetIdOf<T::Runtime>,
		owner: &AccountIdFor<T::Runtime>,
	) -> BalanceOf<T::Runtime> {
		self.execute_with(|| {
			pallet_assets::Pallet::<T::Runtime, Instance1>::balance(asset.clone(), owner)
		})
	}

	fn asset_exists(&mut self, asset: &AssetIdOf<T::Runtime>) -> bool {
		self.execute_with(|| {
			pallet_assets::Pallet::<T::Runtime, Instance1>::asset_exists(asset.clone())
		})
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
		let balance = sandbox.balance_of(&token, &DefaultSandbox::default_actor());

		sandbox.create(&token, &DefaultSandbox::default_actor(), 1).unwrap();
		sandbox.mint_into(&token, &DefaultSandbox::default_actor(), 100).unwrap();

		assert_eq!(sandbox.balance_of(&token, &DefaultSandbox::default_actor()), balance + 100);

		assert!(sandbox.asset_exists(&token));
	}
}
