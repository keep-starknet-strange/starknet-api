use indexmap::IndexMap;
use serde_json::json;

use crate::deprecated_contract_class::EntryPointOffset;

#[test]
fn entry_point_offset_from_json_str() {
    let data = r#"
        {
            "offset_1":  2,
            "offset_2": "0x7b"
        }"#;
    let offsets: IndexMap<String, EntryPointOffset> = serde_json::from_str(data).unwrap();

    assert_eq!(EntryPointOffset(2), offsets["offset_1"]);
    assert_eq!(EntryPointOffset(123), offsets["offset_2"]);
}

#[test]
fn entry_point_offset_into_json_str() {
    let offset = EntryPointOffset(123);
    assert_eq!(json!(offset), json!(format!("{:#x}", offset.0)));
}

#[cfg(test)]
mod thin_state_diff_tests {
    use parity_scale_codec::{Decode, Encode};

    use super::*;
    use crate::state::{
        ClassHash, CompiledClassHash, ContractAddress, Nonce, StarkFelt, StorageKey, ThinStateDiff,
    };

    #[test]
    fn encode_decode_works() {
        let mut deployed_contracts = IndexMap::default();
        deployed_contracts.insert(ContractAddress::from(1_u32), ClassHash::default());
        deployed_contracts.insert(ContractAddress::from(3_u32), ClassHash::default());

        let mut declared_classes = IndexMap::default();
        declared_classes.insert(ClassHash::default(), CompiledClassHash::default());
        declared_classes.insert(ClassHash::default(), CompiledClassHash::default());

        let mut storage_diffs = IndexMap::default();
        let mut storage_updates_1 = IndexMap::default();
        storage_updates_1.insert(StorageKey::from(9_u32), StarkFelt::from(1_u32));
        storage_updates_1.insert(StorageKey::from(11_u32), StarkFelt::from(12_u32));
        storage_diffs.insert(ContractAddress::from(13_u32), storage_updates_1);
        let mut storage_updates_2 = IndexMap::default();
        storage_updates_2.insert(StorageKey::from(14_u32), StarkFelt::from(15_u32));
        storage_updates_2.insert(StorageKey::from(16_u32), StarkFelt::from(17_u32));
        storage_diffs.insert(ContractAddress::from(18_u32), storage_updates_2);

        let mut nonces = IndexMap::default();
        nonces.insert(ContractAddress::from(5_u32), Nonce::default());
        nonces.insert(ContractAddress::from(7_u32), Nonce::default());

        let mut replaced_classes = IndexMap::default();
        replaced_classes.insert(ContractAddress::from(19_u32), ClassHash::default());
        replaced_classes.insert(ContractAddress::from(21_u32), ClassHash::default());

        let deprecated_declared_classes = vec![ClassHash::default()];
        
        let state_diff = ThinStateDiff {
            deployed_contracts,
            storage_diffs,
            declared_classes,
            deprecated_declared_classes,
            nonces,
            replaced_classes,
        };

        let encoded = state_diff.encode();

        let decoded = ThinStateDiff::decode(&mut &encoded[..]).unwrap();

        assert_eq!(state_diff, decoded);
    }
}
