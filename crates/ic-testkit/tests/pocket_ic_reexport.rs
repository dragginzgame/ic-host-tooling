use ic_testkit::{
    pic,
    pocket_ic::{
        CanisterSettings, CreateCanisterParams, PocketIc,
        common::rest::{BlobCompression, IcpFeatures, IcpFeaturesConfig},
    },
};

#[test]
fn platform_client_surface_is_available_through_ic_testkit() {
    fn exported<T>() {}

    exported::<CanisterSettings>();
    exported::<CreateCanisterParams>();
    exported::<BlobCompression>();
    exported::<IcpFeatures>();
    exported::<IcpFeaturesConfig>();

    let upstream: Option<ic_platform::pocket_ic::PocketIc> = None;
    let convenience: Option<pic::PocketIc> = upstream;
    let _: Option<PocketIc> = convenience;
}

#[test]
fn async_client_and_rest_types_retain_platform_identity() {
    let client: Option<ic_platform::pocket_ic::nonblocking::PocketIc> = None;
    let _: Option<ic_testkit::pocket_ic::nonblocking::PocketIc> = client;

    let response: Option<ic_platform::pocket_ic::common::rest::CreateInstanceResponse> = None;
    let _: Option<ic_testkit::pocket_ic::common::rest::CreateInstanceResponse> = response;
}
