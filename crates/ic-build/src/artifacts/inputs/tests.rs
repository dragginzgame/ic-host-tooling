use crate::artifacts::{
    InputEvidence, InputEvidenceError, InputEvidenceSpec, InputEvidenceValue,
    digest::{InputHasher, digest_labeled_paths},
    parse_cargo_depfile, read_cargo_depfile_inputs,
};
use crate::fs_fixture::unique_temp_directory;
use sha2::{Digest as _, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[test]
fn relative_digest_is_relocatable_and_tracks_names_bytes_identity_and_membership() {
    let left = unique_temp_directory("relative-inputs-left");
    let right = unique_temp_directory("relative-inputs-right");
    for root in [&left, &right] {
        fs::create_dir(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("src/path with λ.rs"), "// Unicode source\n").unwrap();
    }
    let before = capture(b"consumer-source", &[left.join("src"), left.join("absent")]);
    let relocated = capture(
        b"consumer-source",
        &[right.join("absent"), right.join("src")],
    );
    let expected = before.digest_relative_to(&left).unwrap();
    assert_eq!(expected, relocated.digest_relative_to(&right).unwrap());
    assert_ne!(before.digest().unwrap(), relocated.digest().unwrap());
    for mutation in 0..4 {
        match mutation {
            0 => fs::write(right.join("src/main.rs"), "fn main() { todo!() }\n").unwrap(),
            1 => fs::rename(right.join("src/main.rs"), right.join("src/renamed.rs")).unwrap(),
            2 => fs::write(right.join("src/new.rs"), "// new source\n").unwrap(),
            _ => fs::write(right.join("absent"), "created").unwrap(),
        }
        let after = capture(
            b"consumer-source",
            &[right.join("src"), right.join("absent")],
        );
        assert_ne!(expected, after.digest_relative_to(&right).unwrap());
        match mutation {
            0 => fs::write(right.join("src/main.rs"), "fn main() {}\n").unwrap(),
            1 => fs::rename(right.join("src/renamed.rs"), right.join("src/main.rs")).unwrap(),
            2 => fs::remove_file(right.join("src/new.rs")).unwrap(),
            _ => fs::remove_file(right.join("absent")).unwrap(),
        }
        assert_eq!(
            expected,
            capture(
                b"consumer-source",
                &[right.join("src"), right.join("absent")]
            )
            .digest_relative_to(&right)
            .unwrap(),
            "each mutation is tested independently"
        );
    }
    assert_ne!(
        expected,
        capture(
            b"different identity",
            &[left.join("src"), left.join("absent")]
        )
        .digest_relative_to(&left)
        .unwrap()
    );
    fs::remove_dir_all(&left).unwrap();
    fs::remove_dir_all(right).unwrap();
    assert_eq!(
        expected,
        before.digest_relative_to(&left).unwrap(),
        "digest uses captured evidence without rereading source"
    );
}

#[test]
fn relative_digest_refuses_outside_inputs_and_invalid_roots() {
    let root = unique_temp_directory("relative-inputs-refusal");
    fs::write(root.join("input"), "evidence").unwrap();
    let evidence = capture(b"identity", &[root.join("input")]);
    assert_eq!(
        evidence.digest_relative_to(&root).unwrap(),
        evidence.digest_relative_to(&root.join(".")).unwrap()
    );
    for invalid in [
        PathBuf::from("relative"),
        root.join("child"),
        root.join("../elsewhere"),
    ] {
        assert!(evidence.digest_relative_to(&invalid).is_err());
    }
    #[cfg(target_os = "linux")]
    {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};
        let native = root.join(OsStr::from_bytes(b"non-UTF8-\xff"));
        fs::write(&native, "native bytes").unwrap();
        assert!(matches!(
            capture(b"identity", &[native]).digest_relative_to(&root),
            Err(InputEvidenceError::Io { source, .. })
                if source.kind() == io::ErrorKind::InvalidData
        ));
    }
    fs::remove_dir_all(root).unwrap();
}

