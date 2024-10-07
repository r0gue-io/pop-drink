use std::fmt::Debug;

pub use drink::{
	pallet_assets::Error as AssetsError, pallet_balances::Error as BalancesError,
	pallet_contracts::Error as ContractsError,
};
pub use frame_support::sp_runtime::{
	ArithmeticError, DispatchError, ModuleError, TokenError, TransactionalError,
};
use scale::{Decode, Encode};

fn decode<T: Decode>(data: &[u8]) -> T {
	T::decode(&mut &data[..]).expect("Decoding failed")
}

#[derive(Encode, Decode, Debug)]
pub enum DrinkError<E>
where
	E: Decode + Encode + Debug,
{
	Raw(DispatchError),
	Module(E),
}

impl<E> From<DrinkError<E>> for u32
where
	E: Decode + Encode + Debug,
{
	fn from(error: DrinkError<E>) -> Self {
		let mut value = match error {
			DrinkError::Raw(dispatch_error) => dispatch_error.encode(),
			DrinkError::Module(module_error) => {
				let mut module_error = module_error.encode();
				module_error.insert(0, 3);
				module_error
			},
		};
		value.resize(4, 0);
		u32::from_le_bytes(value.try_into().expect("qed, resized to 4 bytes line above"))
	}
}

impl<E> From<u32> for DrinkError<E>
where
	E: Decode + Encode + Debug,
{
	fn from(value: u32) -> Self {
		let encoded = value.to_le_bytes();
		match encoded {
			[3, module_error @ ..] => DrinkError::Module(decode(&module_error)),
			_ => DrinkError::Raw(decode(&encoded)),
		}
	}
}

#[track_caller]
pub fn assert_runtime_err_inner<T, R, E>(result: Result<R, E>, expected_error: DrinkError<T>)
where
	T: Decode + Encode + Debug,
	E: Into<u32>,
{
	let expected_code: u32 = expected_error.into();
	let expected_error = DrinkError::<T>::from(expected_code);
	if let Err(error) = result {
		let error_code: u32 = error.into();
		if error_code != expected_code {
			panic!(
				r#"assertion `left == right` failed
  left: {:?}
 right: {:?}"#,
				DrinkError::<T>::from(error_code),
				expected_error,
			);
		}
	} else {
		panic!(
			r#"assertion `left == right` failed
  left: Ok()
 right: {:?}"#,
			expected_error,
		);
	}
}

#[macro_export]
macro_rules! assert_runtime_err {
	($result:expr, $error:expr $(,)?) => {
		$crate::error::assert_runtime_err_inner($result, $error);
	};
}

#[cfg(test)]
mod test {
	use pop_api::primitives::Error as PopApiError;

	use super::{
		AssetsError::*,
		BalancesError::*,
		DrinkError::{self, *},
	};

	fn test_cases() -> Vec<(DrinkError<crate::mock::RuntimeError>, PopApiError)> {
		use frame_support::traits::PalletInfoAccess;

		use crate::mock::RuntimeError::*;
		vec![
			(
				Raw(frame_support::sp_runtime::ArithmeticError::Overflow.into()),
				PopApiError::Arithmetic(pop_api::primitives::ArithmeticError::Overflow),
			),
			(
				Module(Assets(BalanceLow)),
				PopApiError::Module { index: crate::mock::Assets::index() as u8, error: [0, 0] },
			),
			(
				Module(Assets(NoAccount)),
				PopApiError::Module { index: crate::mock::Assets::index() as u8, error: [1, 0] },
			),
			(
				Module(Assets(NoPermission)),
				PopApiError::Module { index: crate::mock::Assets::index() as u8, error: [2, 0] },
			),
			(
				Module(Balances(VestingBalance)),
				PopApiError::Module { index: crate::mock::Balances::index() as u8, error: [0, 0] },
			),
			(
				Module(Balances(LiquidityRestrictions)),
				PopApiError::Module { index: crate::mock::Balances::index() as u8, error: [1, 0] },
			),
			(
				Module(Balances(InsufficientBalance)),
				PopApiError::Module { index: crate::mock::Balances::index() as u8, error: [2, 0] },
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
		use crate::error::assert_runtime_err_inner;
		test_cases().into_iter().for_each(|t| {
			assert_runtime_err_inner(Result::<(), pop_primitives::Error>::Err(t.1), t.0);
		});
	}
}
