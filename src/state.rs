use std::fmt::Debug;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::block::{BlockHash, BlockNumber};
use crate::core::{
    ClassHash, CompiledClassHash, ContractAddress, EntryPointSelector, GlobalRoot, Nonce,
    PatriciaKey,
};
use crate::deprecated_contract_class::ContractClass as DeprecatedContractClass;
use crate::hash::{StarkFelt, StarkHash};
use crate::{impl_from_through_intermediate, StarknetApiError};

pub type DeclaredClasses = IndexMap<ClassHash, ContractClass>;
pub type DeprecatedDeclaredClasses = IndexMap<ClassHash, DeprecatedContractClass>;

/// The differences between two states before and after a block with hash block_hash
/// and their respective roots.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct StateUpdate {
    pub block_hash: BlockHash,
    pub new_root: GlobalRoot,
    pub old_root: GlobalRoot,
    pub state_diff: StateDiff,
}

/// The differences between two states.
// Invariant: Addresses are strictly increasing.
// Invariant: Class hashes of declared_classes and deprecated_declared_classes are exclusive.
// TODO(yair): Enforce this invariant.
#[derive(Debug, Default, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct StateDiff {
    pub deployed_contracts: IndexMap<ContractAddress, ClassHash>,
    pub storage_diffs: IndexMap<ContractAddress, IndexMap<StorageKey, StarkFelt>>,
    pub declared_classes: IndexMap<ClassHash, (CompiledClassHash, ContractClass)>,
    pub deprecated_declared_classes: IndexMap<ClassHash, DeprecatedContractClass>,
    pub nonces: IndexMap<ContractAddress, Nonce>,
    pub replaced_classes: IndexMap<ContractAddress, ClassHash>,
}

#[cfg(feature = "scale-info")]
impl scale_info::TypeInfo for StateDiff {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("StateDiff", module_path!()))
            .composite(
                scale_info::build::Fields::named()
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, ClassHash)>>()
                            .name("deployed_contracts")
                            .type_name("Vec<(ContractAddress, ClassHash)>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, Vec<(StorageKey, StarkFelt)>)>>()
                            .name("storage_diffs")
                            .type_name("Vec<(ContractAddress, Vec<(StorageKey, StarkFelt)>)>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ClassHash, (CompiledClassHash, ContractClass))>>()
                            .name("declared_classes")
                            .type_name("Vec<(ClassHash, (CompiledClassHash, ContractClass))>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ClassHash, DeprecatedContractClass)>>()
                            .name("deprecated_declared_classes")
                            .type_name("Vec<(ClassHash, DeprecatedContractClass)>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, Nonce)>>()
                            .name("nonces")
                            .type_name("Vec<(ContractAddress, Nonce)>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, ClassHash)>>()
                            .name("replaced_classes")
                            .type_name("Vec<(ContractAddress, ClassHash)>")
                    }),
            )
    }
}

// TODO find a smarter way than using JSON
// Start refactoring with `Program` struct and then `DeprecatedContractClass`
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for StateDiff {
    fn encode(&self) -> Vec<u8> {
        let json_repr: String = serde_json::json!(self).to_string();
        json_repr.encode()
    }
}

// TODO find a smarter way than using JSON
// Start refactoring with `Program` struct and then `DeprecatedContractClass`
#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for StateDiff {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let json_repr = <String>::decode(input)?;
        serde_json::from_str(&json_repr).map_err(|_e| {
            parity_scale_codec::Error::from("serde_json deserialization error for ContractClass")
        })
    }
}

// Invariant: Addresses are strictly increasing.
// The invariant is enforced as [`ThinStateDiff`] is created only from [`starknet_api`][`StateDiff`]
// where the addresses are strictly increasing.
#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct ThinStateDiff {
    pub deployed_contracts: IndexMap<ContractAddress, ClassHash>,
    pub storage_diffs: IndexMap<ContractAddress, IndexMap<StorageKey, StarkFelt>>,
    pub declared_classes: IndexMap<ClassHash, CompiledClassHash>,
    pub deprecated_declared_classes: Vec<ClassHash>,
    pub nonces: IndexMap<ContractAddress, Nonce>,
    pub replaced_classes: IndexMap<ContractAddress, ClassHash>,
}