fn evidence_spec(identity: &[u8], paths: &[PathBuf]) -> InputEvidenceSpec {
    paths
        .iter()
        .fold(InputEvidenceSpec::new(identity), |spec, path| {
            spec.with_input(path)
        })
}

fn capture(identity: &[u8], paths: &[PathBuf]) -> InputEvidence {
    InputEvidence::capture(&evidence_spec(identity, paths)).unwrap()
}

#[test]
fn standalone_file_directory_absence_and_consumer_encoding_are_preserved() {
    let root = unique_temp_directory("input-evidence-encoding");
    let directory = root.join("a");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("child"), b"abc").unwrap();
    fs::write(directory.join("λ source"), b"Unicode filename").unwrap();
    fs::write(root.join("a.sibling"), b"sibling").unwrap();
    let missing = root.join("missing");
    let evidence = capture(b"same authority", &[root.clone(), missing.clone()]);
    assert_eq!(evidence.entries()[&root], InputEvidenceValue::Directory);
    assert_eq!(evidence.entries()[&missing], InputEvidenceValue::Missing);
    let InputEvidenceValue::File { sha256 } = evidence.entries()[&directory.join("child")] else {
        panic!("file evidence");
    };
    assert_eq!(
        sha256,
        ic_host_artifacts::artifact::Sha256Digest::compute(b"abc")
    );
    assert_eq!(
        sha256.to_string(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    // The prior consumer uses a String-keyed map. Path ordering is different for
    // `a.sibling` and `a/child`, so this checks the actual retained encoding.
    let child = directory.join("child");
    let sibling = root.join("a.sibling");
    assert!(child < sibling);
    assert!(child.to_str().unwrap() > sibling.to_str().unwrap());
    let mut fields = evidence
        .entries()
        .iter()
        .map(|(path, value)| {
            let value = match value {
                InputEvidenceValue::Missing => "absent".into(),
                InputEvidenceValue::Directory => "directory".into(),
                InputEvidenceValue::File { sha256 } => sha256.to_string(),
            };
            (path.to_str().unwrap(), value)
        })
        .collect::<Vec<_>>();
    fields.sort_by_key(|(path, _)| *path);
    let mut expected = Sha256::new();
    for field in std::iter::once(b"same authority".as_slice()).chain(
        fields
            .iter()
            .flat_map(|(path, hash)| [path.as_bytes(), hash.as_bytes()]),
    ) {
        expected.update(u64::try_from(field.len()).unwrap().to_be_bytes());
        expected.update(field);
    }
    assert_eq!(
        evidence.digest().unwrap().as_bytes(),
        &<[u8; 32]>::from(expected.finalize())
    );
    // Independently retain the component/join encoding, including the empty root
    // key, Unicode, missing entries and lexical ordering across path components.
    let mut expected = Sha256::new();
    let mut relative_fields = fields
        .iter()
        .map(|(path, value)| {
            let key = Path::new(path)
                .strip_prefix(&root)
                .unwrap()
                .components()
                .map(|part| part.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>()
                .join("/");
            (key, value)
        })
        .collect::<Vec<_>>();
    relative_fields.sort_by(|(left, _), (right, _)| left.cmp(right));
    for field in std::iter::once(b"same authority".as_slice()).chain(
        relative_fields
            .iter()
            .flat_map(|(path, hash)| [path.as_bytes(), hash.as_bytes()]),
    ) {
        expected.update(u64::try_from(field.len()).unwrap().to_be_bytes());
        expected.update(field);
    }
    assert_eq!(
        evidence.digest_relative_to(&root).unwrap().as_bytes(),
        &<[u8; 32]>::from(expected.finalize())
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replaced_inventory_rechecks_old_directory_membership_and_bytes() {
    let root = unique_temp_directory("input-evidence-replaced");
    let source = root.join("external.rs");
    fs::write(&source, b"first").unwrap();
    let before = capture(b"authority", std::slice::from_ref(&root));
    let after = evidence_spec(b"authority", &[]);
    before.validate_after(&after).unwrap();
    let added = root.join("new.rs");
    fs::write(&added, b"new").unwrap();
    assert!(
        matches!(before.validate_after(&after), Err(InputEvidenceError::ChangedInput { path }) if path == added)
    );
    fs::remove_file(&added).unwrap();
    fs::write(&source, b"other").unwrap();
    assert!(
        matches!(before.validate_after(&after), Err(InputEvidenceError::ChangedInput { path }) if path == source)
    );
    fs::remove_file(&source).unwrap();
    assert!(
        matches!(before.validate_after(&after), Err(InputEvidenceError::ChangedInput { path }) if path == source)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn directory_absence_and_late_external_discovery_have_distinct_authority() {
    let root = unique_temp_directory("input-evidence-absence");
    let before = capture(b"authority", std::slice::from_ref(&root));
    let missing = root.join("not-yet/child.rs");
    let after = before
        .validate_after(&evidence_spec(
            b"authority",
            &[root.clone(), missing.clone()],
        ))
        .unwrap();
    assert_ne!(before.digest().unwrap(), after.digest().unwrap());
    fs::create_dir_all(missing.parent().unwrap()).unwrap();
    fs::write(&missing, b"new bytes").unwrap();
    assert!(matches!(
        after.validate_after(&evidence_spec(b"authority", std::slice::from_ref(&root))),
        Err(InputEvidenceError::ChangedInput { .. })
    ));
    let outside = unique_temp_directory("input-evidence-outside");
    let foreign = outside.join("source");
    fs::write(&foreign, b"old but unobserved").unwrap();
    assert!(matches!(
        before.validate_after(&evidence_spec(b"authority", std::slice::from_ref(&foreign))),
        Err(InputEvidenceError::ChangedInput { .. })
    ));
    // Start from an unchanged directory so the failure is specifically late discovery.
    let before = capture(b"authority", std::slice::from_ref(&root));
    assert!(
        matches!(before.validate_after(&evidence_spec(b"authority", std::slice::from_ref(&foreign))), Err(InputEvidenceError::UnobservedInput { path }) if path == foreign)
    );
    fs::remove_dir_all(root).unwrap();
    fs::remove_dir_all(outside).unwrap();
}

#[test]
fn exclusions_cannot_prove_absence_and_policy_identity_is_retained() {
    let root = unique_temp_directory("input-evidence-exclusions");
    fs::create_dir(root.join("output")).unwrap();
    fs::write(root.join("output/old"), b"unscanned").unwrap();
    let spec = InputEvidenceSpec::new(b"authority".as_slice())
        .with_input(&root)
        .with_excluded_entry_name("output");
    let before = InputEvidence::capture(&spec).unwrap();
    fs::remove_file(root.join("output/old")).unwrap();
    let unscanned = root.join("output/new");
    let after = spec.with_input(&unscanned);
    assert!(
        matches!(before.validate_after(&after), Err(InputEvidenceError::UnobservedInput { path }) if path == unscanned)
    );
    let changed = InputEvidenceSpec::new(b"other".as_slice())
        .with_input(&root)
        .with_excluded_entry_name("output");
    assert!(matches!(
        before.validate_after(&changed),
        Err(InputEvidenceError::IdentityChanged)
    ));
    let changed = InputEvidenceSpec::new(b"authority".as_slice())
        .with_input(&root)
        .with_excluded_root(root.join("output"));
    assert!(matches!(
        before.validate_after(&changed),
        Err(InputEvidenceError::ExclusionsChanged)
    ));
    let spec = InputEvidenceSpec::new(Vec::new())
        .with_excluded_root(root.join("future-output"))
        .with_input(root.join("future-output/missing"));
    assert!(
        matches!(InputEvidence::capture(&spec), Err(InputEvidenceError::Io { source, .. }) if source.kind() == io::ErrorKind::InvalidInput)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn existing_tree_fingerprint_keeps_its_original_field_encoding() {
    let root = unique_temp_directory("input-evidence-existing-hash");
    let source = root.join("a");
    fs::write(&source, b"abc").unwrap();
    let actual = digest_labeled_paths("existing-v1", [(Path::new("logical"), &root)], &[]).unwrap();
    let mut expected = InputHasher::new("existing-v1");
    expected.field("directory", b"logical");
    expected.field("file-path", b"logical/a");
    expected.field("file-content", b"abc");
    assert_eq!(actual, expected.finish());
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn evidence_rejects_links_while_existing_hashing_still_follows_them() {
    let root = unique_temp_directory("input-evidence-links");
    let source = root.join("source");
    fs::write(&source, b"abc").unwrap();
    let link = root.join("link");
    std::os::unix::fs::symlink(&source, &link).unwrap();
    assert!(
        matches!(InputEvidence::capture(&InputEvidenceSpec::new(Vec::new()).with_input(&link)), Err(InputEvidenceError::Io { path, source }) if path == link && source.kind() == io::ErrorKind::InvalidInput)
    );
    assert_eq!(
        digest_labeled_paths("existing-v1", [(Path::new("logical"), &source)], &[]).unwrap(),
        digest_labeled_paths("existing-v1", [(Path::new("logical"), &link)], &[]).unwrap(),
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn depfiles_support_escaped_paths_multiple_rules_and_phony_targets() {
    let text = "target: /source/a /outside/with\\ space /outside/$$value\\#1 \\\r\n/outside/colon\\:name\r\n/source/a:\r\n\r\n# env-dep:KEY=value\nsecond: relative.rs\n";
    assert_eq!(
        parse_cargo_depfile(text).unwrap(),
        [
            "/outside/$value#1",
            "/outside/colon:name",
            "/outside/with space",
            "/source/a",
            "relative.rs"
        ]
        .map(PathBuf::from)
    );
    assert_eq!(
        parse_cargo_depfile(
            "first second: shared.rs other.rs shared.rs\nshared.rs:\nthird: shared.rs\n"
        )
        .unwrap(),
        ["other.rs", "shared.rs"].map(PathBuf::from)
    );
    for text in [
        "no separator",
        ": source.rs",
        "target: source.rs\nno separator",
        "target: source.rs\n: another.rs",
        "target: source.rs\nother # not a full-line comment",
        "target:",
        "target: /path/$(UNKNOWN)",
        "target: /path/\\q",
        "target: /path:",
        "target: /glob/*",
        "target: /path | generated",
        "target: /path; recipe",
        "target: /path\0",
    ] {
        assert_eq!(
            parse_cargo_depfile(text).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
}

#[test]
fn depfile_reads_and_disappearing_records_preserve_external_custody() {
    let root = unique_temp_directory("input-evidence-depfiles");
    let records = root.join("target");
    let external = root.join("external");
    fs::create_dir(&records).unwrap();
    fs::create_dir(&external).unwrap();
    fs::write(external.join("source"), b"first").unwrap();
    fs::write(records.join("generated"), b"output").unwrap();
    let record = records.join("role.d");
    fs::write(&record, "role: external target/generated\n").unwrap();
    let paths =
        read_cargo_depfile_inputs(&root, &record, std::slice::from_ref(&records), 4096).unwrap();
    assert_eq!(paths, std::slice::from_ref(&external));
    let before = capture(b"same authority", &paths);
    fs::remove_file(&record).unwrap();
    assert_eq!(
        read_cargo_depfile_inputs(&root, &record, &[], 4096)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    // The consumer may select an empty cold inventory; it cannot drop custody.
    let after = evidence_spec(b"same authority", &[]);
    before.validate_after(&after).unwrap();
    let added = external.join("added");
    fs::write(&added, b"late bytes").unwrap();
    assert!(
        matches!(before.validate_after(&after), Err(InputEvidenceError::ChangedInput { path }) if path == added)
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn selected_depfile_keeps_foreign_records_out_and_refuses_late_input_authority() {
    let root = unique_temp_directory("input-evidence-selected-record");
    let records = root.join("target");
    fs::create_dir(&records).unwrap();
    let source = root.join("source.rs");
    fs::write(&source, b"before").unwrap();
    fs::write(records.join("generated"), b"output").unwrap();
    let record = records.join("selected.d");
    fs::write(&record, "selected: source.rs target/generated\n").unwrap();
    fs::write(records.join("foreign.d"), "foreign: $(unadmitted)\n").unwrap();
    let read = || read_cargo_depfile_inputs(&root, &record, std::slice::from_ref(&records), 4096);
    let paths = read().unwrap();
    assert_eq!(paths, std::slice::from_ref(&source));
    assert!(read_cargo_depfile_inputs(&root, &record, &[], 1).is_err());
    #[cfg(unix)]
    {
        let alias = records.join("alias.d");
        std::os::unix::fs::symlink(&record, &alias).unwrap();
        assert!(read_cargo_depfile_inputs(&root, &alias, &[], 4096).is_err());
    }
    let before = capture(b"selected recipe", &paths);
    let late = root.join("late.rs");
    fs::write(&late, b"late bytes").unwrap();
    fs::write(&record, "selected: source.rs late.rs\n").unwrap();
    let after = evidence_spec(b"selected recipe", &read().unwrap());
    assert!(matches!(
        before.validate_after(&after),
        Err(InputEvidenceError::UnobservedInput { path }) if path == late
    ));
    fs::write(&record, "selected: $(unadmitted)\n").unwrap();
    assert_eq!(read().unwrap_err().kind(), io::ErrorKind::InvalidData);
    fs::remove_file(&record).unwrap();
    assert_eq!(read().unwrap_err().kind(), io::ErrorKind::NotFound);
    assert_eq!(
        read_cargo_depfile_inputs(&root, Path::new("relative.d"), &[], 4096)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidInput
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn malformed_or_oversized_depfiles_refuse_the_entire_record() {
    let root = unique_temp_directory("input-evidence-invalid-records");
    fs::write(root.join("good.d"), "role: source.rs\n").unwrap();
    fs::write(root.join("bad.d"), "role: $(unadmitted)\n").unwrap();
    assert_eq!(
        read_cargo_depfile_inputs(&root, &root.join("bad.d"), &[], 4096)
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    fs::remove_file(root.join("bad.d")).unwrap();
    assert!(read_cargo_depfile_inputs(&root, &root.join("good.d"), &[], 1).is_err());
    #[cfg(unix)]
    {
        // Syntax refusal precedes canonicalization, even when an earlier token
        // would fail with a symlink loop. No partial inventory may escape.
        std::os::unix::fs::symlink("loop", root.join("loop")).unwrap();
        fs::write(root.join("bad.d"), "role: loop\nmalformed\n").unwrap();
        assert_eq!(
            read_cargo_depfile_inputs(&root, &root.join("bad.d"), &[], 4096)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
        fs::remove_file(root.join("bad.d")).unwrap();
        std::os::unix::fs::symlink(root.join("good.d"), root.join("alias.d")).unwrap();
        assert!(read_cargo_depfile_inputs(&root, &root.join("alias.d"), &[], 4096).is_err());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn replaced_depfile_cannot_authorize_late_bytes_or_drop_old_file_custody() {
    let root = unique_temp_directory("input-evidence-replaced-record");
    let records = root.join("records");
    fs::create_dir(&records).unwrap();
    let original = root.join("original.rs");
    let late = root.join("late.rs");
    fs::write(&original, b"original").unwrap();
    let record = records.join("role.d");
    fs::write(&record, "role: original.rs\n").unwrap();
    let discover = || read_cargo_depfile_inputs(&root, &record, &[], 4096).unwrap();
    let before = capture(b"authority", &discover());
    fs::write(&late, b"late").unwrap();
    fs::write(&record, "role: late.rs\n").unwrap();
    let after = evidence_spec(b"authority", &discover());
    assert!(matches!(before.validate_after(&after),
        Err(InputEvidenceError::UnobservedInput { path }) if path == late));
    fs::write(&original, b"mutated").unwrap();
    assert!(matches!(before.validate_after(&after),
        Err(InputEvidenceError::ChangedInput { path }) if path == original));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn overlapping_directory_spellings_are_complete_not_false_absence() {
    let root = unique_temp_directory("input-evidence-aliases");
    fs::create_dir(root.join("child")).unwrap();
    fs::write(root.join("source.rs"), b"observed").unwrap();
    let alias = root.join("child/..");
    let before = capture(b"authority", &[root.clone(), alias.clone()]);
    assert!(matches!(
        before.entries()[&alias.join("source.rs")],
        InputEvidenceValue::File { .. }
    ));
    before
        .validate_after(&evidence_spec(b"authority", &[alias]))
        .unwrap();
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_generated_traversal_and_cold_records_are_conservative() {
    let root = unique_temp_directory("input-evidence-generated");
    let generated = root.join("generated");
    fs::create_dir(&generated).unwrap();
    fs::write(
        root.join("role.d"),
        "role: generated/output.rs generated/../external.rs\n",
    )
    .unwrap();
    assert_eq!(
        read_cargo_depfile_inputs(
            &root,
            &root.join("role.d"),
            std::slice::from_ref(&generated),
            4096
        )
        .unwrap(),
        [generated.join("../external.rs")]
    );
    assert_eq!(
        read_cargo_depfile_inputs(&root, &root.join("cold/role.d"), &[], 4096)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(unix)]
fn native_paths_are_preserved_and_descendant_io_retains_leaf_context() {
    use std::{
        ffi::OsString,
        os::unix::{ffi::OsStringExt as _, fs::symlink},
    };
    // macOS filesystems require valid UTF-8; retain actual native-path/leaf-error
    // checks there, and exercise Linux's invalid-byte filenames on Linux.
    let names: &[&[u8]] = &[
        b"x\xce\xbb",
        #[cfg(target_os = "linux")]
        b"x\xff",
    ];
    for name in names {
        let root = unique_temp_directory("input-evidence-native");
        let native = root.join(OsString::from_vec(name.to_vec()));
        fs::write(&native, b"native").unwrap();
        let evidence = capture(b"authority", std::slice::from_ref(&root));
        assert!(evidence.entries().contains_key(&native));
        if native.to_str().is_some() {
            evidence.digest().unwrap();
        } else {
            assert!(
                matches!(evidence.digest(), Err(InputEvidenceError::Io { path, source })
                if path == native && source.kind() == io::ErrorKind::InvalidData)
            );
        }
        let leaf = root.join("leaf-link");
        symlink(&native, &leaf).unwrap();
        assert!(
            matches!(InputEvidence::capture(&InputEvidenceSpec::new(Vec::new()).with_input(&root)),
            Err(InputEvidenceError::Io { path, source })
            if path == leaf && source.kind() == io::ErrorKind::InvalidInput)
        );
        fs::remove_dir_all(root).unwrap();
    }
}

#[test]
fn source_bytes_additions_deletions_and_renames_change_the_input_set() {
    let root = unique_temp_directory("input-evidence-source-set");
    fs::create_dir_all(root.join("src")).unwrap();
    let source = root.join("src/lib.rs");
    fs::write(&source, b"one").unwrap();
    let spec = InputEvidenceSpec::new(Vec::new())
        .with_input(&root)
        .with_excluded_entry_name("target");
    let collect = || InputEvidence::capture(&spec).unwrap().entries().clone();
    let original = collect();
    fs::write(&source, b"two").unwrap();
    assert_ne!(original, collect());
    fs::write(&source, b"one").unwrap();
    assert_eq!(original, collect());
    fs::rename(&source, root.join("src/other.rs")).unwrap();
    assert_ne!(original, collect());
    fs::rename(root.join("src/other.rs"), &source).unwrap();
    fs::write(root.join("src/include.txt"), b"new input").unwrap();
    assert_ne!(original, collect());
    fs::remove_file(root.join("src/include.txt")).unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::write(root.join("target/build-output"), b"generated").unwrap();
    assert_eq!(original, collect());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn generated_ownership_uses_resolved_paths_and_preserves_external_inputs() {
    let root = unique_temp_directory("depfile-output-ownership");
    let output = root.join("target");
    fs::create_dir_all(&output).unwrap();
    fs::create_dir_all(root.join("alias")).unwrap();
    let source = root.join("authored.json");
    fs::write(&source, "authored").unwrap();
    fs::write(output.join("translations.json"), "generated").unwrap();
    let record = output.join("role.d");
    fs::write(&record, "role: alias/../target/translations.json\n").unwrap();
    let discover = || {
        read_cargo_depfile_inputs(&root, &record, &[root.join("alias/../target")], 4096).unwrap()
    };
    assert_eq!(discover(), [] as [PathBuf; 0]);
    fs::write(&record, "role: target/../authored.json\n").unwrap();
    let paths = discover();
    assert_eq!(paths, [source.canonicalize().unwrap()]);
    let before = capture(b"authority", &paths);
    fs::write(&source, "edited").unwrap();
    assert!(matches!(
        before.validate_after(&evidence_spec(b"authority", &discover())),
        Err(InputEvidenceError::ChangedInput { .. })
    ));
    let missing = output.join("../not-yet-authored.json");
    let foreign = root.join("foreign/target/translations.json");
    fs::create_dir_all(foreign.parent().unwrap()).unwrap();
    fs::write(&foreign, "foreign").unwrap();
    fs::write(
        &record,
        "role: target/../not-yet-authored.json foreign/target/translations.json\n",
    )
    .unwrap();
    let evidence = capture(b"authority", &discover());
    assert_eq!(evidence.entries()[&missing], InputEvidenceValue::Missing);
    assert!(
        evidence
            .entries()
            .contains_key(&foreign.canonicalize().unwrap())
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn post_work_projection_preserves_selected_roots_and_missing_inventory() {
    let root = unique_temp_directory("input-evidence-projection");
    let file = root.join("source");
    fs::write(&file, b"stable").unwrap();
    let missing = root.join("absent");
    let before = capture(b"authority", &[root.clone(), missing]);
    for selected in [
        evidence_spec(b"authority", &[file]),
        evidence_spec(b"authority", std::slice::from_ref(&root)),
    ] {
        let after = before.validate_after(&selected).unwrap();
        let expected = InputEvidence::capture(&selected).unwrap();
        assert_eq!(after.entries(), expected.entries());
        assert_eq!(after.digest().unwrap(), expected.digest().unwrap());
    }
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn post_work_projection_keeps_explicit_roots_separate_from_excluded_children() {
    let root = unique_temp_directory("input-evidence-projection-excluded");
    let output = root.join("output");
    fs::create_dir(&output).unwrap();
    fs::write(output.join("source"), b"stable").unwrap();
    let selected = InputEvidenceSpec::new(b"authority".as_slice())
        .with_input(&root)
        .with_excluded_entry_name("output");
    let before = InputEvidence::capture(&selected.clone().with_input(&output)).unwrap();
    let after = before.validate_after(&selected).unwrap();
    let expected = InputEvidence::capture(&selected).unwrap();
    assert_eq!(after.entries(), expected.entries());
    assert_eq!(after.digest().unwrap(), expected.digest().unwrap());
    fs::write(output.join("source"), b"changed").unwrap();
    assert!(matches!(
        before.validate_after(&selected),
        Err(InputEvidenceError::ChangedInput { .. })
    ));
    fs::remove_dir_all(root).unwrap();
}
