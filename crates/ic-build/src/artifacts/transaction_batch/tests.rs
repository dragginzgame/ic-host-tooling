use crate::batch::BatchLabelError;
use std::{cell::Cell, fs};

use super::{
    ArtifactCacheBatchEntry, ArtifactCacheBatchFailure, LabeledArtifactCacheSpec,
    build_artifact_caches_batch,
};
use crate::artifacts::{
    ArtifactCacheOutcome, ArtifactCachePreparation, ArtifactCacheSpec, prepare_artifact_cache,
};
use crate::fs_fixture::unique_temp_directory;

#[test]
fn labeled_transactions_build_then_reuse_in_order() {
    let root = unique_temp_directory("artifact-cache-batch");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        LabeledArtifactCacheSpec::new(
            "first",
            ArtifactCacheSpec::new(&root.join("cache"), "first", "recipe/v1")
                .with_coordination_scope("shared-builder")
                .with_input("input", &input)
                .with_output("output", &root.join("first.out")),
        ),
        LabeledArtifactCacheSpec::new(
            "second",
            ArtifactCacheSpec::new(&root.join("cache"), "second", "recipe/v1")
                .with_coordination_scope("shared-builder")
                .with_input("input", &input)
                .with_output("output", &root.join("second.out")),
        ),
    ];
    let mut built_labels = Vec::new();
    let built = build_artifact_caches_batch(&specs, |label, transaction| {
        built_labels.push(label.to_owned());
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{label}"),
        )
        .expect("write batch output");
        Ok::<(), &'static str>(())
    })
    .expect("valid labeled batch");

    assert!(built.is_success());
    assert_eq!(built.entries().len(), 2);
    assert_eq!(built.entries()[0].label(), "first");
    assert_eq!(built.entries()[1].label(), "second");
    assert_eq!(built_labels, ["first", "second"]);
    assert!(built.outcomes().all(|entry| {
        matches!(entry.outcome().unwrap(), ArtifactCacheOutcome::Built(_))
            && entry.entry_elapsed() <= built.total()
    }));
    let built_metrics = built.metrics();
    assert_eq!(built_metrics.entries(), 2);
    assert_eq!(built_metrics.succeeded(), 2);
    assert_eq!(built_metrics.failed(), 0);
    assert_eq!(built_metrics.built(), 2);
    assert_eq!(built_metrics.reused(), 0);
    assert!(built_metrics.successful_timings().caller_build().is_some());

    let reused = build_artifact_caches_batch(&specs, |_label, _transaction| {
        Err::<(), _>("unexpected cache miss")
    })
    .expect("valid reused batch");
    assert!(reused.is_success());
    assert!(
        reused
            .outcomes()
            .all(|entry| entry.outcome().unwrap().is_reused())
    );
    let reused_metrics = reused.metrics();
    assert_eq!(reused_metrics.succeeded(), 2);
    assert_eq!(reused_metrics.failed(), 0);
    assert_eq!(reused_metrics.built(), 0);
    assert_eq!(reused_metrics.reused(), 2);
    assert!(reused_metrics.successful_timings().caller_build().is_none());
    fs::remove_dir_all(root).expect("remove artifact batch fixture");
}

