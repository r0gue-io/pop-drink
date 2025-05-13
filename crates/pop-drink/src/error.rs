//! The error type and utilities for testing smart contracts using the Pop API.

use std::fmt::Debug;

pub use drink::{
	pallet_assets::Error as AssetsError, pallet_balances::Error as BalancesError,
	pallet_contracts::Error as ContractsError, pallet_nfts::Error as NftsError,
};
use scale::{Decode, Encode};

/// A simplified error type representing errors from the runtime and its modules.
///
/// This type can be used to assert to an error that holds a [status code](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/lib.rs#L33).
/// The status code is returned by the Pop API and represents a runtime error.
///
/// # Generic Parameters:
/// - `ApiError`: The pop api error type.
/// - `ModuleError`: The error type for specific runtime modules.
/// - `MODULE_INDEX`: Index of the variant `Error::Module`.
#[derive(Encode, Decode, Debug)]
pub enum Error<ApiError, ModuleError, const MODULE_INDEX: u8>
where
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
	ModuleError: Decode + Encode + Debug,
{
	/// An error not related to any specific module.
	Raw(ApiError),
	/// An error originating from a runtime module.
	Module(ModuleError),
}

impl<ApiError, ModuleError, const MODULE_INDEX: u8> From<Error<ApiError, ModuleError, MODULE_INDEX>>
	for u32
where
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
	ModuleError: Decode + Encode + Debug,
{
	/// Converts an `Error` to a `u32` status code.
	fn from(error: Error<ApiError, ModuleError, MODULE_INDEX>) -> Self {
		match error {
			Error::Raw(error) => decode::<ApiError>(&error.encode()),
			Error::Module(error) => {
				let mut encoded = error.encode();
				encoded.insert(0, MODULE_INDEX);
				encoded.resize(4, 0);
				decode::<ApiError>(&encoded)
			},
		}
		.into()
	}
}

impl<ApiError, ModuleError, const MODULE_INDEX: u8> From<u32>
	for Error<ApiError, ModuleError, MODULE_INDEX>
where
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
	ModuleError: Decode + Encode + Debug,
{
	/// Converts a `u32` into an `Error`.
	///
	/// If the status code represents a module error it converts it into `Error::Module` in stead
	/// of `Error::Raw`.
	fn from(value: u32) -> Self {
		let error = ApiError::from(value);
		let encoded = error.encode();
		if encoded[0] == MODULE_INDEX {
			let (index, module_error) = (encoded[1], &encoded[2..]);
			let data = [vec![index], module_error.to_vec()].concat();
			return Error::Module(decode(&data));
		}
		Error::Raw(error)
	}
}

fn decode<T: Decode>(data: &[u8]) -> T {
	T::decode(&mut &data[..]).expect("Decoding failed")
}

#[cfg(test)]
mod test {
	use pop_api::primitives::v0::Error as ApiError;

	use crate::error::{AssetsError::*, BalancesError::*, *};

	fn test_cases() -> Vec<(Error<ApiError, crate::mock::RuntimeError, 3>, ApiError)> {
		use frame_support::traits::PalletInfoAccess;
		use pop_api::primitives::{ArithmeticError::*, TokenError::*};

		use crate::mock::RuntimeError::*;
		vec![
			(Error::Raw(ApiError::BadOrigin), ApiError::BadOrigin),
			(Error::Raw(ApiError::Token(BelowMinimum)), ApiError::Token(BelowMinimum)),
			(Error::Raw(ApiError::Arithmetic(Overflow)), ApiError::Arithmetic(Overflow)),
			(
				Error::Module(Assets(BalanceLow)),
				ApiError::Module { index: crate::mock::Assets::index() as u8, error: [0, 0] },
			),
			(
				Error::Module(Assets(NoAccount)),
				ApiError::Module { index: crate::mock::Assets::index() as u8, error: [1, 0] },
			),
			(
				Error::Module(Balances(VestingBalance)),
				ApiError::Module { index: crate::mock::Balances::index() as u8, error: [0, 0] },
			),
			(
				Error::Module(Balances(LiquidityRestrictions)),
				ApiError::Module { index: crate::mock::Balances::index() as u8, error: [1, 0] },
			),
		]
	}

	#[test]
	fn runtime_error_to_primitives_error_conversion_works() {
		test_cases().into_iter().for_each(|t| {
			let runtime_error: u32 = t.0.into();
			let pop_api_error: u32 = t.1.into();
			assert_eq!(runtime_error, pop_api_error);
		});
	}

	#[test]
	fn assert_err_works() {
		test_cases().into_iter().for_each(|t| {
			crate::assert_err!(Result::<(), pop_api::primitives::v0::Error>::Err(t.1), t.0,);
		});
	}
}
