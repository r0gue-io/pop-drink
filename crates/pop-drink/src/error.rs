//! A set of errors used for testing smart contracts.

use std::fmt::Debug;

pub use drink::{
	pallet_assets::Error as AssetsError, pallet_balances::Error as BalancesError,
	pallet_contracts::Error as ContractsError,
};
use scale::{Decode, Encode};

fn decode<T: Decode>(data: &[u8]) -> T {
	T::decode(&mut &data[..]).expect("Decoding failed")
}

/// Runtime error for testing.
///
/// # Generic Parameters:
///
/// - `ModuleError` - Error type of the runtime modules. Reference: https://paritytech.github.io/polkadot-sdk/master/solochain_template_runtime/enum.Error.html.
/// - `ApiError` - Error type of the API, which depends on version.
/// - `MODULE_INDEX` - Index of the variant `Error::Module`. This is based on the index of
///   `ApiError::Module`
#[derive(Encode, Decode, Debug)]
pub enum Error<ModuleError, ApiError, const MODULE_INDEX: u8>
where
	ModuleError: Decode + Encode + Debug,
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
{
	/// Module errors of the runtime.
	Module(ModuleError),
	/// Every `ApiError`.
	Api(ApiError),
}

impl<ModuleError, ApiError, const MODULE_INDEX: u8> From<Error<ModuleError, ApiError, MODULE_INDEX>>
	for u32
where
	ModuleError: Decode + Encode + Debug,
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
{
	/// Converts a `Error` into a numerical value of `ApiError`.
	///
	/// This conversion is necessary for comparing `Error` instances with other types.
	// Compared types must implement `Into<u32>`, as `Error` does not implement `Eq`.
	// Use this function to obtain a numerical representation of the error for comparison or
	// further processing.
	fn from(error: Error<ModuleError, ApiError, MODULE_INDEX>) -> Self {
		match error {
			Error::Module(error) => {
				let mut encoded = error.encode();
				encoded.insert(0, MODULE_INDEX);
				encoded.resize(4, 0);
				decode::<ApiError>(&encoded)
			},
			Error::Api(error) => decode::<ApiError>(&error.encode()),
		}
		.into()
	}
}

impl<ModuleError, ApiError, const MODULE_INDEX: u8> From<u32>
	for Error<ModuleError, ApiError, MODULE_INDEX>
where
	ModuleError: Decode + Encode + Debug,
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
{
	/// Converts a numerical value of `ApiError` into a `Error`.
	///
	/// This is used to reconstruct and display a `Error` from its numerical representation
	/// when an error is thrown.
	fn from(value: u32) -> Self {
		let error = ApiError::from(value);
		let encoded = error.encode();
		if encoded[0] == MODULE_INDEX {
			let (index, module_error) = (encoded[1], &encoded[2..]);
			let data = vec![vec![index], module_error.to_vec()].concat();
			return Error::Module(decode(&data));
		}
		Error::Api(error)
	}
}

/// A utility macro to assert that an error returned from a smart contract method using API
/// V0.
///
/// # Parameters:
///
/// - `result` - The result returned by a smart contract method. It is of type `Result<R, E>`, where
///   the error type `E` must implement a conversion to `u32`.
/// - `error` - A `Error` type configured specifically for the API V0.
///
/// # Example:
///
/// ```rs
/// use drink::devnet::{
/// 	error::{
/// 		v0::{assert_err, Error},
/// 		Assets,
/// 		AssetsError::AssetNotLive,
/// 	}
/// };
///
/// /// Example `pop-drink` testing method to interact with PSP22 contract.
/// fn transfer(session: &mut Session<Pop>, to: AccountId, amount: Balance) -> Result<(), PSP22Error> {
/// 	call::<Pop, (), PSP22Error>(
/// 		session,
/// 		"Psp22::transfer",
/// 		vec![to.to_string(), amount.to_string(), serde_json::to_string::<[u8; 0]>(&[]).unwrap()],
/// 		None,
/// 	)
/// }
///
/// /// Using macro to test the returned error.
/// assert_err!(
/// 	mint(&mut session, BOB, AMOUNT),
/// 	Error::Module(Assets(AssetNotLive))
/// );
/// ```
#[macro_export]
macro_rules! assert_err {
	($result:expr, $error:expr $(,)?) => {
		$crate::error::assert_err_inner::<_, _, _>($result, $error);
	};
}

/// A utility macro to assert that an error returned from a smart contract method matches the
/// `Error`.
///
/// # Generic parameters:
///
/// - `R` - Success type returned if Ok().
/// - `E` - Returned `Err()` value of a method result. Must be convertable to `u32`.
/// - `Error` - Runtime error type.
///
/// # Parameters:
///
/// - `result` - Result returned by a smart contract method.
/// - `expected_error` - `Error` to be asserted.
#[track_caller]
pub fn assert_err_inner<R, E, Error>(result: Result<R, E>, expected_error: Error)
where
	E: Into<u32>,
	Error: From<u32> + Into<u32> + Debug,
{
	let expected_code: u32 = expected_error.into();
	let expected_error = Error::from(expected_code);
	if let Err(error) = result {
		let error_code: u32 = error.into();
		if error_code != expected_code {
			panic!(
				r#"assertion `left == right` failed
  left: {:?}
 right: {:?}"#,
				Error::from(error_code),
				expected_error
			);
		}
	} else {
		panic!(
			r#"assertion `left == right` failed
  left: Ok()
 right: {:?}"#,
			expected_error
		);
	}
}

#[cfg(test)]
mod test {
	use pop_api::primitives::v0::Error as ApiError;

	use crate::error::{AssetsError::*, BalancesError::*, *};

	fn test_cases() -> Vec<(Error<crate::mock::RuntimeError, ApiError, 3>, ApiError)> {
		use frame_support::traits::PalletInfoAccess;
		use pop_api::primitives::{ArithmeticError::*, TokenError::*};

		use crate::mock::RuntimeError::*;
		vec![
			(Error::Api(ApiError::BadOrigin), ApiError::BadOrigin),
			(Error::Api(ApiError::Token(BelowMinimum)), ApiError::Token(BelowMinimum)),
			(Error::Api(ApiError::Arithmetic(Overflow)), ApiError::Arithmetic(Overflow)),
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
