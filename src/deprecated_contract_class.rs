use cairo_lang_casm_contract_class::CasmContractEntryPoint;
use serde::de::Error as DeserializationError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::api_core::EntryPointSelector;
use crate::serde_utils::deserialize_optional_contract_class_abi_entry_vector;
use crate::stdlib::collections::HashMap;
use crate::stdlib::string::String;
use crate::stdlib::vec::Vec;
use crate::StarknetApiError;

/// A deprecated contract class.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct ContractClass {
    // Starknet does not verify the abi. If we can't parse it, we set it to None.
    #[serde(default, deserialize_with = "deserialize_optional_contract_class_abi_entry_vector")]
    pub abi: Option<Vec<ContractClassAbiEntry>>,
    pub program: Program,
    /// The selector of each entry point is a unique identifier in the program.
    // TODO: Consider changing to IndexMap, since this is used for computing the
    // class hash.
    pub entry_points_by_type: HashMap<EntryPointType, Vec<EntryPoint>>,
}

/// A [ContractClass](`crate::deprecated_contract_class::ContractClass`) abi entry.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
pub enum ContractClassAbiEntry {
    /// An event abi entry.
    Event(EventAbiEntry),
    /// A function abi entry.
    Function(FunctionAbiEntryWithType),
    /// A struct abi entry.
    Struct(StructAbiEntry),
}

/// An event abi entry.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct EventAbiEntry {
    pub name: String,
    pub keys: Vec<TypedParameter>,
    pub data: Vec<TypedParameter>,
}

/// A function abi entry with type.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct FunctionAbiEntryWithType {
    pub r#type: FunctionAbiEntryType,
    #[serde(flatten)]
    pub entry: FunctionAbiEntry,
}

/// A function abi entry type.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum FunctionAbiEntryType {
    #[serde(rename = "constructor")]
    Constructor,
    #[serde(rename = "l1_handler")]
    L1Handler,
    #[serde(rename = "function")]
    #[default]
    Function,
}

/// A function abi entry.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct FunctionAbiEntry {
    pub name: String,
    pub inputs: Vec<TypedParameter>,
    pub outputs: Vec<TypedParameter>,
}

/// A struct abi entry.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct StructAbiEntry {
    pub name: String,
    pub size: usize,
    pub members: Vec<StructMember>,
}

/// A struct member for [StructAbiEntry](`crate::deprecated_contract_class::StructAbiEntry`).
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct StructMember {
    #[serde(flatten)]
    pub param: TypedParameter,
    pub offset: usize,
}

/// A program corresponding to a [ContractClass](`crate::deprecated_contract_class::ContractClass`).
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Program {
    #[serde(default)]
    pub attributes: serde_json::Value,
    pub builtins: serde_json::Value,
    #[serde(default)]
    pub compiler_version: serde_json::Value,
    pub data: serde_json::Value,
    pub debug_info: serde_json::Value,
    pub hints: serde_json::Value,
    pub identifiers: serde_json::Value,
    pub main_scope: serde_json::Value,
    pub prime: serde_json::Value,
    pub reference_manager: serde_json::Value,
}

/// An entry point type of a [ContractClass](`crate::deprecated_contract_class::ContractClass`).
#[derive(
    Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "scale-info", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub enum EntryPointType {
    /// A constructor entry point.
    #[serde(rename = "CONSTRUCTOR")]
    Constructor,
    /// An external4 entry point.
    #[serde(rename = "EXTERNAL")]
    #[default]
    External,
    /// An L1 handler entry point.
    #[serde(rename = "L1_HANDLER")]
    L1Handler,
}

/// An entry point of a [ContractClass](`crate::deprecated_contract_class::ContractClass`).
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
#[cfg_attr(feature = "scale-info", derive(parity_scale_codec::Encode, parity_scale_codec::Decode))]
pub struct EntryPoint {
    pub selector: EntryPointSelector,
    pub offset: EntryPointOffset,
}

impl TryFrom<CasmContractEntryPoint> for EntryPoint {
    type Error = StarknetApiError;

    fn try_from(value: CasmContractEntryPoint) -> Result<Self, Self::Error> {
        Ok(EntryPoint {
            selector: EntryPointSelector(value.selector.to_str_radix(16).as_str().try_into()?),
            offset: EntryPointOffset(value.offset),
        })
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct TypedParameter {
    pub name: String,
    pub r#type: String,
}

/// The offset of an [EntryPoint](`crate::deprecated_contract_class::EntryPoint`).
#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
pub struct EntryPointOffset(#[serde(deserialize_with = "number_or_string")] pub usize);
impl TryFrom<String> for EntryPointOffset {
    type Error = StarknetApiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(hex_string_try_into_usize(&value)?))
    }
}

pub fn number_or_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<usize, D::Error> {
    let usize_value = match Value::deserialize(deserializer)? {
        Value::Number(number) => {
            number.as_u64().ok_or(DeserializationError::custom("Cannot cast number to usize."))?
                as usize
        }
        Value::String(s) => hex_string_try_into_usize(&s).map_err(DeserializationError::custom)?,
        _ => return Err(DeserializationError::custom("Cannot cast value into usize.")),
    };
    Ok(usize_value)
}

fn hex_string_try_into_usize(hex_string: &str) -> Result<usize, crate::stdlib::num::ParseIntError> {
    usize::from_str_radix(hex_string.trim_start_matches("0x"), 16)
}

#[cfg(feature = "scale-info")]
impl parity_scale_codec::Encode for EntryPointOffset {
    fn encode(&self) -> Vec<u8> {
        (self.0 as u64).encode()
    }
}

#[cfg(feature = "scale-info")]
impl parity_scale_codec::Decode for EntryPointOffset {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        Ok(Self(<u64>::decode(input)? as usize))
    }
}
