//! A set of errors used for testing smart contracts.

use std::fmt::Debug;

pub use drink::{
	pallet_assets::Error as AssetsError, pallet_balances::Error as BalancesError,
	pallet_contracts::Error as ContractsError,
};
pub use pop_api::primitives::v0::*;
use scale::{Decode, Encode};

use crate::utils::decode;

/// Configuration for the runtime error.
pub trait RuntimeErrorConfig: Debug {
	type ModuleError: Decode + Encode + Debug;
	type PopApiError: Decode + Encode + Debug + From<u32> + Into<u32>;
	const MODULE_INDEX: u8;
}

/// Runtime error type for testing Pop API V0.
#[derive(Encode, Decode, Debug)]
pub enum RuntimeError<T: RuntimeErrorConfig> {
	Module(T::ModuleError),
	Raw(T::PopApiError),
}

impl<T: RuntimeErrorConfig> From<RuntimeError<T>> for u32 {
	/// Converts a `RuntimeError` into a numerical value of `pop_api::primitives::v0::Error`.
	///
	/// This conversion is necessary for comparing `RuntimeError` instances with other types
	/// that implement `Into<u32>`, as `RuntimeError` does not implement `Eq`.
	/// Use this function to obtain a numerical representation of the error for comparison or
	/// further processing.
	fn from(error: RuntimeError<T>) -> Self {
		let pop_api_error = match error {
			RuntimeError::Module(error) => {
				let mut encoded = error.encode();
				encoded.insert(0, T::MODULE_INDEX);
				encoded.resize(4, 0);
				decode::<T::PopApiError>(&encoded)
			},
			RuntimeError::Raw(error) => decode::<T::PopApiError>(&error.encode()),
		};
		pop_api_error.into()
	}
}

impl<T: RuntimeErrorConfig> From<u32> for RuntimeError<T> {
	/// Converts a numerical value of `pop_api::primitives::v0::Error` into a `RuntimeError`.
	///
	/// This is used to reconstruct and display a `RuntimeError` from its numerical representation
	/// when an error is thrown.
	fn from(value: u32) -> Self {
		let error = T::PopApiError::from(value);
		let encoded = error.encode();
		if encoded[0] == T::MODULE_INDEX {
			let (index, module_error) = (encoded[1], &encoded[2..]);
			let data = vec![vec![index], module_error.to_vec()].concat();
			return RuntimeError::Module(decode(&data));
		}
		RuntimeError::Raw(error)
	}
}

pub mod v0 {
	use std::fmt::Debug;
	pub fn assert_runtime_err_inner<R, E, RuntimeError>(
		result: Result<R, E>,
		expected_error: RuntimeError,
	) where
		// E: Returned `Err()` value of a method result. Must be convertable to `u32`.
		E: Into<u32>,
		// D: Expected `RuntimeError`.
		RuntimeError: From<u32> + Into<u32> + Debug,
	{
		crate::error::assert_runtime_err_inner::<pop_primitives::Error, R, E, RuntimeError>(
			result,
			expected_error,
		)
	}

	/// A utility macro to assert that an error returned from a smart contract method using
	/// `pop_api::primitives::v0::Error` matches the `RuntimeError`.
	#[macro_export]
	macro_rules! __assert_runtime_err {
		($result:expr, $error:expr $(,)?) => {
			$crate::error::v0::assert_runtime_err_inner::<_, _, _>($result, $error);
		};
	}

	pub use __assert_runtime_err as assert_runtime_err;
}

/// * 'R': Type returned if Ok()
/// * 'V': Version of Pop API.
/// * 'E': Returned Err() value of a method result. Must be convertable to `u32`.
/// * 'D': Expected RuntimeError.
#[track_caller]
pub fn assert_runtime_err_inner<VersionedApiError, R, E, RuntimeError>(
	result: Result<R, E>,
	expected_error: RuntimeError,
) where
	VersionedApiError: Into<u32>,
	E: Into<u32>,
	RuntimeError: From<u32> + Into<u32> + Debug,
{
	let expected_code: u32 = expected_error.into();
	let expected_error = RuntimeError::from(expected_code);
	if let Err(error) = result {
		let error_code: u32 = error.into();
		if error_code != expected_code {
			panic!(
				r#"assertion `left == right` failed
  left: {:?}
 right: {:?}"#,
				RuntimeError::from(error_code),
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
	use pop_primitives::v0::Error as PopApiError;

	use crate::error::{AssetsError::*, BalancesError::*, *};

	#[derive(Debug)]
	struct Config;

	impl RuntimeErrorConfig for Config {
		type ModuleError = crate::mock::RuntimeError;
		type PopApiError = PopApiError;

		const MODULE_INDEX: u8 = 3;
	}

	fn test_cases() -> Vec<(RuntimeError<Config>, PopApiError)> {
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
			crate::error::v0::assert_runtime_err!(
				Result::<(), pop_primitives::Error>::Err(t.1),
				t.0,
			);
		});
	}
}