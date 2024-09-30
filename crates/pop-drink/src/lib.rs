pub use drink::*;
pub use ink_sandbox::api::{assets_api::AssetsAPI};
pub use sp_io::TestExternalities;
pub use frame_support::{self, sp_runtime::DispatchError, assert_ok};
pub use session::{error::SessionError, ContractBundle, Session, NO_SALT};
use scale::Decode;
use ink_sandbox::{AccountIdFor, BalanceFor};

pub mod devnet {
    use super::*;
    pub use pop_runtime_devnet::{BuildStorage, Runtime};

    // Types used in the pop runtime.
    pub type Balance = BalanceFor<Runtime>;
    pub type AccountId = AccountIdFor<Runtime>;

    // This is used to resolve type mismatches between the `AccountId` in the quasi testing
    // environment and the contract environment.
    pub fn account_id_from_slice(s: &AccountId) -> pop_api::primitives::AccountId {
        let account: [u8; 32] = s.clone().into();
        utils::account_id_from_slice(&account)
    }
}

pub mod utils {
    use super::*;
    pub fn deploy<S, E>
    (
        session: &mut Session<S>,
        bundle: ContractBundle,
        method: &str,
        input: Vec<String>,
        salt: Vec<u8>,
        init_value: Option<BalanceFor<S::Runtime>>,
    )
        -> Result<AccountIdFor<S::Runtime>, E>
        where
            S: Sandbox,
            S::Runtime: pallet_contracts::Config,
            E: Decode,
    {
        let result = session.deploy_bundle(bundle, method, &input, salt, init_value);
        if result.is_err() {
            let deployment_result = session.record().last_deploy_result().result.clone();
            let error = deployment_result.unwrap().result.data;
            return Err(E::decode(&mut &error[2..]).unwrap());
        }
        Ok(result.unwrap())
    }

    // Call a contract method and decode the returned data.
    pub fn call<S, O, E>(
        session: &mut Session<S>,
        func_name: &str,
        input: Vec<String>,
        endowment: Option<BalanceFor<S::Runtime>>,
    ) -> Result<O, E>
    where
        S: Sandbox,
        S::Runtime: pallet_contracts::Config,
        O: Decode,
        E: Decode,
    {
        match session.call::<String, ()>(func_name, &input, endowment) {
            // If the call is reverted, decode the error into `PSP22Error`.
            Err(SessionError::CallReverted(error)) =>
                Err(E::decode(&mut &error[2..])
                    .unwrap_or_else(|_| panic!("Decoding failed"))),
            // If the call is successful, decode the last returned value.
            Ok(_) => Ok(session
                .record()
                .last_call_return_decoded::<O>()
                .unwrap_or_else(|_| panic!("Expected a return value"))
                .unwrap_or_else(|_| panic!("Decoding failed"))),
            // Catch-all for unexpected results.
            _ => panic!("Expected call to revert or return a value"),
        }
    }

    // This is used to resolve type mismatches between the `AccountId` in the quasi testing
    // environment and the contract environment.
    pub fn account_id_from_slice(s: &[u8; 32]) -> pop_api::primitives::AccountId {
        pop_api::primitives::AccountId::decode(&mut &s[..]).expect("Should be decoded to AccountId")
    }

    // Get the last event from pallet contracts.
    pub fn last_contract_event<S>(session: &Session<S>) -> Option<Vec<u8>>
        where
            S: Sandbox,
            S::Runtime: pallet_contracts::Config,
            <S::Runtime as frame_system::Config>::RuntimeEvent: TryInto<pallet_contracts::Event<S::Runtime>>,
    {
        session.record().last_event_batch().contract_events().last().cloned()
    }
}
