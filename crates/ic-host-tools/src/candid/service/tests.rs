use super::*;

const LIMITS: ServiceLimits = ServiceLimits {
    source_bytes: 16_384,
    methods: 32,
};

#[test]
fn aliases_constructors_comments_and_escaped_names_preserve_modes() {
    let source = r#"
        /* braces { and service : inside comments are not declarations */
        type Node = opt record { next : Node; value : nat };
        type Read = func (Node) -> (Node) query;
        type API = service {
            write : () -> ();
            notify : () -> () oneway;
            "a:b\22 { //" : Read;
            composed : () -> () composite_query;
        };
        service : (text) -> API;
    "#;
    let methods = inspect(source, LIMITS).unwrap();
    assert_eq!(
        methods,
        vec![
            ServiceMethod {
                name: "a:b\" { //".into(),
                mode: MethodMode::Query
            },
            ServiceMethod {
                name: "composed".into(),
                mode: MethodMode::CompositeQuery
            },
            ServiceMethod {
                name: "notify".into(),
                mode: MethodMode::Oneway
            },
            ServiceMethod {
                name: "write".into(),
                mode: MethodMode::Update
            },
        ]
    );
    assert_eq!(
        methods[2].as_export(),
        ExportedMethod {
            name: "notify",
            mode: MethodMode::Update
        }
    );
}

#[test]
fn invalid_or_partial_contracts_never_return_an_inventory() {
    for source in [
        "service : { read : () -> () query;",
        "service : { read : () -> (); read : () -> () query; };",
        "service : { read : Missing; };",
        "type A = B; type B = A; service : {};",
        "service : { read : () -> () query oneway; };",
        "service : { notify : () -> (nat) oneway; };",
        "service : { read : nat; };",
        "service : {}; trailing garbage",
    ] {
        assert!(
            matches!(inspect(source, LIMITS), Err(ServiceError::Parse(_))),
            "{source}"
        );
    }
    assert!(matches!(
        inspect("type A = nat;", LIMITS),
        Err(ServiceError::MissingService)
    ));
    for source in [
        "import \"missing.did\"; service : {};",
        "import service \"missing.did\"; service : {};",
    ] {
        assert!(matches!(
            inspect(source, LIMITS),
            Err(ServiceError::ImportsUnsupported)
        ));
    }
}

#[test]
fn bounds_are_inclusive_and_checked_at_the_documented_boundary() {
    let source = "service : { m : () -> (); };";
    assert!(
        inspect(
            source,
            ServiceLimits {
                source_bytes: source.len(),
                methods: 1
            }
        )
        .is_ok()
    );
    assert!(
        matches!(inspect(source, ServiceLimits { source_bytes: source.len() - 1, methods: 1 }),
        Err(ServiceError::InputLimit { actual, limit }) if actual == source.len() && limit == source.len() - 1)
    );
    assert!(matches!(
        inspect(
            source,
            ServiceLimits {
                methods: 0,
                ..LIMITS
            }
        ),
        Err(ServiceError::MethodLimit {
            actual: 1,
            limit: 0
        })
    ));
    assert_eq!(
        inspect(
            "service : {};",
            ServiceLimits {
                methods: 0,
                ..LIMITS
            }
        )
        .unwrap(),
        Vec::<ServiceMethod>::new()
    );
    assert!(matches!(
        inspect(
            "not candid",
            ServiceLimits {
                source_bytes: 0,
                methods: 0
            }
        ),
        Err(ServiceError::InputLimit { .. })
    ));
}

#[test]
fn shared_argument_types_do_not_expand_into_the_result() {
    use std::fmt::Write as _;
    let mut source = String::from("type T0 = nat;\n");
    for index in 1..40 {
        writeln!(
            source,
            "type T{index} = record {{ a : T{}; b : T{} }};",
            index - 1,
            index - 1
        )
        .unwrap();
    }
    source.push_str("service : { read : (T39) -> (T39) query; };");
    assert_eq!(
        inspect(&source, LIMITS).unwrap(),
        vec![ServiceMethod {
            name: "read".into(),
            mode: MethodMode::Query
        }]
    );
}
