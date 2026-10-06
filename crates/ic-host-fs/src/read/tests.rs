use super::*;
use ic_host_artifacts::artifact::Sha256Digest;
use std::io;

#[test]
fn bounded_file_operations_leave_source_intact_and_reject_special_inputs() {
    // A checked-in source fixture avoids temp allocation and filesystem cleanup.
    let path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/empty.wasm"
    ));
    let original = std::fs::read(path).unwrap();
    assert_eq!(read_file(path, 8).unwrap(), original);
    assert_eq!(
        hash_file(path, 8).unwrap().sha256,
        Sha256Digest::compute(&original)
    );
    assert!(matches!(
        read_file(path, 7),
        Err(ArtifactError::LimitExceeded { limit: 7 })
    ));
    assert!(matches!(
        hash_file(path.parent().unwrap(), 100),
        Err(ArtifactError::NotRegularFile)
    ));
    assert_eq!(std::fs::read(path).unwrap(), original);
    let missing = path.with_extension("missing");
    assert!(
        matches!(hash_file(&missing, 10), Err(ArtifactError::Io(source)) if source.kind() == io::ErrorKind::NotFound)
    );
}
