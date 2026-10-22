use super::*;
use ic_host_artifacts::wasm::validate;

const LIMITS: RawReplyLimits = RawReplyLimits {
    methods: 3,
    reply_bytes: 70_000,
    method_name_bytes: 100,
};

#[test]
fn validates_stateful_slots_and_opaque_page_spanning_replies() {
    let large = vec![0xff; 65_537];
    let methods = [
        RawReplyMethod {
            name: "read\"\\é",
            mode: ReplyMode::Update,
            reply: &large,
        },
        RawReplyMethod {
            name: "read\"\\é",
            mode: ReplyMode::Update,
            reply: b"second\0",
        },
        RawReplyMethod {
            name: "other",
            mode: ReplyMode::Update,
            reply: &[],
        },
    ];
    let wasm = module(
        &methods,
        LIMITS,
        Some(("https://ic-testkit.invalid/test/", 50_000_000_000)),
        None,
        100_000,
    )
    .unwrap();
    validate(&wasm, 100_000).unwrap();
    assert!(wasm.windows(large.len()).any(|bytes| bytes == large));
    validate(&module(&[], LIMITS, None, None, 10_000).unwrap(), 10_000).unwrap();
}

#[test]
fn refuses_catalog_and_compiled_output_limits() {
    let method = RawReplyMethod {
        name: "read",
        mode: ReplyMode::Update,
        reply: b"raw",
    };
    for limits in [
        RawReplyLimits {
            methods: 0,
            ..LIMITS
        },
        RawReplyLimits {
            reply_bytes: 2,
            ..LIMITS
        },
        RawReplyLimits {
            method_name_bytes: 3,
            ..LIMITS
        },
    ] {
        assert!(matches!(
            module(&[method], limits, None, None, 10_000),
            Err(HeldReplyError::Catalog(RawReplyError::Budget { .. }))
        ));
    }
    assert!(matches!(
        module(
            &[RawReplyMethod {
                mode: ReplyMode::Query,
                ..method
            }],
            LIMITS,
            None,
            None,
            10_000
        ),
        Err(HeldReplyError::Query { slot: 0 })
    ));
    for name in ["", "bad\0", "bad\n"] {
        assert!(matches!(
            module(
                &[RawReplyMethod { name, ..method }],
                LIMITS,
                None,
                None,
                10_000
            ),
            Err(HeldReplyError::Catalog(RawReplyError::InvalidName {
                index: 0
            }))
        ));
    }
    assert!(matches!(
        module(&[method], LIMITS, None, None, 1),
        Err(HeldReplyError::Compile(CompileError::ModuleLimit { .. }))
    ));
    let mut offset = usize::MAX;
    assert!(segment(&mut String::new(), &mut offset, b"x").is_err());
}

#[test]
fn validates_cycle_caps_without_changing_existing_catalog_admission() {
    let methods = [RawReplyMethod {
        name: "paid",
        mode: ReplyMode::Update,
        reply: b"opaque\0\xff",
    }];
    for cap in [0, 1, 1 << 65, u128::MAX] {
        for barrier in [
            None,
            Some(("https://ic-testkit.invalid/test/", 50_000_000_000)),
        ] {
            let wasm = module(&methods, LIMITS, barrier, Some(cap), 10_000).unwrap();
            validate(&wasm, 10_000).unwrap();
            assert!(
                wasm.windows(20)
                    .any(|bytes| bytes == b"msg_cycles_accept128")
            );
        }
    }
    for limits in [
        RawReplyLimits {
            methods: 0,
            ..LIMITS
        },
        RawReplyLimits {
            reply_bytes: 0,
            ..LIMITS
        },
    ] {
        assert!(matches!(
            module(&methods, limits, None, Some(0), 10_000),
            Err(HeldReplyError::Catalog(RawReplyError::Budget { .. }))
        ));
    }
}
