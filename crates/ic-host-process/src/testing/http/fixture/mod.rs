//! Owned, one-request HTTP 200 fixtures for native transport tests.

use super::{HttpContentLength, evidence::capture_http_request_until};
use crate::testing::net::accept_tcp_until_with_cancellation;
use std::{
    io::{self, Write as _},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[cfg(test)]
mod tests;

///
/// `HttpFixtureLimits`
///
/// Explicit request budgets and one deadline covering accept, read and response writes.
///
#[derive(Clone, Copy, Debug)]
pub struct HttpFixtureLimits {
    /// Total fixture lifetime, starting before the worker is spawned.
    pub timeout: Duration,
    /// Maximum raw request-header bytes, including the request line.
    pub max_header_bytes: usize,
    /// Maximum request-body bytes.
    pub max_body_bytes: usize,
}

///
/// `HttpFixture`
///
/// One loopback HTTP responder; finishing returns the bounded, unmodified raw request.
///
/// Dropping cancels accept/read/write and joins the worker. Payload interpretation,
/// Candid/CBOR envelopes, source policy and response contents belong to the test.
///
#[derive(Debug)]
pub struct HttpFixture {
    endpoint: String,
    stop: Arc<AtomicBool>,
    active: Arc<Mutex<Option<TcpStream>>>,
    worker: Option<JoinHandle<io::Result<Vec<u8>>>>,
}

impl HttpFixture {
    /// Bind an ephemeral IPv4 loopback port and respond once with the supplied body.
    ///
    /// Sends HTTP 200, the selected Content-Type, exact Content-Length and Connection: close.
    /// Response bytes are supplied in memory by the caller; no network service is contacted.
    ///
    /// # Errors
    /// Refuses invalid timeout/byte budgets, invalid Content-Type, deadline overflow,
    /// socket setup failures and worker-spawn failures before returning a fixture.
    pub fn spawn(body: &[u8], content_type: &str, limits: HttpFixtureLimits) -> io::Result<Self> {
        if limits.timeout.is_zero()
            || limits.max_header_bytes < 4
            || limits
                .max_header_bytes
                .checked_add(limits.max_body_bytes)
                .is_none()
            || content_type.is_empty()
            || content_type.len() > 128
            || !content_type
                .bytes()
                .all(|byte| byte.is_ascii_graphic() || byte == b' ')
        {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        let deadline = Instant::now()
            .checked_add(limits.timeout)
            .ok_or(io::ErrorKind::InvalidInput)?;
        let mut response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()
        ).into_bytes();
        response.extend_from_slice(body);
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let endpoint = format!("http://{}", listener.local_addr()?);
        let stop = Arc::new(AtomicBool::new(false));
        let active = Arc::new(Mutex::new(None));
        let worker_stop = Arc::clone(&stop);
        let worker_active = Arc::clone(&active);
        let worker = thread::Builder::new()
            .name("ic-host-http-fixture".into())
            .spawn(move || {
                let result = serve(
                    &listener,
                    &response,
                    deadline,
                    limits,
                    &worker_stop,
                    &worker_active,
                );
                if let Ok(mut active) = worker_active.lock() {
                    *active = None;
                }
                result
            })?;
        Ok(Self {
            endpoint,
            stop,
            active,
            worker: Some(worker),
        })
    }

    /// The explicitly selected loopback endpoint for the caller's HTTP client.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Join the completed exchange and return its raw request, including headers.
    ///
    /// # Errors
    /// Returns the original request-admission, deadline or response-write failure;
    /// worker panics become an I/O error. No request causes a deadline failure.
    pub fn finish(mut self) -> io::Result<Vec<u8>> {
        self.worker
            .take()
            .ok_or_else(|| io::Error::other("HTTP fixture worker missing"))?
            .join()
            .map_err(|_| io::Error::other("HTTP fixture worker panicked"))?
    }
}

impl Drop for HttpFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Ok(active) = self.active.lock()
            && let Some(stream) = active.as_ref()
        {
            let _ = stream.shutdown(Shutdown::Both);
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn serve(
    listener: &TcpListener,
    response: &[u8],
    deadline: Instant,
    limits: HttpFixtureLimits,
    stop: &AtomicBool,
    active: &Mutex<Option<TcpStream>>,
) -> io::Result<Vec<u8>> {
    let (mut stream, _) =
        accept_tcp_until_with_cancellation(listener, deadline, || stop.load(Ordering::Relaxed))?;
    *active
        .lock()
        .map_err(|_| io::Error::other("HTTP fixture control poisoned"))? =
        Some(stream.try_clone()?);
    if stop.load(Ordering::Relaxed) {
        return Err(io::ErrorKind::Interrupted.into());
    }
    let request = capture_http_request_until(
        &mut stream,
        Vec::new(),
        deadline,
        limits.max_header_bytes,
        limits.max_body_bytes,
        HttpContentLength::Optional,
    )
    .into_result()?;
    write_response(&mut stream, response, deadline)?;
    Ok(request)
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| io::ErrorKind::TimedOut.into())
}

fn write_response(
    stream: &mut TcpStream,
    mut response: &[u8],
    deadline: Instant,
) -> io::Result<()> {
    while !response.is_empty() {
        stream.set_write_timeout(Some(remaining(deadline)?))?;
        match stream.write(response) {
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(bytes) => response = &response[bytes..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
    Ok(())
}
