use std::{collections::TryReserveError, error::Error, fmt};

use ic_host_artifacts::artifact::buffer::reserve_bounded;
use reqwest::{Response, StatusCode};

/// Explicit body admission and failure-evidence choices; no hidden byte ceiling.
#[derive(Clone, Copy, Debug)]
pub struct ReadOptions {
    /// Maximum decoded body bytes retained by this reader.
    pub max_bytes: usize,
    /// Reject an available oversized declared length before reading any bytes.
    /// A missing or smaller declaration never bypasses streamed admission.
    pub reject_declared_oversize: bool,
    /// Preserve the bounded prefix on failure, including an overflowing chunk.
    /// False returns no body evidence on any failure.
    pub retain_partial_on_error: bool,
}

/// A complete body. HTTP status has not been classified as success or failure.
#[derive(Debug)]
pub struct Body {
    /// Original HTTP status, including unsuccessful statuses.
    pub status: StatusCode,
    /// Complete decoded bytes within the selected ceiling.
    pub bytes: Vec<u8>,
}

/// The cause of an incomplete body read.
#[derive(Debug)]
pub enum ReadFailure {
    /// The declared decoded length exceeded the ceiling before reading.
    DeclaredLength {
        /// Length reported by reqwest; not proof of bytes received.
        declared: u64,
        /// Caller-selected ceiling.
        limit: usize,
    },
    /// Bytes yielded by the response exceeded the ceiling.
    Limit {
        /// Caller-selected ceiling.
        limit: usize,
    },
    /// Original body transport or decompression failure.
    Transport(reqwest::Error),
    /// Original allocation/capacity failure while reserving bounded storage.
    Allocation(TryReserveError),
}

/// Typed failure plus optional caller-selected evidence.
///
/// Display excludes body bytes; consumers must redact evidence before logging.
#[derive(Debug)]
pub struct ReadError {
    /// HTTP status remains available even when body reading fails.
    pub status: StatusCode,
    /// Bytes yielded before stopping, including the entire overflowing chunk.
    /// Saturates at `u64::MAX` if the running count cannot be represented.
    pub received_bytes: u64,
    /// Bounded prefix, or empty when failure retention was disabled.
    pub partial_body: Vec<u8>,
    /// Original typed refusal.
    pub failure: ReadFailure,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTTP body read failed (status={}, received={}): ",
            self.status, self.received_bytes
        )?;
        match &self.failure {
            ReadFailure::DeclaredLength { declared, limit } => {
                write!(f, "declared length {declared} exceeds {limit}")
            }
            ReadFailure::Limit { limit } => write!(f, "body exceeds {limit}"),
            ReadFailure::Transport(_) => f.write_str("transport or decompression failure"),
            ReadFailure::Allocation(_) => f.write_str("allocation failure"),
        }
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.failure {
            ReadFailure::Transport(source) => Some(source),
            ReadFailure::Allocation(source) => Some(source),
            ReadFailure::DeclaredLength { .. } | ReadFailure::Limit { .. } => None,
        }
    }
}

/// Consume an already-created response under explicit caller-selected limits.
///
/// No status refusal, retry, request, timeout or decoding policy is added. Check
/// status before calling for early rejection, or inspect the returned status to
/// classify it after retaining body evidence. Overflow stops reading immediately.
/// Allocation is fallible and requested vector capacity never exceeds `max_bytes`;
/// the allocator may round allocations up. The response is dropped on failure.
///
/// # Errors
/// Returns declared-size, streamed-size, transport/decompression or allocation
/// failure. The error retains status and, only when requested, partial bytes.
pub async fn read_body(mut response: Response, options: ReadOptions) -> Result<Body, ReadError> {
    let status = response.status();
    let mut bytes = Vec::new();
    let mut received_bytes = 0_u64;
    let result = async {
        if options.reject_declared_oversize
            && let Some(declared) = response.content_length()
            && u128::from(declared) > options.max_bytes as u128
        {
            return Err(ReadFailure::DeclaredLength {
                declared,
                limit: options.max_bytes,
            });
        }
        while let Some(chunk) = response.chunk().await.map_err(ReadFailure::Transport)? {
            received_bytes =
                received_bytes.saturating_add(u64::try_from(chunk.len()).unwrap_or(u64::MAX));
            let overflow = chunk.len() > options.max_bytes.saturating_sub(bytes.len());
            if overflow && !options.retain_partial_on_error {
                return Err(ReadFailure::Limit {
                    limit: options.max_bytes,
                });
            }
            let admitted = reserve_bounded(&mut bytes, chunk.len(), options.max_bytes)
                .map_err(ReadFailure::Allocation)?;
            bytes.extend_from_slice(&chunk[..admitted]);
            if overflow {
                return Err(ReadFailure::Limit {
                    limit: options.max_bytes,
                });
            }
        }
        Ok(())
    }
    .await;
    match result {
        Ok(()) => Ok(Body { status, bytes }),
        Err(failure) => Err(ReadError {
            status,
            received_bytes,
            partial_body: if options.retain_partial_on_error {
                bytes
            } else {
                Vec::new()
            },
            failure,
        }),
    }
}

#[cfg(test)]
mod tests;
