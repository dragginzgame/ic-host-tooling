use std::{fs, os::unix::fs::PermissionsExt as _, process::Command, thread};

use super::write_executable_script;
use crate::test_support::Fixture;

#[test]
fn parallel_writers_return_complete_immediately_executable_scripts() {
    let fixture = Fixture::new();
    thread::scope(|scope| {
        for index in 0..8 {
            let path = fixture.path().join(format!("script-{index}"));
            scope.spawn(move || {
                for iteration in 0..8 {
                    let value = format!("{index}-{iteration}");
                    let contents = format!(
                        "#!/bin/sh\nprintf '%s' '{value}'\n# {}\n",
                        "x".repeat(128 * 1024)
                    );
                    write_executable_script(&path, &contents).unwrap();
                    assert_eq!(fs::read(&path).unwrap(), contents.as_bytes());
                    assert_eq!(
                        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                        0o755
                    );
                    let output = Command::new(&path).output().unwrap();
                    assert!(output.status.success());
                    assert_eq!(output.stdout, value.as_bytes());
                }
            });
        }
    });
}

#[test]
fn missing_parent_is_refused_without_creating_directories() {
    let fixture = Fixture::new();
    let path = fixture.path().join("absent/script");
    assert!(write_executable_script(&path, b"#!/bin/sh\nexit 0\n").is_err());
    assert!(!path.exists());
    assert!(!fixture.path().join("absent").exists());
    drop(fixture);
}