#[test]
fn callback_failure_retains_label_timings_and_later_transaction() {
    let root = unique_temp_directory("artifact-cache-batch-builder-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        artifact_spec(&root, &input, "second", "second.out"),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |label, transaction| {
        if label == "second" {
            return Err("synthetic builder failure".to_owned());
        }
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{label}"),
        )
        .expect("write successful batch output");
        Ok(())
    })
    .expect("valid labeled batch");

    assert!(!report.is_success());
    assert_eq!(report.entries().len(), 3);
    assert_eq!(
        report
            .outcomes()
            .map(ArtifactCacheBatchEntry::label)
            .collect::<Vec<_>>(),
        ["first", "third"]
    );
    for entry in report.outcomes().chain(report.failures()) {
        assert!(std::ptr::eq(
            entry,
            &raw const report.entries()[entry.index()]
        ));
    }
    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    let failure = &failures[0];
    assert_eq!(failure.index(), 1);
    assert_eq!(failure.label(), "second");
    assert_eq!(
        failure.entry_elapsed(),
        failure.failure().unwrap().timings().total()
    );
    assert!(failure.failure().unwrap().timings().callback().is_some());
    assert!(failure.failure().unwrap().timings().cleanup().is_some());
    assert!(failure.failure().unwrap().timings().commit().is_none());
    assert!(matches!(
        failure.failure().unwrap(),
        ArtifactCacheBatchFailure::Callback {
            cleanup_error: None,
            ..
        }
    ));
    assert!(!root.join("second.out").exists());
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-third");
    let metrics = report.metrics();
    assert_eq!(metrics.succeeded(), 2);
    assert_eq!(metrics.failed(), 1);
    assert_eq!(metrics.built(), 2);

    let preparation = prepare_artifact_cache(specs[1].spec()).expect("prepare failed entry again");
    let ArtifactCachePreparation::Build(transaction) = preparation else {
        panic!("failed batch entry must not be published");
    };
    transaction.abort().expect("abort verification transaction");
    fs::remove_dir_all(root).expect("remove artifact batch failure fixture");
}

#[test]
#[cfg(unix)]
fn callback_and_abort_failures_retain_both_causes_and_continue() {
    let root = unique_temp_directory("artifact-cache-batch-abort-failure");
    let input = root.join("input");
    fs::write(&input, b"input").unwrap();
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        artifact_spec(&root, &input, "second", "second.out"),
    ];
    let mut retained_stage = None;
    let report = build_artifact_caches_batch(&specs, |label, transaction| {
        if label == "first" {
            let stage = transaction.staging_directory().to_owned();
            let parent = stage.parent().unwrap();
            let retained = root.join("retained-staging");
            fs::rename(parent, &retained).unwrap();
            fs::write(
                parent,
                b"obstruct abort without relying on process permissions",
            )
            .unwrap();
            retained_stage = Some(retained.join(stage.file_name().unwrap()));
            return Err("primary callback failure");
        }
        fs::write(transaction.output_path("output").unwrap(), b"later output").unwrap();
        Ok(())
    });
    let report = report.unwrap();
    let failed = report.failures().next().unwrap();
    let ArtifactCacheBatchFailure::Callback {
        source,
        cleanup_error: Some(cleanup),
        timings,
    } = failed.failure().unwrap()
    else {
        panic!("callback failure must retain the separate abort error");
    };
    assert_eq!(**source, "primary callback failure");
    let crate::artifacts::ArtifactCacheError::Io { source, .. } = cleanup.as_ref() else {
        panic!("abort must retain its original filesystem error");
    };
    assert_eq!(source.kind(), std::io::ErrorKind::NotADirectory);
    assert_eq!(failed.entry_elapsed(), timings.total());
    assert!(timings.callback().is_some());
    assert!(timings.cleanup().is_some());
    assert!(timings.commit().is_none());
    assert!(retained_stage.unwrap().exists());
    assert_eq!(fs::read(root.join("second.out")).unwrap(), b"later output");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn preparation_failure_is_labeled_timed_and_does_not_stop_later_entries() {
    let root = unique_temp_directory("artifact-cache-batch-cache-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        LabeledArtifactCacheSpec::new(
            "invalid",
            ArtifactCacheSpec::new(&root.join("cache"), "", "recipe/v1")
                .with_input("input", &input)
                .with_output("output", &root.join("invalid.out")),
        ),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |label, transaction| {
        fs::write(
            transaction
                .output_path("output")
                .expect("batch output path"),
            format!("output-{label}"),
        )
        .expect("write successful batch output");
        Ok::<(), &'static str>(())
    })
    .expect("valid labeled batch");

    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    let failure = &failures[0];
    assert_eq!(failure.index(), 1);
    assert_eq!(failure.label(), "invalid");
    assert_eq!(
        failure.entry_elapsed(),
        failure.failure().unwrap().timings().total()
    );
    assert!(failure.failure().unwrap().timings().callback().is_none());
    assert!(failure.failure().unwrap().timings().cleanup().is_none());
    assert!(failure.failure().unwrap().timings().commit().is_none());
    assert!(matches!(
        failure.failure().unwrap(),
        ArtifactCacheBatchFailure::Preparation { .. }
    ));
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-third");
    fs::remove_dir_all(root).expect("remove artifact cache failure fixture");
}

