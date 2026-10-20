use super::*;

const MANIFEST: &str = "# é [workspace.package]\r\n[workspace.package]\r\nversion = \"0.1.0\" # keep\r\n[workspace.dependencies]\r\nrenamed = { package = 'core', path = 'core', version = '^0.1', optional = true }\r\nexternal = '0.1.0'";
fn changes() -> [WorkspaceVersionProjection<'static>; 2] {
    ["app", "core"].map(|package| WorkspaceVersionProjection {
        package,
        previous: "0.1.0",
        next: "0.2.0",
    })
}
const LOCK: &str = "# preserve\nversion = 4\n[[package]]\nname = 'app'\nversion = \"0.1.0\"\ndependencies = [\n \"core 0.1.0\", # local\n \"core 0.1.0 (registry+https://example.invalid)\",\n]\n[[package]]\nname = 'core'\nversion = \"0.1.0\"\n[[package]]\nname = 'core'\nversion = \"0.1.0\"\nsource = 'registry+https://example.invalid'\nchecksum = 'fixed'\n";

#[test]
fn spans_preserve_every_unselected_manifest_byte() {
    let dependencies = [DependencyVersionProjection {
        alias: "renamed",
        previous: "^0.1",
        next: "^0.2",
    }];
    let rendered = render_workspace_manifest(MANIFEST, "0.1.0", "0.2.0", &dependencies).unwrap();
    assert_eq!(
        rendered,
        MANIFEST
            .replacen("version = \"0.1.0\"", "version = \"0.2.0\"", 1)
            .replace("version = '^0.1'", "version = \"^0.2\"")
    );
    assert!(rendered.ends_with("external = '0.1.0'"));
    assert!(rendered.contains("\r\n"));
}

#[test]
fn toml_owns_quoted_dotted_and_nested_literal_boundaries() {
    let input = "description='''\n[workspace.package]\nversion = \"wrong\"\n'''\n['workspace'.'package']\nversion = '0.1.0'\n[workspace.dependencies.'renamed.key']\nversion='~0.1'\npath='core'\n";
    let rendered = render_workspace_manifest(
        input,
        "0.1.0",
        "0.2.0",
        &[DependencyVersionProjection {
            alias: "renamed.key",
            previous: "~0.1",
            next: "~0.2",
        }],
    )
    .unwrap();
    assert_eq!(
        rendered,
        input
            .replace("version = '0.1.0'", "version = \"0.2.0\"")
            .replace("version='~0.1'", "version=\"~0.2\"")
    );
}

#[test]
fn malformed_conflicting_or_duplicate_manifest_selections_return_no_candidate() {
    let dependency = DependencyVersionProjection {
        alias: "renamed",
        previous: "^0.1",
        next: "^0.2",
    };
    for input in [
        "broken [",
        "[workspace.package]\nversion='0.1.0'\nversion='0.1.0'",
        "[workspace.package]\nversion=12",
        "[workspace.package]\nversion='0.1.1'",
    ] {
        assert!(render_workspace_manifest(input, "0.1.0", "0.2.0", &[]).is_err());
    }
    for dependencies in [
        vec![dependency, dependency],
        vec![DependencyVersionProjection {
            previous: "^0.0",
            ..dependency
        }],
        vec![DependencyVersionProjection {
            alias: "missing",
            ..dependency
        }],
    ] {
        assert!(render_workspace_manifest(MANIFEST, "0.1.0", "0.2.0", &dependencies).is_err());
    }
    for next in ["01.0.0", "0.2.0-beta", "0.2.0\n", "0.2", ""] {
        assert!(render_workspace_manifest(MANIFEST, "0.1.0", next, &[]).is_err());
    }
    let huge = "999999999999999999999999999999999999999.0.0";
    assert!(
        render_workspace_manifest(MANIFEST, "0.1.0", huge, &[])
            .unwrap()
            .contains(huge)
    );
}

#[test]
fn lock_edits_only_selected_local_versions_and_exact_unqualified_references() {
    let rendered = render_workspace_lock(LOCK, &changes()).unwrap();
    let expected = LOCK
        .replacen("version = \"0.1.0\"", "version = \"0.2.0\"", 2)
        .replace("\"core 0.1.0\",", "\"core 0.2.0\",");
    assert_eq!(rendered, expected);
    verify_workspace_release_projection(LOCK, &rendered, &changes()).unwrap();
    assert!(
        super::super::lock_projection::verify_workspace_version_projection(
            LOCK,
            &rendered,
            &changes()
        )
        .is_err()
    );
    for corruption in [
        rendered.replace("fixed", "changed"),
        rendered.replace(
            "registry+https://example.invalid",
            "git+https://other.invalid",
        ),
        rendered.replace("core 0.2.0", "core 0.3.0"),
        rendered.replace("version = 4", "version = 3"),
    ] {
        assert!(verify_workspace_release_projection(LOCK, &corruption, &changes()).is_err());
    }
}

#[test]
fn lock_rejects_missing_duplicate_wrong_prior_and_unsupported_inputs() {
    for input in [
        "not TOML [".to_owned(),
        LOCK.replace("version = 4", "version = 2"),
        format!("{LOCK}\n[[package]]\nname='app'\nversion='0.1.0'\n"),
        LOCK.replace("name = 'app'", "name = 'absent'"),
    ] {
        assert!(render_workspace_lock(&input, &changes()).is_err());
    }
    let change = changes()[0];
    for projections in [
        vec![],
        vec![change, change],
        vec![WorkspaceVersionProjection {
            previous: "0.0.9",
            ..change
        }],
        vec![WorkspaceVersionProjection {
            next: "0.2.0-beta",
            ..change
        }],
        vec![WorkspaceVersionProjection {
            next: "0.1.0",
            ..change
        }],
    ] {
        assert!(render_workspace_lock(LOCK, &projections).is_err());
    }
}

#[test]
fn source_qualified_same_name_references_are_immutable() {
    let rendered = render_workspace_lock(LOCK, &changes()).unwrap();
    assert!(rendered.contains("core 0.1.0 (registry+https://example.invalid)"));
    assert!(
        verify_workspace_release_projection(
            LOCK,
            &rendered.replace("core 0.1.0 (registry+", "core 0.2.0 (registry+"),
            &changes()
        )
        .is_err()
    );
}
