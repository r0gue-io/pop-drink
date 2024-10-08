//! A set of errors used for testing smart contracts.

use std::fmt::Debug;

pub use drink::{
	pallet_assets::Error as AssetsError, pallet_balances::Error as BalancesError,
	pallet_contracts::Error as ContractsError,
};

pub mod v0;

#[track_caller]
pub fn assert_runtime_err_inner<VersionedApiError, R, E, RuntimeError>(
	result: Result<R, E>,
	expected_error: RuntimeError,
) where
	// V: Version of Pop API.
	VersionedApiError: Into<u32>,
	// E: Returned Err() value of a method result. Must be convertable to `u32`.
	E: Into<u32>,
	// D: Expected RuntimeError.
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
