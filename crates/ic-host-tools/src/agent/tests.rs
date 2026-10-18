use super::*;
use ic_host_process::testing::http::{HttpFixture, HttpFixtureLimits};
use ic_platform::{
    Principal,
    agent::{AgentError, TransportError},
};
use std::{sync::Barrier, thread, time::Duration};

#[test]
fn native_builder_retains_caller_customization_without_network() {
    agent_builder(&Url::parse("https://replica.example").unwrap(), 64)
        .with_ingress_expiry(Duration::from_secs(90))
        .build()
        .expect("customized native agent");
}

#[test]
fn binding_reuses_exact_pair_and_refuses_different_text_or_ceiling() {
    let source = EndpointAgent::default();
    let first = source.get("https://replica.example", 64).unwrap();
    assert!(std::ptr::eq(
        first,
        source.get("https://replica.example", 64).unwrap()
    ));
    // URL-equivalent text is still a different binding.
    for (endpoint, limit) in [
        ("https://replica.example/", 64),
        ("https://other.example", 64),
        ("https://replica.example", 65),
    ] {
        assert!(matches!(
            source.get(endpoint, limit),
            Err(EndpointAgentError::BindingMismatch)
        ));
    }
    assert!(std::ptr::eq(
        first,
        source.get("https://replica.example", 64).unwrap()
    ));
    assert!(source.clone().get("https://other.example", 64).is_err());
}

#[test]
fn failed_url_parse_leaves_binding_unselected() {
    let source = EndpointAgent::default();
    assert!(matches!(
        source.get("invalid endpoint", 64),
        Err(EndpointAgentError::Endpoint(_))
    ));
    assert!(source.transport.get().is_none());
    source.get("https://replica.example", 64).unwrap();
}

#[test]
fn competing_bindings_select_exactly_one_and_keep_the_winner() {
    let source = EndpointAgent::default();
    let barrier = Barrier::new(2);
    thread::scope(|scope| {
        let calls: Vec<_> = ["https://first.example", "https://second.example"]
            .map(|endpoint| {
                let source = &source;
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    (endpoint, source.get(endpoint, 64).is_ok())
                })
            })
            .into_iter()
            .map(|call| call.join().unwrap())
            .collect();
        assert_eq!(calls.iter().filter(|(_, success)| *success).count(), 1);
        for (endpoint, success) in calls {
            assert_eq!(source.get(endpoint, 64).is_ok(), success);
        }
    });
}

#[tokio::test(flavor = "current_thread")]
async fn native_transport_enforces_the_selected_response_ceiling() {
    let fixture = HttpFixture::spawn(
        &[b'x'; 65],
        "application/cbor",
        HttpFixtureLimits {
            timeout: Duration::from_secs(10),
            max_header_bytes: 64 * 1024,
            max_body_bytes: 8 * 1024 * 1024,
        },
    )
    .expect("response fixture");
    let source = EndpointAgent::default();
    let error = source
        .get(fixture.endpoint(), 64)
        .unwrap()
        .query(&Principal::anonymous(), "bounded_response")
        .call_without_verification()
        .await
        .expect_err("body exceeds the selected ceiling");
    fixture.finish().expect("fixture exchange");
    assert!(matches!(
        error,
        AgentError::TransportError(TransportError::Generic(_))
    ));
}
