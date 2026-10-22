// Each integration target uses only the fixtures it needs.
#![allow(dead_code)]

pub mod wait;

pub fn startup_config() -> ic_testkit::pic::PocketIcStartupConfig {
    ic_testkit::pic::PocketIcStartupConfig::from_env(std::time::Duration::from_secs(30))
        .expect("prepare POCKET_IC_BIN or configure IC_TESTKIT_POCKET_IC_URL for live tests")
}

pub fn pocket_ic() -> ic_testkit::pic::PocketIcInstance<ic_platform::pocket_ic::PocketIc> {
    startup_config()
        .build_instance(ic_platform::pocket_ic::PocketIcBuilder::new().with_application_subnet())
        .expect("construct explicitly configured test instance")
}
