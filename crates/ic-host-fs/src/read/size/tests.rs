use super::*;

#[test]
fn directory_size_sums_wide_and_nested_files() {
    let fixture = crate::test_support::fixture();
    let root = fixture.path();
    assert_eq!(directory_logical_size(root).unwrap(), 0);
    fs::create_dir_all(root.join("wide")).unwrap();
    fs::create_dir_all(root.join("nested/deep/empty")).unwrap();
    let mut expected = 0;
    for index in 0..128 {
        let bytes = vec![42; index % 13];
        fs::write(root.join("wide").join(index.to_string()), &bytes).unwrap();
        expected += bytes.len() as u64;
    }
    let sparse = root.join("nested/deep/sparse");
    fs::File::create(&sparse)
        .unwrap()
        .set_len(1024 * 1024)
        .unwrap();
    assert_eq!(directory_logical_size(&sparse).unwrap(), 1024 * 1024);
    assert_eq!(
        directory_logical_size(root).unwrap(),
        expected + 1024 * 1024
    );
    assert_eq!(
        directory_logical_size(&root.join("missing"))
            .unwrap_err()
            .kind(),
        std::io::ErrorKind::NotFound,
    );
}

#[test]
#[cfg(unix)]
fn directory_size_counts_symlinks_without_following_them() {
    let fixture = crate::test_support::fixture();
    let root = fixture.path();
    let walked = root.join("walked");
    fs::create_dir_all(&walked).unwrap();
    fs::create_dir_all(root.join("external")).unwrap();
    fs::write(root.join("external/payload"), vec![42; 4096]).unwrap();
    fs::write(root.join("outside-file"), vec![42; 4096]).unwrap();
    fs::write(walked.join("payload"), b"abc").unwrap();
    let targets = ["../external", "../outside-file", "missing", "."];
    for (index, target) in targets.iter().enumerate() {
        std::os::unix::fs::symlink(target, walked.join(index.to_string())).unwrap();
    }
    let expected = 3 + targets
        .iter()
        .map(|target| target.len() as u64)
        .sum::<u64>();
    assert_eq!(directory_logical_size(&walked).unwrap(), expected);
    assert_eq!(
        directory_logical_size(&walked.join("0")).unwrap(),
        targets[0].len() as u64,
    );
}
