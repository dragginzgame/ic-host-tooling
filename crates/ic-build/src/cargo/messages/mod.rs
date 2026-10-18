//! Shared Cargo message decoding and exact package/target matching.
use super::observation::schema;
use ic_host_process::framing::{FrameReadError, LineFramer};
use std::io;
pub fn artifact_matches(
    artifact: &schema::Artifact,
    package: &str,
    name: &str,
    kind: &schema::TargetKind,
) -> bool {
    artifact.package_id.repr == package
        && artifact.target.name == name
        && artifact.target.kind.contains(kind)
}
pub struct MessageDecoder {
    framing: LineFramer,
    allow_text: bool,
}
impl MessageDecoder {
    pub const fn new(maximum: usize, allow_text: bool) -> Self {
        Self {
            framing: LineFramer::new(maximum as u64),
            allow_text,
        }
    }
    pub fn feed(
        &mut self,
        bytes: &[u8],
        mut emit: impl FnMut(schema::Message) -> io::Result<()>,
    ) -> io::Result<()> {
        let allow_text = self.allow_text;
        self.framing
            .feed(bytes, |frame| emit(Self::decode(frame, allow_text)?))
            .map_err(Self::framing_error)
    }
    pub fn finish(self, mut emit: impl FnMut(schema::Message) -> io::Result<()>) -> io::Result<()> {
        match self.framing.finish() {
            Ok(()) => Ok(()),
            // Cargo permits a final record without a newline.
            Err(FrameReadError::Unterminated(tail)) => emit(Self::decode(&tail, self.allow_text)?),
            Err(error) => Err(Self::framing_error(error)),
        }
    }
    fn framing_error(error: FrameReadError) -> io::Error {
        match error {
            FrameReadError::Io(source) => source,
            FrameReadError::TooLarge(_) => io::Error::new(
                io::ErrorKind::InvalidData,
                "Cargo JSON message exceeds caller allowance",
            ),
            other => io::Error::new(io::ErrorKind::InvalidData, other),
        }
    }
    fn decode(bytes: &[u8], allow_text: bool) -> io::Result<schema::Message> {
        match serde_json::from_slice(bytes) {
            Ok(message) => Ok(message),
            Err(_)
                if allow_text
                    && bytes
                        .iter()
                        .copied()
                        .find(|byte| !byte.is_ascii_whitespace())
                        != Some(b'{') =>
            {
                Ok(schema::Message::TextLine(
                    String::from_utf8_lossy(bytes).into_owned(),
                ))
            }
            Err(error) => Err(io::Error::new(io::ErrorKind::InvalidData, error)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_partitions_crlf_and_eof_tails_keep_complete_records() {
        let bytes = b"{\"reason\":\"build-finished\",\"success\":true}\r\n{\"reason\":\"build-finished\",\"success\":false}";
        for width in 1..=bytes.len() {
            let mut decoder = MessageDecoder::new(bytes.len(), false);
            let mut successes = Vec::new();
            let mut receive = |message| {
                let schema::Message::BuildFinished(message) = message else {
                    panic!("build-finished record");
                };
                successes.push(message.success);
                Ok(())
            };
            for chunk in bytes.chunks(width) {
                decoder.feed(chunk, &mut receive).unwrap();
            }
            decoder.finish(&mut receive).unwrap();
            assert_eq!(successes, [true, false]);
        }
    }

    #[test]
    fn limits_malformed_json_text_policy_and_original_sink_errors() {
        let mut limited = MessageDecoder::new(3, false);
        assert!(limited.feed(b"1234", |_| Ok(())).is_err());
        for allow_text in [false, true] {
            let mut malformed = MessageDecoder::new(64, allow_text);
            assert!(malformed.feed(b"{invalid}\n", |_| Ok(())).is_err());
        }
        let mut strict = MessageDecoder::new(64, false);
        assert!(strict.feed(b"plain output\n", |_| Ok(())).is_err());
        let mut text = MessageDecoder::new(64, true);
        text.feed(b"plain output\n", |message| {
            assert!(matches!(message, schema::Message::TextLine(_)));
            Ok(())
        })
        .unwrap();
        let bytes = b"{\"reason\":\"build-finished\",\"success\":true}\n";
        let mut decoder = MessageDecoder::new(bytes.len(), false);
        let error = decoder
            .feed(bytes, |_| Err(io::Error::from_raw_os_error(28)))
            .unwrap_err();
        assert_eq!(error.raw_os_error(), Some(28));

        // The same exact bound applies to newline records and accepted EOF tails.
        for bytes in [bytes.as_slice(), &bytes[..bytes.len() - 1]] {
            let mut exact = MessageDecoder::new(bytes.len(), false);
            let mut delivered = 0;
            exact
                .feed(bytes, |_| {
                    delivered += 1;
                    Ok(())
                })
                .unwrap();
            exact
                .finish(|_| {
                    delivered += 1;
                    Ok(())
                })
                .unwrap();
            assert_eq!(delivered, 1);
            let mut short = MessageDecoder::new(bytes.len() - 1, false);
            let error = short
                .feed(bytes, |_| panic!("oversized record must not emit"))
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(
                error.to_string(),
                "Cargo JSON message exceeds caller allowance"
            );
        }
        let mut tail = MessageDecoder::new(bytes.len(), false);
        tail.feed(&bytes[..bytes.len() - 1], |_| panic!("not yet complete"))
            .unwrap();
        assert_eq!(
            tail.finish(|_| Err(io::Error::from_raw_os_error(28)))
                .unwrap_err()
                .raw_os_error(),
            Some(28)
        );
        let mut malformed_tail = MessageDecoder::new(64, true);
        malformed_tail.feed(b"{invalid}", |_| Ok(())).unwrap();
        assert_eq!(
            malformed_tail.finish(|_| Ok(())).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}
