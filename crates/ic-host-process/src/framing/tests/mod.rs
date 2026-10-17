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

#[test]
fn chunk_framing_preserves_split_frames_and_eof_policy() {
    let input = "a\r\nβ\nlast".as_bytes();
    for split_mask in 0..(1_usize << (input.len() - 1)) {
        let mut framer = LineFramer::new(5);
        let mut output = Vec::new();
        let mut start = 0;
        for end in 1..=input.len() {
            if end == input.len() || split_mask & (1 << (end - 1)) != 0 {
                framer
                    .feed(&input[start..end], |frame| {
                        output.push(frame.to_vec());
                        Ok(())
                    })
                    .unwrap();
                start = end;
            }
        }
        assert_eq!(output, [b"a\r\n".as_slice(), "β\n".as_bytes()]);
        let error = framer.finish().unwrap_err();
        assert!(matches!(&error, FrameReadError::Unterminated(_)));
        assert_eq!(error.rejected_bytes(), Some(b"last".as_slice()));
        assert!(!error.to_string().contains("last"));
    }
    let mut framer = LineFramer::new(u64::MAX);
    framer.feed(b"\n", |_| Ok(())).unwrap();
    framer.finish().unwrap();
    LineFramer::new(0).finish().unwrap();
}

#[test]
fn chunk_overflow_and_sink_refusal_are_terminal() {
    let mut framer = LineFramer::new(3);
    framer.feed(b"a", |_| panic!("no complete frame")).unwrap();
    let error = framer
        .feed(b"bcd\nnext\n", |_| panic!("refused frame must not emit"))
        .unwrap_err();
    assert!(matches!(&error, FrameReadError::TooLarge(_)));
    assert_eq!(error.rejected_bytes(), Some(b"abcd".as_slice()));
    assert!(!error.to_string().contains("abcd"));
    assert!(
        matches!(framer.feed(b"", |_| Ok(())), Err(FrameReadError::Io(source)) if source.kind() == io::ErrorKind::InvalidInput)
    );
    assert!(framer.finish().is_err());

    let mut framer = LineFramer::new(0);
    assert!(
        matches!(framer.feed(b"\n", |_| panic!("zero allowance")), Err(FrameReadError::TooLarge(bytes)) if bytes == b"\n")
    );
    let mut framer = LineFramer::new(3);
    let mut calls = 0;
    let error = framer
        .feed(b"a\nb\n", |_| {
            calls += 1;
            Err(io::Error::from_raw_os_error(13))
        })
        .unwrap_err();
    assert_eq!(calls, 1);
    assert!(matches!(error, FrameReadError::Io(source) if source.raw_os_error() == Some(13)));
    assert!(framer.feed(b"c\n", |_| panic!("no recovery")).is_err());
}

#[test]
fn reader_and_chunk_framing_agree_on_limits() {
    for input in [
        b"".as_slice(),
        b"\n",
        b"secret",
        b"long\n",
        "β\n".as_bytes(),
    ] {
        for maximum in 0..=10 {
            let mut reader = input;
            let expected = read_line(&mut reader, maximum);
            let mut framer = LineFramer::new(maximum);
            let mut output = None;
            let result = framer
                .feed(input, |bytes| {
                    output = Some(bytes.to_vec());
                    Ok(())
                })
                .and_then(|()| framer.finish())
                .and_then(|()| output.ok_or(FrameReadError::EndOfStream));
            assert_eq!(format!("{result:?}"), format!("{expected:?}"));
        }
    }
}
