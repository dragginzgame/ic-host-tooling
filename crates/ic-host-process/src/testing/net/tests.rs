use super::*;
use std::io::Write as _;

#[test]
fn bounded_accept_preserves_peer_and_request_bytes() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    client.write_all(b"GET /original HTTP/1.1\r\n\r\n").unwrap();
    let deadline = Instant::now().checked_add(Duration::from_secs(5)).unwrap();
    let (mut stream, peer) = accept_tcp_until(&listener, deadline).unwrap();
    assert_eq!(peer, client.local_addr().unwrap());
    assert_eq!(
        crate::testing::http::capture_http_request(
            &mut stream,
            Duration::from_secs(5),
            128,
            0,
            crate::testing::http::HttpContentLength::Optional,
        )
        .into_result()
        .unwrap(),
        b"GET /original HTTP/1.1\r\n\r\n"
    );
}

#[test]
fn elapsed_accept_deadline_leaves_queued_connection_unconsumed() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let _client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    assert_eq!(
        accept_tcp_until(&listener, Instant::now())
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    listener.accept().unwrap();
}

#[test]
fn idle_accept_expires_without_a_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let deadline = Instant::now()
        .checked_add(Duration::from_millis(20))
        .unwrap();
    assert_eq!(
        accept_tcp_until(&listener, deadline).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
}
