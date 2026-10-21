use super::*;
use std::{
    io::{Read as _, Write as _},
    net::TcpListener,
    time::Duration,
};

async fn response(headers: &str, body: &[u8], gzip: bool) -> Response {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/", listener.local_addr().unwrap());
    let raw = [headers.as_bytes(), b"\r\n\r\n", body].concat();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        stream.write_all(&raw).unwrap();
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(5))
        .gzip(gzip)
        .build()
        .unwrap()
        .get(url)
        .send()
        .await
        .unwrap();
    server.join().unwrap();
    response
}

fn options(max_bytes: usize, retain: bool) -> ReadOptions {
    ReadOptions {
        max_bytes,
        reject_declared_oversize: false,
        retain_partial_on_error: retain,
    }
}

#[tokio::test(flavor = "current_thread")]
async fn complete_bodies_preserve_status_and_exact_limits() {
    for bytes in [b"".as_slice(), b"data"] {
        let headers = format!("HTTP/1.1 404 Not Found\r\nContent-Length: {}", bytes.len());
        let body = read_body(
            response(&headers, bytes, false).await,
            options(bytes.len(), false),
        )
        .await
        .unwrap();
        assert_eq!(body.status, StatusCode::NOT_FOUND);
        assert_eq!(body.bytes, bytes);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn declared_oversize_is_optional_and_does_not_claim_received_bytes() {
    for reject in [false, true] {
        let mut settings = options(3, true);
        settings.reject_declared_oversize = reject;
        let error = read_body(
            response("HTTP/1.1 503 Busy\r\nContent-Length: 4", b"data", false).await,
            settings,
        )
        .await
        .unwrap_err();
        assert_eq!(error.status, StatusCode::SERVICE_UNAVAILABLE);
        if reject {
            assert!(matches!(
                error.failure,
                ReadFailure::DeclaredLength {
                    declared: 4,
                    limit: 3
                }
            ));
            assert_eq!(error.received_bytes, 0);
            assert_eq!(error.partial_body, [] as [u8; 0]);
        } else {
            assert!(matches!(error.failure, ReadFailure::Limit { limit: 3 }));
            assert_eq!(error.partial_body, b"dat");
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn streamed_overflow_keeps_only_the_requested_prefix() {
    for wire in [
        b"6\r\nsecret\r\n0\r\n\r\n".as_slice(),
        b"2\r\nse\r\n4\r\ncret\r\n0\r\n\r\n",
    ] {
        for retain in [false, true] {
            for limit in [0, 3] {
                let error = read_body(
                    response("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked", wire, false).await,
                    options(limit, retain),
                )
                .await
                .unwrap_err();
                assert!(matches!(error.failure, ReadFailure::Limit { .. }));
                assert!(error.received_bytes > limit as u64);
                assert_eq!(
                    error.partial_body,
                    if retain { &b"secret"[..limit] } else { b"" }
                );
                assert!(!error.to_string().contains("secret"));
            }
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn body_transport_failure_preserves_typed_cause_and_optional_evidence() {
    for retain in [false, true] {
        let error = read_body(
            response(
                "HTTP/1.1 502 Bad Gateway\r\nTransfer-Encoding: chunked",
                b"3\r\nabc\r\n",
                false,
            )
            .await,
            options(10, retain),
        )
        .await
        .unwrap_err();
        assert_eq!(error.status, StatusCode::BAD_GATEWAY);
        assert_eq!(error.received_bytes, 3);
        assert_eq!(
            error.partial_body,
            if retain { b"abc".as_slice() } else { b"" }
        );
        assert!(matches!(error.failure, ReadFailure::Transport(_)));
        assert!(error.source().unwrap().is::<reqwest::Error>());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn decoded_bytes_are_bounded_even_when_compressed_length_is_small() {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(&[b'x'; 4096]).unwrap();
    let compressed = encoder.finish().unwrap();
    assert!(compressed.len() < 128);
    let headers = format!(
        "HTTP/1.1 200 OK\r\nContent-Encoding: gzip\r\nContent-Length: {}",
        compressed.len()
    );
    let error = read_body(
        response(&headers, &compressed, true).await,
        ReadOptions {
            max_bytes: 128,
            reject_declared_oversize: true,
            retain_partial_on_error: true,
        },
    )
    .await
    .unwrap_err();
    assert!(matches!(error.failure, ReadFailure::Limit { limit: 128 }));
    assert_eq!(error.partial_body, vec![b'x'; 128]);
}

#[tokio::test(flavor = "current_thread")]
async fn missing_length_and_truncated_declared_length_remain_stream_checked() {
    let body = read_body(
        response("HTTP/1.1 200 OK\r\nConnection: close", b"abc", false).await,
        options(3, false),
    )
    .await
    .unwrap();
    assert_eq!(body.bytes, b"abc");
    let error = read_body(
        response("HTTP/1.1 200 OK\r\nContent-Length: 10", b"abc", false).await,
        options(10, true),
    )
    .await
    .unwrap_err();
    assert!(matches!(error.failure, ReadFailure::Transport(_)));
    assert!(error.partial_body.len() <= 3);
}

#[tokio::test(flavor = "current_thread")]
async fn cancelled_read_preserves_caller_owned_prefix() {
    for retain in [false, true] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let (release, hold) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            stream
                .write_all(b"HTTP/1.1 503 Busy\r\nContent-Length: 6\r\n\r\nabc")
                .unwrap();
            hold.recv_timeout(Duration::from_secs(5)).unwrap();
        });
        let response = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(url)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let mut bytes = b"old evidence".to_vec();
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            read_body_into(response, options(6, retain), &mut bytes),
        )
        .await;
        assert!(result.is_err());
        assert_eq!(bytes, b"abc");
        release.send(()).unwrap();
        server.join().unwrap();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn caller_buffer_replaces_old_bytes_and_moves_error_evidence() {
    let mut bytes = b"old".to_vec();
    let status = read_body_into(
        response("HTTP/1.1 200 OK\r\nContent-Length: 3", b"abc", false).await,
        options(3, true),
        &mut bytes,
    )
    .await
    .unwrap();
    assert_eq!(status, StatusCode::OK);
    assert_eq!(bytes, b"abc");
    for retain in [false, true] {
        bytes.extend_from_slice(b"old");
        let error = read_body_into(
            response("HTTP/1.1 200 OK\r\nContent-Length: 3", b"xyz", false).await,
            options(2, retain),
            &mut bytes,
        )
        .await
        .unwrap_err();
        assert_eq!(bytes, [] as [u8; 0]);
        assert_eq!(
            error.partial_body,
            if retain { b"xy".as_slice() } else { b"" }
        );
    }
}
