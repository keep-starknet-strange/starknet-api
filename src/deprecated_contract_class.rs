use cairo_lang_starknet_classes::casm_contract_class::CasmContractEntryPoint;
use indexmap::IndexMap;
use serde::de::Error as DeserializationError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::core::EntryPointSelector;
use crate::serde_utils::deserialize_optional_contract_class_abi_entry_vector;
use crate::StarknetApiError;

/// A deprecated contract class.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct ContractClass {
    // Starknet does not verify the abi. If we can't parse it, we set it to None.
    #[serde(default, deserialize_with = "deserialize_optional_contract_class_abi_entry_vector")]
    pub abi: Option<Vec<ContractClassAbiEntry>>,
    pub program: Program,
    /// The selector of each entry point is a unique identifier in the program.
    pub entry_points_by_type: IndexMap<EntryPointType, Vec<EntryPoint>>,
}

#[cfg(feature = "scale-info")]
impl scale_info::TypeInfo for ContractClass {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("ContractClass", module_path!()))
            .composite(
                scale_info::build::Fields::named()
                    .field(|f| {
                        f.ty::<Option<Vec<ContractClassAbiEntry>>>()
                            .name("abi")
                            .type_name("Option<Vec<ContractClassAbiEntry>>")
                    })
                    .field(|f| f.ty::<Program>().name("program").type_name("Program"))
                    .field(|f| {
                        f.ty::<Vec<(EntryPointType, Vec<EntryPoint>)>>()
                            .name("program")
                            .type_name("Vec<(EntryPointType, Vec<EntryPoint>)>")
                    }),
            )
    }
}

// TODO find a smarter way than using JSON
// Start refactoring with `Program` struct
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for ContractClass {
    fn encode(&self) -> Vec<u8> {
        let json_repr: String = serde_json::json!(self).to_string();
        json_repr.encode()
    }
}

// TODO find a smarter way than using JSON
// Start refactoring with `Program` struct
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for ContractClass {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let json_repr = <String>::decode(input)?;
        serde_json::from_str(&json_repr).map_err(|_e| {
            parity_scale_codec::Error::from("serde_json deserialization error for ContractClass")
        })
    }
}

/// A [ContractClass](`crate::deprecated_contract_class::ContractClass`) abi entry.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[serde(tag = "type")]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum ContractClassAbiEntry {
    #[serde(rename = "event")]
    Event(EventAbiEntry),
    #[serde(rename = "function")]
    Function(FunctionAbiEntry),
    #[serde(rename = "constructor")]
    Constructor(FunctionAbiEntry),
    #[serde(rename = "l1_handler")]
    L1Handler(FunctionAbiEntry),
    #[serde(rename = "struct")]
    Struct(StructAbiEntry),
}

/// An event abi entry.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct EventAbiEntry {
    pub name: String,
    pub keys: Vec<TypedParameter>,
    pub data: Vec<TypedParameter>,
}

/// A function abi entry.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct FunctionAbiEntry {
    pub name: String,
    pub inputs: Vec<TypedParameter>,
    pub outputs: Vec<TypedParameter>,
    #[serde(rename = "stateMutability", default, skip_serializing_if = "Option::is_none")]
    pub state_mutability: Option<FunctionStateMutability>,
}

/// A function state mutability.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub enum FunctionStateMutability {
    #[serde(rename = "view")]
    #[default]
    View,
}

/// A struct abi entry.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct StructAbiEntry {
    pub name: String,
    pub size: u64,
    pub members: Vec<StructMember>,
}

/// A struct member for [StructAbiEntry](`crate::deprecated_contract_class::StructAbiEntry`).
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct StructMember {
    #[serde(flatten)]
    pub param: TypedParameter,
    pub offset: u64,
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
    #[serde(default)]
    pub debug_info: serde_json::Value,
    pub hints: serde_json::Value,
    pub identifiers: serde_json::Value,
    pub main_scope: serde_json::Value,
    pub prime: serde_json::Value,
    pub reference_manager: serde_json::Value,
}

#[cfg(feature = "scale-info")]
impl scale_info::TypeInfo for Program {
    type Identity = String;

    fn type_info() -> scale_info::Type {
        Self::Identity::type_info()
    }
}

// TODO: Find out smarter way than `Program` -> Json -> SCALE
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for Program {
    fn encode(&self) -> Vec<u8> {
        let json_repr: String = serde_json::json!(self).to_string();
        json_repr.encode()
    }
}

// TODO: Find out smarter way than SCALE -> Json -> `Program`
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for Program {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let json_repr = <String>::decode(input)?;
        serde_json::from_str(&json_repr).map_err(|_e| {
            parity_scale_codec::Error::from("serde_json deserialization error for Program")
        })
    }
}

/// An entry point type of a [ContractClass](`crate::deprecated_contract_class::ContractClass`).
#[derive(
    Debug, Default, Clone, Copy, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
#[serde(deny_unknown_fields)]
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
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct EntryPoint {
    pub selector: EntryPointSelector,
    pub offset: EntryPointOffset,
}

impl TryFrom<CasmContractEntryPoint> for EntryPoint {
    type Error = StarknetApiError;

    fn try_from(value: CasmContractEntryPoint) -> Result<Self, Self::Error> {
        Ok(EntryPoint {
            selector: EntryPointSelector(value.selector.to_str_radix(16).as_str().try_into()?),
            // TODO get rid of casting
            offset: EntryPointOffset(value.offset as u64),
        })
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct TypedParameter {
    pub name: String,
    pub r#type: String,
}

/// The offset of an [EntryPoint](`crate::deprecated_contract_class::EntryPoint`).
#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct EntryPointOffset(
    #[serde(deserialize_with = "number_or_string", serialize_with = "u64_to_hex")] pub u64,
);
impl TryFrom<String> for EntryPointOffset {
    type Error = StarknetApiError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(hex_string_try_into_u64(&value)?))
    }
}

pub fn number_or_string<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    let value = match Value::deserialize(deserializer)? {
        Value::Number(number) => {
            number.as_u64().ok_or(DeserializationError::custom("Cannot cast number to usize."))?
        }
        Value::String(s) => hex_string_try_into_u64(&s).map_err(DeserializationError::custom)?,
        _ => return Err(DeserializationError::custom("Cannot cast value into usize.")),
    };
    Ok(value)
}

fn hex_string_try_into_u64(hex_string: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(hex_string.trim_start_matches("0x"), 16)
}

fn u64_to_hex<S>(value: &u64, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(format!("{:#x}", value).as_str())
}
