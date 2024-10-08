//! Runtime error type for testing Pop API V0.

use std::fmt::Debug;

use pop_api::primitives::v0::Error as PopApiError;
pub use pop_api::primitives::v0::*;
use scale::{Decode, Encode};

use crate::utils::decode;

/// Runtime error type for testing Pop API V0.
#[derive(Encode, Decode, Debug)]
#[repr(u8)]
pub enum RuntimeError<E>
where
	E: Decode + Encode + Debug,
{
	Module(E),
	Raw(PopApiError),
}

impl<E> From<RuntimeError<E>> for u32
where
	E: Decode + Encode + Debug,
{
	/// Converts a `RuntimeError` into a numerical value of `pop_api::primitives::v0::Error`.
	///
	/// This conversion is necessary for comparing `RuntimeError` instances with other types
	/// that implement `Into<u32>`, as `RuntimeError` does not implement `Eq`.
	/// Use this function to obtain a numerical representation of the error for comparison or
	/// further processing.
	fn from(error: RuntimeError<E>) -> Self {
		let pop_api_error = match error {
			RuntimeError::Module(error) => {
				let encoded = error.encode();
				let (index, mut runtime_error) = (encoded[0], encoded[1..].to_vec());
				runtime_error.resize(2, 0);
				PopApiError::Module { index, error: [runtime_error[0], runtime_error[1]] }
			},
			RuntimeError::Raw(error) => decode::<PopApiError>(&error.encode()),
		};
		pop_api_error.into()
	}
}

impl<E> From<u32> for RuntimeError<E>
where
	E: Decode + Encode + Debug,
{
	/// Converts a numerical value of `pop_api::primitives::v0::Error` into a `RuntimeError`.
	///
	/// This is used to reconstruct and display a `RuntimeError` from its numerical representation
	/// when an error is thrown.
	fn from(value: u32) -> Self {
		let error = PopApiError::from(value);
		match error {
			PopApiError::Module { index, error } => {
				let data = vec![vec![index], error.to_vec()].concat();
				RuntimeError::Module(decode(&data))
			},
			_ => RuntimeError::Raw(error),
		}
	}
}

/// A  method to assert that an error returned from a method matches
/// the `RuntimeError` type using `pop_api::primitives::v0::Error`.
pub fn assert_runtime_err_inner<R, E, RuntimeError>(
	result: Result<R, E>,
	expected_error: RuntimeError,
) where
	// E: Returned Err() value of a method result. Must be convertable to `u32`.
	E: Into<u32>,
	// D: Expected RuntimeError.
	RuntimeError: From<u32> + Into<u32> + Debug,
{
	crate::errors::assert_runtime_err_inner::<PopApiError, R, E, RuntimeError>(
		result,
		expected_error,
	)
}

/// A utility macro to assert that an error returned from a smart contract method using
/// `pop_api::primitives::v0::Error` matches the `RuntimeError`.
#[macro_export]
macro_rules! __assert_runtime_err_v0 {
	($result:expr, $error:expr $(,)?) => {
		$crate::errors::v0::assert_runtime_err_inner::<_, _, _>($result, $error);
	};
}

pub use __assert_runtime_err_v0 as assert_runtime_err;

#[cfg(test)]
mod test {
	use pop_primitives::v0::Error as PopApiError;
	use v0::RuntimeError;

	use crate::errors::{AssetsError::*, BalancesError::*, *};

	fn test_cases() -> Vec<(RuntimeError<crate::mock::RuntimeError>, PopApiError)> {
		use frame_support::traits::PalletInfoAccess;
		use pop_api::primitives::{ArithmeticError::*, TokenError::*};

		use crate::mock::RuntimeError::*;
		vec![
			(RuntimeError::Raw(PopApiError::BadOrigin), PopApiError::BadOrigin),
			(RuntimeError::Raw(PopApiError::Token(BelowMinimum)), PopApiError::Token(BelowMinimum)),
			(
				RuntimeError::Raw(PopApiError::Arithmetic(Overflow)),
				PopApiError::Arithmetic(Overflow),
			),
			(
				RuntimeError::Module(Assets(BalanceLow)),
				PopApiError::Module { index: crate::mock::Assets::index() as u8, error: [0, 0] },
			),
			(
				RuntimeError::Module(Assets(NoAccount)),
				PopApiError::Module { index: crate::mock::Assets::index() as u8, error: [1, 0] },
			),
			(
				RuntimeError::Module(Balances(VestingBalance)),
				PopApiError::Module { index: crate::mock::Balances::index() as u8, error: [0, 0] },
			),
			(
				RuntimeError::Module(Balances(LiquidityRestrictions)),
				PopApiError::Module { index: crate::mock::Balances::index() as u8, error: [1, 0] },
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
	fn assert_runtime_error_works() {
		test_cases().into_iter().for_each(|t| {
			crate::errors::v0::assert_runtime_err!(
				Result::<(), pop_primitives::Error>::Err(t.1),
				t.0,
			);
		});
	}
}
