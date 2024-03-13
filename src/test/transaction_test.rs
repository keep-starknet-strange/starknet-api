#[cfg(all(test, feature = "parity-scale-codec"))]
mod execution_resources_psc_test {
    use std::collections::hash_map::RandomState as HasherBuilder;

    use indexmap::IndexMap;
    use parity_scale_codec::{Decode, Encode};

    use super::*;
    use crate::transaction::{Builtin, ExecutionResources};

    #[test]
    fn encode_decode_work() {
        let steps = 30u64;
        let memory_holes = 12u64;
        let builtin_instance_counter = IndexMap::<Builtin, u64, HasherBuilder>::from_iter(vec![
            (Builtin::RangeCheck, 0u64),
            (Builtin::Pedersen, 1u64),
            (Builtin::Poseidon, 2u64),
            (Builtin::EcOp, 3u64),
            (Builtin::Ecdsa, 4u64),
            (Builtin::Bitwise, 5u64),
            (Builtin::Keccak, 6u64),
            (Builtin::SegmentArena, 7u64),
        ]);

        let execution_resources =
            ExecutionResources { steps, builtin_instance_counter, memory_holes };

        let encoded = execution_resources.encode();
        let decoded = ExecutionResources::decode(&mut &encoded[..]).unwrap();

        assert_eq!(execution_resources, decoded);
    }
}
