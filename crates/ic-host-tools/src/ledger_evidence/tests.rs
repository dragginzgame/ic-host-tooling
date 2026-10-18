use super::*;
use std::collections::BTreeMap;

const LIMITS: BlockEvidenceLimits = BlockEvidenceLimits {
    blocks: 8,
    nodes: 128,
    bytes: 4096,
    depth: 8,
};

fn chain() -> Vec<LedgerBlock> {
    let root = Value::Map(BTreeMap::new());
    let child = Value::Map(BTreeMap::from([(
        "phash".into(),
        Value::Blob(root.hash().to_vec().into()),
    )]));
    vec![
        LedgerBlock {
            id: Nat::from(1_u8),
            block: child,
        },
        LedgerBlock {
            id: Nat::from(0_u8),
            block: root,
        },
    ]
}

#[test]
fn exact_contiguous_chain_returns_the_target_block() {
    let blocks = chain();
    let tip = LedgerTip {
        index: 1,
        hash: blocks[0].block.hash(),
    };
    let bounded = BoundedLedgerBlocks::admit(&blocks, LIMITS).unwrap();
    assert_eq!(bounded.verify_chain(&tip, 0).unwrap(), &blocks[1].block);
    assert_eq!(
        bounded.verify_chain(&tip, 1),
        Err(BlockEvidenceError::ChainMismatch)
    );
    let wrong_tip = LedgerTip {
        index: 1,
        hash: [0; 32],
    };
    assert_eq!(
        bounded.verify_chain(&wrong_tip, 0),
        Err(BlockEvidenceError::ChainMismatch)
    );

    for corrupt in 0..3 {
        let mut blocks = chain();
        match corrupt {
            0 => {
                let Value::Map(fields) = &mut blocks[0].block else {
                    unreachable!()
                };
                fields.insert("phash".into(), Value::Blob(vec![0; 32].into()));
            }
            1 => blocks[1].id = Nat::from(1_u8),
            _ => blocks.swap(0, 1),
        }
        let tip = LedgerTip {
            index: 1,
            hash: blocks[0].block.hash(),
        };
        assert_eq!(
            BoundedLedgerBlocks::admit(&blocks, LIMITS)
                .unwrap()
                .verify_chain(&tip, 0),
            Err(BlockEvidenceError::ChainMismatch)
        );
    }
    let blocks = [LedgerBlock {
        id: Nat::from(0_u8),
        block: Value::Map(BTreeMap::from([(
            "phash".into(),
            Value::Blob(vec![0; 32].into()),
        )])),
    }];
    let tip = LedgerTip {
        index: 0,
        hash: blocks[0].block.hash(),
    };
    assert_eq!(
        BoundedLedgerBlocks::admit(&blocks, LIMITS)
            .unwrap()
            .verify_chain(&tip, 0),
        Err(BlockEvidenceError::ChainMismatch)
    );
}

#[test]
fn all_work_budgets_refuse_before_hashing() {
    let blocks = chain();
    for limits in [
        BlockEvidenceLimits {
            blocks: 1,
            ..LIMITS
        },
        BlockEvidenceLimits { nodes: 1, ..LIMITS },
        BlockEvidenceLimits { bytes: 1, ..LIMITS },
        BlockEvidenceLimits { depth: 0, ..LIMITS },
    ] {
        assert!(matches!(
            BoundedLedgerBlocks::admit(&blocks, limits),
            Err(BlockEvidenceError::BudgetExceeded)
        ));
    }
}

#[test]
fn signed_value_overflow_and_excessive_indexes_refuse_before_hashing() {
    let blocks = [LedgerBlock {
        id: Nat::from(0_u8),
        block: Value::Int(ic_platform::candid::contract::Int::from(1_u128 << 127)),
    }];
    assert!(matches!(
        BoundedLedgerBlocks::admit(&blocks, LIMITS),
        Err(BlockEvidenceError::UnsupportedValue)
    ));
    let mut index = Nat::from(u128::MAX);
    index.0 += 1_u8;
    let blocks = [LedgerBlock {
        id: index,
        block: Value::Map(BTreeMap::new()),
    }];
    assert!(matches!(
        BoundedLedgerBlocks::admit(&blocks, LIMITS),
        Err(BlockEvidenceError::ChainMismatch)
    ));
}
