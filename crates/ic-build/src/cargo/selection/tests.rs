use super::*;

#[test]
fn typed_features_keep_build_and_metadata_resolution_in_step() {
    let selected = CargoSelection {
        profile: Some("release".into()),
        features: CargoFeatureSelection {
            features: ["zeta".into(), "alpha".into()].into(),
            default_features: false,
            all_features: true,
        },
        locked: true,
        offline: true,
    };
    let metadata = selected.metadata_arguments();
    assert_eq!(
        metadata,
        [
            "--locked",
            "--offline",
            "--all-features",
            "--no-default-features",
            "--features=alpha,zeta"
        ]
        .map(OsString::from)
    );
    assert_eq!(
        selected.build_arguments(),
        [vec![OsString::from("--release")], metadata].concat()
    );
    assert_eq!(selected.profile_target_dir(), Ok("release"));
    assert_eq!(
        CargoSelection::default().metadata_arguments(),
        [] as [OsString; 0]
    );
    assert_eq!(
        CargoSelection::default().build_arguments(),
        [] as [OsString; 0]
    );
}

#[test]
fn option_shaped_values_cannot_override_the_recipe() {
    let selected = CargoSelection {
        profile: Some("--help".into()),
        features: CargoFeatureSelection {
            features: ["--target-dir=elsewhere".into()].into(),
            ..Default::default()
        },
        ..Default::default()
    };
    assert_eq!(
        selected.build_arguments(),
        ["--profile=--help", "--features=--target-dir=elsewhere"].map(OsString::from)
    );
}
