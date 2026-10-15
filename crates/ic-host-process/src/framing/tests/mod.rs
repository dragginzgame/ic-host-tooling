use super::*;
use std::io::BufReader;

#[test]
fn fragmented_exact_frames_keep_following_messages_and_distinct_eof() {
    let mut reader = BufReader::with_capacity(1, &b"a\r\nb\n"[..]);
    assert_eq!(read_line(&mut reader, 3).unwrap(), b"a\r\n");
    assert_eq!(read_line(&mut reader, 2).unwrap(), b"b\n");
    assert!(matches!(
        read_line(&mut reader, 2),
        Err(FrameReadError::EndOfStream)
    ));
}

#[test]
fn overflow_and_partial_frames_remain_bounded_and_never_resynchronize() {
    let mut reader = &b"secret\nnext\n"[..];
    let error = read_line(&mut reader, 3).unwrap_err();
    assert!(matches!(&error, FrameReadError::TooLarge(_)));
    assert_eq!(error.rejected_bytes(), Some(b"secr".as_slice()));
    assert!(!error.to_string().contains("secr"));
    assert_eq!(reader, b"et\nnext\n");
    let error = read_line(&mut &b"partial"[..], 7).unwrap_err();
    assert!(matches!(&error, FrameReadError::Unterminated(_)));
    assert_eq!(error.rejected_bytes(), Some(b"partial".as_slice()));
    assert!(matches!(
        read_line(&mut &b"\n"[..], 0),
        Err(FrameReadError::TooLarge(_))
    ));
    assert_eq!(read_line(&mut &b"\n"[..], u64::MAX).unwrap(), b"\n");
}

#[test]
fn io_failures_keep_the_original_source() {
    let mut reader = BufReader::new(Failing);
    let error = read_line(&mut reader, 10).unwrap_err();
    assert!(
        matches!(&error, FrameReadError::Io(source) if source.kind() == io::ErrorKind::PermissionDenied)
    );
    assert!(std::error::Error::source(&error).is_some());
}

struct Failing;
impl io::Read for Failing {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "reader refused",
        ))
    }
}