#[cfg(feature = "scale-info")]
impl scale_info::TypeInfo for ThinStateDiff {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("ThinStateDiff", module_path!()))
            .composite(
                scale_info::build::Fields::named()
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, ClassHash)>>()
                            .name("deployed_contracts")
                            .type_name("Vec<(ContractAddress, ClassHash)>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, Vec<(StorageKey, StarkFelt)>)>>()
                            .name("storage_diffs")
                            .type_name("Vec<(ContractAddress, Vec<(StorageKey, StarkFelt)>)>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ClassHash, CompiledClassHash)>>()
                            .name("declared_classes")
                            .type_name("Vec<(ClassHash, (CompiledClassHash, ContractClass))>")
                    })
                    .field(|f| {
                        f.ty::<Vec<ClassHash>>()
                            .name("deprecated_declared_classes")
                            .type_name("Vec<ClassHash>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, Nonce)>>()
                            .name("nonces")
                            .type_name("Vec<(ContractAddress, Nonce)>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(ContractAddress, ClassHash)>>()
                            .name("replaced_classes")
                            .type_name("Vec<(ContractAddress, ClassHash)>")
                    }),
            )
    }
}

impl ThinStateDiff {
    // Returns also the declared classes without cloning them.
    pub fn from_state_diff(diff: StateDiff) -> (Self, DeclaredClasses, DeprecatedDeclaredClasses) {
        (
            Self {
                deployed_contracts: diff.deployed_contracts,
                storage_diffs: diff.storage_diffs,
                declared_classes: diff
                    .declared_classes
                    .iter()
                    .map(|(class_hash, (compiled_hash, _class))| (*class_hash, *compiled_hash))
                    .collect(),
                deprecated_declared_classes: diff
                    .deprecated_declared_classes
                    .keys()
                    .copied()
                    .collect(),
                nonces: diff.nonces,
                replaced_classes: diff.replaced_classes,
            },
            diff.declared_classes
                .into_iter()
                .map(|(class_hash, (_compiled_class_hash, class))| (class_hash, class))
                .collect(),
            diff.deprecated_declared_classes,
        )
    }
}

impl From<StateDiff> for ThinStateDiff {
    fn from(diff: StateDiff) -> Self {
        Self::from_state_diff(diff).0
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for ThinStateDiff {
    fn size_hint(&self) -> usize {
        (6 + self.storage_diffs.len())
            + self.deployed_contracts.len()
                * (core::mem::size_of::<ContractAddress>() + core::mem::size_of::<ClassHash>())
            + self.nonces.len()
                * (core::mem::size_of::<ContractAddress>() + core::mem::size_of::<Nonce>())
            + self.declared_classes.len()
                * (core::mem::size_of::<ClassHash>() + core::mem::size_of::<CompiledClassHash>())
            + self.storage_diffs.len() * core::mem::size_of::<ContractAddress>()
            + self.deprecated_declared_classes.len() * core::mem::size_of::<ClassHash>()
            + self.replaced_classes.len()
                * (core::mem::size_of::<ContractAddress>() + core::mem::size_of::<ClassHash>())
    }

    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, dest: &mut T) {
        parity_scale_codec::Compact(self.deployed_contracts.len() as u64).encode_to(dest);
        self.deployed_contracts.iter().for_each(|v| v.encode_to(dest));
        parity_scale_codec::Compact(self.storage_diffs.len() as u64).encode_to(dest);
        self.storage_diffs.iter().for_each(|(address, idx_map)| {
            address.encode_to(dest);
            parity_scale_codec::Compact(idx_map.len() as u64).encode_to(dest);
            idx_map.iter().for_each(|v| v.encode_to(dest));
        });
        parity_scale_codec::Compact(self.declared_classes.len() as u64).encode_to(dest);
        self.declared_classes.iter().for_each(|v| v.encode_to(dest));
        parity_scale_codec::Compact(self.deprecated_declared_classes.len() as u64).encode_to(dest);
        self.deprecated_declared_classes.iter().for_each(|v| v.encode_to(dest));
        parity_scale_codec::Compact(self.nonces.len() as u64).encode_to(dest);
        self.nonces.iter().for_each(|v| v.encode_to(dest));
        parity_scale_codec::Compact(self.replaced_classes.len() as u64).encode_to(dest);
        self.replaced_classes.iter().for_each(|v| v.encode_to(dest));
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for ThinStateDiff {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let res = <(
            Vec<(ContractAddress, ClassHash)>,
            Vec<(ContractAddress, Vec<(StorageKey, StarkFelt)>)>,
            Vec<(ClassHash, CompiledClassHash)>,
            Vec<ClassHash>,
            Vec<(ContractAddress, Nonce)>,
            Vec<(ContractAddress, ClassHash)>,
        )>::decode(input)?;

        Ok(ThinStateDiff {
            deployed_contracts: res.0.into_iter().collect(),
            storage_diffs: res
                .1
                .into_iter()
                .map(|(address, v)| (address, v.into_iter().collect()))
                .collect(),
            declared_classes: res.2.into_iter().collect(),
            deprecated_declared_classes: res.3.into_iter().collect(),
            nonces: res.4.into_iter().collect(),
            replaced_classes: res.5.into_iter().collect(),
        })
    }
}

/// The sequential numbering of the states between blocks.
// Example:
// States: S0       S1       S2
// Blocks      B0->     B1->
#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct StateNumber(pub BlockNumber);

impl StateNumber {
    /// The state at the beginning of the block.
    pub fn right_before_block(block_number: BlockNumber) -> StateNumber {
        StateNumber(block_number)
    }

    /// The state at the end of the block.
    pub fn right_after_block(block_number: BlockNumber) -> StateNumber {
        StateNumber(block_number.next())
    }

    pub fn is_before(&self, block_number: BlockNumber) -> bool {
        self.0 <= block_number
    }

    pub fn is_after(&self, block_number: BlockNumber) -> bool {
        !self.is_before(block_number)
    }

    pub fn block_after(&self) -> BlockNumber {
        self.0
    }
}

/// A storage key in a contract.
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Hash,
    Deserialize,
    Serialize,
    PartialOrd,
    Ord,
    derive_more::Deref,
)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct StorageKey(pub PatriciaKey);

impl From<StorageKey> for StarkFelt {
    fn from(storage_key: StorageKey) -> StarkFelt {
        **storage_key
    }
}

impl TryFrom<StarkHash> for StorageKey {
    type Error = StarknetApiError;

