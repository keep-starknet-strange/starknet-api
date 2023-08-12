//! Representations of canonical [`Starknet`] components.
//!
//! [`Starknet`]: https://starknet.io/

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!("./with_std.rs");

#[cfg(not(feature = "std"))]
include!("./without_std.rs");

pub mod stdlib {
    #[cfg(feature = "std")]
    pub use crate::with_std::*;
    #[cfg(not(feature = "std"))]
    pub use crate::without_std::*;
}

pub mod api_core;
pub mod block;
pub mod deprecated_contract_class;
pub mod hash;
pub mod serde_utils;
pub mod state;
pub mod transaction;
pub mod type_utils;

use serde_utils::InnerDeserializationError;

use crate::stdlib::num::ParseIntError;
use crate::stdlib::string::String;

/// The error type returned by StarknetApi.
#[derive(Clone, Debug)]
pub enum StarknetApiError {
    /// Error in the inner deserialization of the node.
    InnerDeserialization(InnerDeserializationError),
    /// An error for when a value is out of range.
    OutOfRange { string: String },
    /// Error when serializing into number.
    ParseIntError(ParseIntError),
}

#[cfg(feature = "std")]
impl std::error::Error for StarknetApiError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StarknetApiError::InnerDeserialization(e) => e.source(),
            StarknetApiError::OutOfRange { .. } => std::option::Option::None,
            StarknetApiError::ParseIntError(e) => e.source(),
        }
    }
}

impl core::fmt::Display for StarknetApiError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StarknetApiError::InnerDeserialization(e) => e.fmt(f),
            StarknetApiError::OutOfRange { string } => {
                write!(f, "Out of range {string}.")
            }
            StarknetApiError::ParseIntError(e) => e.fmt(f),
        }
    }
}
impl core::convert::From<InnerDeserializationError> for StarknetApiError {
    fn from(source: InnerDeserializationError) -> Self {
        StarknetApiError::InnerDeserialization(source)
    }
}

impl core::convert::From<ParseIntError> for StarknetApiError {
    fn from(source: ParseIntError) -> Self {
        StarknetApiError::ParseIntError(source)
    }
}
