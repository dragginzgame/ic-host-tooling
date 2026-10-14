//! Fresh-process collision preservation for the opt-in fixture allocator.
#![cfg(feature = "test-support")]

#[test]
fn a_fresh_process_skips_an_occupied_candidate_without_reclaiming_it() {
    use std::{
        fs,
        io::Write as _,
        time::{SystemTime, UNIX_EPOCH},
    };
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let prefix = format!(
        "ic-host-collision-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    // This isolated test binary has not called the allocator: its first candidate
    // collides with evidence deliberately retained in the caller-owned namespace.
    let occupied = parent.join(format!("{prefix}-{}-0", std::process::id()));
    fs::File::create_new(&occupied)
        .unwrap()
        .write_all(b"retained evidence")
        .unwrap();
    let allocated = ic_host_fs::testing::create_fixture_directory(&parent, &prefix).unwrap();
    assert_ne!(allocated, occupied);
    assert_eq!(fs::read(&occupied).unwrap(), b"retained evidence");
    assert!(allocated.is_dir());
    fs::remove_dir(&allocated).unwrap();
    fs::remove_file(&occupied).unwrap();
}
