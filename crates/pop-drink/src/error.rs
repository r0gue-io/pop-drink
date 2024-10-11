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

/// Runtime error for efficiently testing both runtime module errors and API errors.
/// It is designed for use with the `assert_err!` macro.
///
/// # Generic Parameters
///
/// - `ModuleError` - Error type of the runtime modules. [Reference](https://paritytech.github.io/polkadot-sdk/master/solochain_template_runtime/enum.Error.html).
/// - `ApiError` - Error type of the API, which depends on version. [Reference](https://github.com/r0gue-io/pop-node/tree/main/pop-api).
/// - `MODULE_INDEX` - Index of the variant `Error::Module`. This is based on the index of [`ApiError::Module`](https://github.com/r0gue-io/pop-node/blob/main/primitives/src/lib.rs#L38).
///
/// # Examples
///
/// ### Runtime module errors
///
/// - Import types to construct runtime module errors:
///
/// ```rs
/// use drink::devnet::{
/// 	Assets,
/// 	AssetsError::AssetNotLive,
/// 	Balances::BelowMinimum,
/// 	BalancesError::BelowMinimum
/// };
/// ```
///
/// - Construct a runtime module error [`Assets(AssetNotLive)`](https://paritytech.github.io/polkadot-sdk/master/pallet_assets/pallet/enum.Error.html#variant.AssetNotLive):
///
/// ```rs
/// Error::Module(Assets(AssetNotLive))
/// ```
///
/// - Construct a runtime module error [`Balances(InsufficientBalance)`](https://docs.rs/pallet-balances/latest/pallet_balances/pallet/enum.Error.html#variant.InsufficientBalance):
///
/// ```rs
/// Error::Module(Balances(InsufficientBalance))
/// ```
///
/// ### API errors
///
/// - Import types to construct API errors:
///
/// ```rs
/// use drink::devnet::v0::{
/// 	Arithmetic,
/// 	ArithmeticError::Overflow,
/// 	BadOrigin
/// };
/// ```
///
/// - API error [`Arithmetic(Overflow)`](https://github.com/r0gue-io/pop-node/blob/main/primitives/src/lib.rs#L55):
///
/// ```rs
/// Error::Api(Arithmetic(Overflow))
/// ```
///
/// - API error [`BadOrigin`](https://github.com/r0gue-io/pop-node/blob/main/primitives/src/lib.rs#L36C4-L36C18):
///
/// ```rs
/// Error::Api(BadOrigin)
/// ```
#[derive(Encode, Decode, Debug)]
pub enum Error<ModuleError, ApiError, const MODULE_INDEX: u8>
where
	ModuleError: Decode + Encode + Debug,
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
{
	/// Error type of the runtime modules. [Reference](https://paritytech.github.io/polkadot-sdk/master/solochain_template_runtime/enum.Error.html).
	Module(ModuleError),
	/// Every [`ApiError`](https://github.com/r0gue-io/pop-node/blob/52fb7f06a89955d462900e33d2b9c9170c4534a0/primitives/src/lib.rs#L30).
	Api(ApiError),
}

impl<ModuleError, ApiError, const MODULE_INDEX: u8> From<Error<ModuleError, ApiError, MODULE_INDEX>>
	for u32
where
	ModuleError: Decode + Encode + Debug,
	ApiError: Decode + Encode + Debug + From<u32> + Into<u32>,
{
	/// Converts an `Error` into a numerical value of `ApiError`.
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
	/// Converts a numerical value of `ApiError` into an `Error`.
	///
	/// This is used to reconstruct and display an `Error` from its numerical representation
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

/// Asserts that a `Result` with an error type convertible to `u32` matches the expected `Error`
/// from pop-drink.
///
/// # Parameters
///
/// - `result` - The `Result<R, E>` from a smart contract method, where `E` must be convertible to
///   `u32`.
/// - `error` - The expected runtime specific `Error` to assert against the `E` error type from
///   `result`.
///
/// # Examples
///
/// The below example interacts with a [PSP22](https://github.com/w3f/PSPs/blob/master/PSPs/psp-22.md) contract that uses [Pop API](https://github.com/r0gue-io/pop-node/tree/main/pop-api). The contract method returns [`PSP22Error`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/v0/fungibles/errors.rs#L73C1-L73C22) which is provided by Pop API library.
/// Learn more in the [PSP22 example contract](https://github.com/r0gue-io/pop-node/blob/main/pop-api/examples/fungibles/lib.rs).
///
/// The macro is used to test a customer error which is returned by the API if the error doesn't conform to the [`PSP22Error`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/v0/fungibles/errors.rs#L73C1-L73C22) standard.
/// The custom error is represented by a [`StatusCode`](https://github.com/r0gue-io/pop-node/blob/main/pop-api/src/lib.rs#L33), which encapsulates a `u32` value indicating the success or failure of a runtime call via the Pop API.
///
/// Pop DRink! provides an error type and a [macro](https://doc.rust-lang.org/book/ch19-06-macros.html) to simplify testing both runtime module errors and API errors.
///
/// - `Error`: Runtime error for efficiently testing both runtime module errors and API errors.
/// - `assert_err`: Asserts that a `Result` with an error type convertible to `u32` matches the
///   expected `Error` from pop-drink.
///
/// Note: `PSP22Error` is used here only as an example. The test suite utility library provided by
/// Pop DRink! is not limited to a single specific error type.
///
/// ```rs
/// // Required imports to test the custom error.
/// use drink::{
/// 	assert_err,
/// 	devnet::{
/// 		error::{
/// 			v0::Error,
/// 			Assets,
/// 			AssetsError::AssetNotLive,
/// 		},
/// 	},
/// };
///
/// // `PSP22Error` is provided by the API library.
/// use pop_api::v0::fungibles::PSP22Error;
/// ```
///
/// - Example `pop-drink` testing method to interact with PSP22 contract.
///
/// ```rs
/// fn transfer(session: &mut Session<Pop>, to: AccountId, amount: Balance) -> Result<(), PSP22Error> {
/// 	call::<Pop, (), PSP22Error>(
/// 		session,
/// 		"Psp22::transfer",
/// 		vec![to.to_string(), amount.to_string(), serde_json::to_string::<[u8; 0]>(&[]).unwrap()],
/// 		None,
/// 	)
/// }
/// ```
///
/// - Using macro to test the returned error.
///
/// ```rs
/// assert_err!(
/// 	transfer(&mut session, ALICE, AMOUNT),
/// 	Error::Module(Assets(AssetNotLive))
/// );
/// ```
#[macro_export]
macro_rules! assert_err {
	($result:expr, $error:expr $(,)?) => {
		$crate::error::assert_err_inner::<_, _, _>($result, $error);
	};
}

/// Asserts that a `Result` with an error type convertible to `u32` matches the expected `Error`
/// from pop-drink.
///
/// # Generic parameters
///
/// - `R` - Type returned if `result` is `Ok()`.
/// - `E` - Type returend if `result` is `Err()`. Must be convertible to `u32`.
/// - `Error` - Runtime error type.
///
/// # Parameters
///
/// - `result` - The `Result<R, E>` from a smart contract method, where `E` must be convertible to
///   `u32`.
/// - `expected_error` - The expected runtime specific `Error` to assert against the `E` error type
///   from `result`.
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
