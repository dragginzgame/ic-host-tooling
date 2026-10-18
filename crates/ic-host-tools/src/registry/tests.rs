use super::{
    proto::*,
    routing_shards::{canister_range_start_from_key, validate_routing_table_shard_bounds},
};
use ic_platform::Principal;
use prost::Message;

fn canister(value: u64) -> CanisterId {
    let mut raw = value.to_be_bytes().to_vec();
    raw.extend([1, 1]);
    CanisterId {
        principal_id: Some(PrincipalId { raw }),
    }
}

fn shard(start: u64, end: u64) -> RoutingTable {
    RoutingTable {
        entries: vec![RoutingTableEntry {
            range: Some(CanisterIdRange {
                start_canister_id: Some(canister(start)),
                end_canister_id: Some(canister(end)),
            }),
            subnet_id: None,
        }],
    }
}

#[test]
fn wire_tags_preserve_version_wrappers_partial_records_and_unknown_fields() {
    let request = RegistryGetValueRequest {
        version: Some(UInt64Value { value: 7 }),
        key: b"k".to_vec(),
    };
    let bytes = [10, 2, 8, 7, 18, 1, b'k'];
    assert_eq!(request.encode_to_vec(), bytes);
    assert_eq!(
        RegistryGetValueRequest::decode(bytes.as_slice()).unwrap(),
        request
    );
    // Node operator bytes use field 15; unrelated field 1 remains ignored.
    let node = NodeRecord::decode([10, 1, 99, 122, 2, 1, 2].as_slice()).unwrap();
    assert_eq!(node.node_operator_id, [1, 2]);
    assert_eq!(node.encode_to_vec(), [122, 2, 1, 2]);
    assert!(RegistryGetValueRequest::decode([18, 2, 1].as_slice()).is_err());
    assert_eq!(RegistryErrorCode::InternalError as i32, 999);
    assert_eq!(RegistryMutationType::Upsert as i32, 4);
    assert_eq!(SubnetType::CloudEngine as i32, 5);
    assert_eq!(CanisterCyclesCostSchedule::Free as i32, 2);
}

#[test]
fn certified_tree_and_large_value_alternatives_preserve_golden_tags() {
    let response = RegistryCertifiedResponse {
        hash_tree: Some(RegistryMixedHashTree {
            tree: Some(registry_mixed_hash_tree::Tree::LeafData(vec![42])),
        }),
        certificate: vec![7],
    };
    let bytes = [10, 3, 34, 1, 42, 18, 1, 7];
    assert_eq!(response.encode_to_vec(), bytes);
    assert_eq!(
        RegistryCertifiedResponse::decode(bytes.as_slice()).unwrap(),
        response
    );
    let mutation = HighCapacityRegistryMutation {
        mutation_type: RegistryMutationType::Upsert as i32,
        key: b"k".to_vec(),
        content: Some(high_capacity_registry_mutation::Content::Value(vec![8])),
    };
    let bytes = [8, 4, 18, 1, b'k', 26, 1, 8];
    assert_eq!(mutation.encode_to_vec(), bytes);
    assert_eq!(
        HighCapacityRegistryMutation::decode(bytes.as_slice()).unwrap(),
        mutation
    );
}

#[test]
fn keys_require_canonical_routable_ids_and_shards_respect_interval_bounds() {
    let lower = canister_range_start_from_key("ranges_", "ranges_00000000000000000101").unwrap();
    let next = Principal::from_slice(&canister(10).principal_id.unwrap().raw);
    // A shard need not start exactly at its key's lower bound.
    validate_routing_table_shard_bounds("key", &lower, &shard(2, 9), Some(&next)).unwrap();
    for table in [
        shard(9, 2),
        shard(2, 10),
        RoutingTable { entries: vec![] },
        RoutingTable {
            entries: vec![RoutingTableEntry {
                range: None,
                subnet_id: None,
            }],
        },
    ] {
        assert!(validate_routing_table_shard_bounds("key", &lower, &table, Some(&next)).is_err());
    }
    for key in [
        "ranges_",
        "ranges_0",
        "ranges_00000000000000000A01",
        "ranges_00000000000000000102",
        "other_00000000000000000101",
    ] {
        assert!(canister_range_start_from_key("ranges_", key).is_err());
    }
    assert!(
        canister_range_start_from_key("ranges_", &format!("ranges_{}", "0".repeat(100_000)))
            .is_err()
    );
}
