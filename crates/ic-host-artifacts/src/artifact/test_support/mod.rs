use std::io::{self, Read, Write};

pub(super) struct DeniedReader;

impl Read for DeniedReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::ErrorKind::PermissionDenied.into())
    }
}

pub(super) struct InvalidReader;

impl Read for InvalidReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        Ok(buffer.len() + 1)
    }
}

pub(super) struct InvalidWriter;

impl Write for InvalidWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        Ok(bytes.len() + 1)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
