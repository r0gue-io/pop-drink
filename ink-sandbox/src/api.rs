pub mod balances_api;
pub mod contracts_api;
pub mod system_api;
pub mod timestamp_api;
pub mod assets_api;

pub mod prelude {
    pub use super::{
        assets_api::AssetsAPI, balances_api::BalanceAPI, contracts_api::ContractAPI, system_api::SystemAPI,
        timestamp_api::TimestampAPI,
    };
}