#[test]
fn commit_failure_is_labeled_timed_and_does_not_stop_later_entries() {
    let root = unique_temp_directory("artifact-cache-batch-commit-failure");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let specs = [
        artifact_spec(&root, &input, "first", "first.out"),
        artifact_spec(&root, &input, "second", "second.out"),
        artifact_spec(&root, &input, "third", "third.out"),
    ];
    let report = build_artifact_caches_batch(&specs, |label, transaction| {
        if label != "second" {
            fs::write(
                transaction
                    .output_path("output")
                    .expect("batch output path"),
                format!("output-{label}"),
            )
            .expect("write successful batch output");
        }
        Ok::<(), &'static str>(())
    })
    .expect("valid labeled batch");

    let failures = report.failures().collect::<Vec<_>>();
    assert_eq!(failures.len(), 1);
    let failure = &failures[0];
    assert_eq!(failure.label(), "second");
    assert_eq!(
        failure.entry_elapsed(),
        failure.failure().unwrap().timings().total()
    );
    assert!(failure.failure().unwrap().timings().callback().is_some());
    assert!(failure.failure().unwrap().timings().cleanup().is_none());
    assert!(failure.failure().unwrap().timings().commit().is_some());
    assert!(matches!(
        failure.failure().unwrap(),
        ArtifactCacheBatchFailure::Commit { .. }
    ));
    assert!(!root.join("second.out").exists());
    assert_eq!(fs::read(root.join("third.out")).unwrap(), b"output-third");
    fs::remove_dir_all(root).expect("remove artifact cache commit failure fixture");
}

#[test]
fn batch_rejects_empty_or_duplicate_labels_before_work() {
    let root = unique_temp_directory("artifact-cache-batch-label-contract");
    let input = root.join("input");
    fs::write(&input, b"input").expect("write batch input");
    let callback_ran = Cell::new(false);
    let empty = [artifact_spec(&root, &input, "", "empty.out")];
    let error = build_artifact_caches_batch(&empty, |_label, _transaction| {
        callback_ran.set(true);
        Ok::<(), &'static str>(())
    })
    .expect_err("empty label must fail");
    assert_eq!(error, BatchLabelError::Empty { index: 0 });
    assert!(!callback_ran.get());

    let duplicate = [
        artifact_spec(&root, &input, "same", "first.out"),
        LabeledArtifactCacheSpec::new(
            "same",
            ArtifactCacheSpec::new(&root.join("cache"), "other", "recipe/v1")
                .with_input("input", &input)
                .with_output("output", &root.join("second.out")),
        ),
    ];
    let error = build_artifact_caches_batch(&duplicate, |_label, _transaction| {
        callback_ran.set(true);
        Ok::<(), &'static str>(())
    })
    .expect_err("duplicate label must fail");
    assert_eq!(
        error,
        BatchLabelError::Duplicate {
            label: "same".to_owned(),
            first_index: 0,
            duplicate_index: 1,
        }
    );
    assert!(!callback_ran.get());
    fs::remove_dir_all(root).expect("remove artifact cache label fixture");
}

fn artifact_spec(
    root: &std::path::Path,
    input: &std::path::Path,
    label: &str,
    output: &str,
) -> LabeledArtifactCacheSpec {
    LabeledArtifactCacheSpec::new(
        label,
        ArtifactCacheSpec::new(&root.join("cache"), label, "recipe/v1")
            .with_input("input", input)
            .with_output("output", &root.join(output)),
    )
}
