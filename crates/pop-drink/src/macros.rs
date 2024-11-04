use std::fmt::Debug;

use drink::{session::Session, Sandbox};
use scale::{Decode, Encode};

use crate::last_contract_event;

/// Asserts that a result matches an expected `Error`.
///
/// This can be used to assert that a contract execution resulted in a specific runtime error
/// `Error`. The contract error must be convertible to a `u32` (i.e. the status code received from
/// the api).
///
/// # Example
///
/// ## Errors
///
/// ```rs
/// use drink::devnet::{
/// 	Assets,
/// 	AssetsError::BalanceLow,
/// 	v0::{
/// 		Arithmetic,
/// 		ArithmeticError::Overflow,
/// 		BadOrigin
/// 	},
/// };
/// ```
///
/// [`BadOrigin`](https://github.com/r0gue-io/pop-node/blob/main/primitives/src/lib.rs#L36C4-L36C18):
/// ```rs
/// Error::Raw(BadOrigin)
/// ```
///
/// [`Arithmetic(Overflow)`](https://github.com/r0gue-io/pop-node/blob/main/primitives/src/lib.rs#L55):
/// ```rs
/// Error::Raw(Arithmetic(Overflow))
/// ```
///
/// [`Assets(BalanceLow)`](https://paritytech.github.io/polkadot-sdk/master/pallet_assets/pallet/enum.Error.html#variant.BalanceLow):
/// ```rs
/// Error::Module(Assets(BalanceLow))
/// ```
///
/// ## How to use `assert_err` macro.
///
/// - Create a custom error type that holds the status code.
///
/// ```rs
/// use pop_api::StatusCode;
///
/// /// Custom error in contract.
/// pub enum CustomError {
/// 	...,
/// 	/// Error with status code.
/// 	StatusCode(u32),
/// }
///
/// impl From<StatusCode> for CustomError {
/// 	/// Converts a `StatusCode` (returned by the api) to a `CustomError`.
/// 	fn from(value: StatusCode) -> Self {
/// 		match value {
/// 			...,
/// 			_ => CustomError::StatusCode(value.0),
/// 		}
/// 	}
/// }
///
/// impl From<CustomError> for u32 {
/// 	/// Converts a `CustomError to a `u32`.
/// 	//
/// 	// Required for the `assert_err` macro to assert to `Error`.
/// 	fn from(value: CustomError) -> Self {
/// 		match value {
/// 			...,
/// 			CustomError::StatusCode(status_code) => status_code,
/// 		}
/// 	}
/// }
///
/// - Use `assert_err` in a test.
///
/// #[drink::test(sandbox = Pop)]
/// fn test_custom_error(mut session: Session) {
/// 	...
///
/// 	// Call a contract method that returns a `Result<(), CustomError>`.
/// 	let result = call::<Pop, (), CustomError>(session, "hello_world", vec![], None);
///
/// 	// Assert the result to the expected error.
/// 	assert_err!(result, Error::Raw(BadOrigin)));
///
/// 	// Other assertions:
/// 	...
/// 	assert_err!(result, Error::Raw(Arithmetic(Overflow)));
/// 	...
/// 	assert_err!(result, Error::Module(Assets(BalanceLow)));
/// }
/// ```
///
/// # Parameters:
/// - `result` - The result which contains the custom error type.
/// - `error` - The expected error.
#[macro_export]
macro_rules! assert_err {
	($result:expr, $error:expr $(,)?) => {
		$crate::macros::assert_err_inner::<_, _, _>($result, $error);
	};
}

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
			panic!("{}", assert_message(&Error::from(error_code), &expected_error));
		}
	} else {
		panic!("{}", assert_message(&"Ok()", &expected_error));
	}
}

/// Asserts that the latest event matches an expected `event`.
///
/// This can be used to assert that an event emitted from the latest contract execution resulted in
/// a specific event.
///
/// # Example
///
/// ```rs
/// assert_last_event!(
/// 	&session,
/// 	Transfer {
/// 		from: Some(account_id_from_slice(&contract)),
/// 		to: Some(account_id_from_slice(&BOB)),
/// 		value,
/// 	}
/// );
/// ```
///
/// # Parameters:
/// - `session` - The session for interacting with contracts.
/// - `event` - The expected event.
#[macro_export]
macro_rules! assert_last_event {
	($session:expr, $event:expr $(,)?) => {
		$crate::macros::assert_last_event_inner::<_, _>($session, $event);
	};
}

#[track_caller]
pub fn assert_last_event_inner<S, E>(session: &Session<S>, event: E)
where
	S: Sandbox,
	S::Runtime: pallet_contracts::Config,
	<S::Runtime as frame_system::Config>::RuntimeEvent:
		TryInto<pallet_contracts::Event<S::Runtime>>,
	E: Decode + Encode + Debug,
{
	match last_contract_event(session) {
		Some(last_event) =>
			if last_event != event.encode().as_slice() {
				let decoded = E::decode(&mut &last_event[..]).expect("Decoding failed");
				panic!("{}", assert_message(&decoded, &event));
			},
		None => panic!("{}", assert_message(&"None", &event)),
	}
}

fn assert_message<L: Debug, R: Debug>(left: &L, right: &R) -> String {
	format!(
		r#"assertion `left == right` failed
  left: {:?}
 right: {:?}"#,
		left, right
	)
}
