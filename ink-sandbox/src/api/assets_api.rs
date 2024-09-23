use frame_support::{
    sp_runtime::DispatchError,
    traits::fungibles::{
        approvals::{Inspect as ApprovalsInspect, Mutate as ApprovalsMutate},
        Create, Inspect, Mutate,
    },
};
use pallet_assets::Instance1;

use crate::{AccountIdFor, Sandbox};

type AssetIdOf<T> = <AssetsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;
type AssetsOf<T> = pallet_assets::Pallet<T, Instance1>;
type BalanceOf<T> = <AssetsOf<T> as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

/// Assets API for the sandbox.
pub trait AssetsAPI<T: Sandbox>
where
    T: Sandbox,
    T::Runtime: pallet_assets::Config<Instance1>,
{
    /// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
    ///
    /// # Arguments
    /// * `id` - ID of the new asset to be created.
    /// * `owner` - The owner of the created asset.
    /// * `min_balance` - The asset amount one account need at least.
    fn create(
        &mut self,
        id: &AssetIdOf<T::Runtime>,
        owner: &AccountIdFor<T::Runtime>,
        min_balance: BalanceOf<T::Runtime>,
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
        asset: &AssetIdOf<T::Runtime>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
        amount: BalanceOf<T::Runtime>,
    ) -> Result<(), DispatchError>;

    /// Creates `value` amount of tokens and assigns them to `account`, increasing the total supply.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `account` - The account to be credited with the created tokens.
    /// * `value` - The number of tokens to mint.
    fn mint_into(
        &mut self,
        asset: &AssetIdOf<T::Runtime>,
        account: &AccountIdFor<T::Runtime>,
        value: BalanceOf<T::Runtime>,
    ) -> Result<BalanceOf<T::Runtime>, DispatchError>;

    /// Returns the account balance for the specified `owner`.
    ///
    /// # Arguments
    /// * `owner` - The account whose balance is being queried.
    fn balance_of(
        &mut self,
        asset: &AssetIdOf<T::Runtime>,
        owner: &AccountIdFor<T::Runtime>,
    ) -> BalanceOf<T::Runtime>;

    /// Returns the total supply of the `asset`.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    fn total_supply(&mut self, asset: &AssetIdOf<T::Runtime>) -> BalanceOf<T::Runtime>;

    /// Returns the allowance for a `spender` approved by an `owner`.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
    /// * `owner` - The account that owns the tokens.
    /// * `spender` - The account that is allowed to spend the tokens.
    fn allowance(
        &mut self,
        asset: &AssetIdOf<T::Runtime>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
    ) -> BalanceOf<T::Runtime>;

    /// Check if the asset is created.
    ///
    /// # Arguments
    /// * `asset` - ID of the asset.
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

    fn approve(
        &mut self,
        asset: &AssetIdOf<T::Runtime>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
        amount: BalanceOf<T::Runtime>,
    ) -> Result<(), DispatchError> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, Instance1>::approve(
                asset.clone(),
                owner,
                delegate,
                amount,
            )
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

    fn total_supply(&mut self, asset: &AssetIdOf<T::Runtime>) -> BalanceOf<T::Runtime> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, Instance1>::total_supply(asset.clone())
        })
    }

    fn allowance(
        &mut self,
        asset: &AssetIdOf<T::Runtime>,
        owner: &AccountIdFor<T::Runtime>,
        delegate: &AccountIdFor<T::Runtime>,
    ) -> BalanceOf<T::Runtime> {
        self.execute_with(|| {
            pallet_assets::Pallet::<T::Runtime, Instance1>::allowance(
                asset.clone(),
                owner,
                delegate,
            )
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
        let actor = DefaultSandbox::default_actor();
        let balance = sandbox.balance_of(&token, &actor);

        sandbox.create(&token, &actor, 1).unwrap();
        sandbox.mint_into(&token, &actor, 100).unwrap();
        assert_eq!(sandbox.balance_of(&token, &actor), balance + 100);

        assert!(sandbox.asset_exists(&token));
    }
}
