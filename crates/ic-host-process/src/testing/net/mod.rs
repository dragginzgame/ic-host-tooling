//! Explicit deadline-bounded native socket acceptance for test fixtures.

use std::{
    io,
    net::{SocketAddr, TcpListener, TcpStream},
    time::{Duration, Instant},
};

/// Accept one native test connection before an absolute deadline.
///
/// Selects nonblocking mode on the listener and blocking mode on the returned
/// stream explicitly, including on macOS. Retries interrupted accepts and bounds
/// idle waits by the remaining deadline. The caller owns socket provenance,
/// request framing, read/write deadlines and cancellation policy.
///
/// # Errors
/// Returns `TimedOut` without accepting when the deadline has elapsed, or the
/// original socket setup/accept error. Does not spawn a worker or send bytes.
pub fn accept_tcp_until(
    listener: &TcpListener,
    deadline: Instant,
) -> io::Result<(TcpStream, SocketAddr)> {
    accept_tcp_until_with_cancellation(listener, deadline, || false)
}

// One accept loop serves caller-owned sockets and the owned HTTP fixture worker.
pub(super) fn accept_tcp_until_with_cancellation(
    listener: &TcpListener,
    deadline: Instant,
    mut cancelled: impl FnMut() -> bool,
) -> io::Result<(TcpStream, SocketAddr)> {
    listener.set_nonblocking(true)?;
    loop {
        if cancelled() {
            return Err(io::ErrorKind::Interrupted.into());
        }
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or(io::ErrorKind::TimedOut)?;
        match listener.accept() {
            Ok((stream, peer)) => {
                stream.set_nonblocking(false)?;
                return Ok((stream, peer));
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(remaining.min(Duration::from_millis(5)));
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests;