    fn try_from(val: StarkHash) -> Result<Self, Self::Error> {
        Ok(Self(PatriciaKey::try_from(val)?))
    }
}

impl From<u128> for StorageKey {
    fn from(val: u128) -> Self {
        StorageKey(PatriciaKey::from(val))
    }
}

impl_from_through_intermediate!(u128, StorageKey, u8, u16, u32, u64);

/// A contract class.
#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct ContractClass {
    pub sierra_program: Vec<StarkFelt>,
    pub entry_points_by_type: IndexMap<EntryPointType, Vec<EntryPoint>>,
    pub abi: String,
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
                        f.ty::<Vec<StarkFelt>>().name("sierra_program").type_name("Vec<StarkFelt>")
                    })
                    .field(|f| {
                        f.ty::<Vec<(EntryPointType, Vec<EntryPoint>)>>()
                            .name("entry_points_by_type")
                            .type_name("Vec<(EntryPointType, Vec<EntryPoint>)>")
                    })
                    .field(|f| f.ty::<String>().name("abi").type_name("String")),
            )
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Encode for ContractClass {
    fn size_hint(&self) -> usize {
        self.sierra_program.size_hint()
            + 1
            + self.entry_points_by_type.len() * core::mem::size_of::<EntryPointType>()
            + self.abi.size_hint()
    }

    fn encode_to<T: parity_scale_codec::Output + ?Sized>(&self, dest: &mut T) {
        self.sierra_program.encode_to(dest);
        parity_scale_codec::Compact(self.entry_points_by_type.len() as u32).encode_to(dest);
        self.entry_points_by_type.iter().for_each(|v| v.encode_to(dest));
        self.abi.encode_to(dest);
    }
}

#[cfg(feature = "parity-scale-codec")]
impl parity_scale_codec::Decode for ContractClass {
    fn decode<I: parity_scale_codec::Input>(
        input: &mut I,
    ) -> Result<Self, parity_scale_codec::Error> {
        let data =
            <(Vec<StarkFelt>, Vec<(EntryPointType, Vec<EntryPoint>)>, String)>::decode(input)?;

        Ok(ContractClass {
            sierra_program: data.0,
            entry_points_by_type: data.1.into_iter().collect(),
            abi: data.2,
        })
    }
}

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
    /// An external entry point.
    #[serde(rename = "EXTERNAL")]
    #[default]
    External,
    /// An L1 handler entry point.
    #[serde(rename = "L1_HANDLER")]
    L1Handler,
}

/// An entry point of a [ContractClass](`crate::state::ContractClass`).
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct EntryPoint {
    pub function_idx: FunctionIndex,
    pub selector: EntryPointSelector,
}

#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
#[cfg_attr(
    feature = "parity-scale-codec",
    derive(parity_scale_codec::Encode, parity_scale_codec::Decode)
)]
#[cfg_attr(feature = "scale-info", derive(scale_info::TypeInfo))]
pub struct FunctionIndex(pub u64);
