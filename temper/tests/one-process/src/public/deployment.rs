//! Opening figures and start refusals (jig/docs/design/protocol/configuration.md, 6).
use crate::{
    fixtures, stories,
    world::{self, Story},
};
use skein_io::kernel::Fd;

fn start(value: &serde_json::Value) -> Result<temper_service::Service, String> {
    let mut document = value.clone();
    document.as_object_mut().expect("deployment object").remove("world_settings");
    temper_service::startup::with_setup_given(
        &serde_json::to_vec(&document).expect("deployment JSON"),
        &jig_protocol_config::Directories::working(),
        &fixtures::files(),
        Some(&temper_playground::documents::agent()),
        &fixtures::seeds(),
        None,
        Some(temper_service::WorkerRuntime { root: Fd::new(3), path: b"/worker".as_slice().into() }),
        temper_playground::documents::forges(),
        None,
        None,
        &stories::given(&document),
    )
    .map_err(|problems| String::from_utf8_lossy(&temper_service::startup::lines(&problems)).into_owned())
}

pub(super) fn relations() {
    super::ACTIVE.with(|active| active.set(true));
    stories::set(Story::Chat { words: b"Hello" });
    stories::setup_settings(world::Phase::Restart);
    let value = fixtures::world_document();
    start(&value).unwrap_or_else(|problem| {
        panic!("the specified public deployment must start before testing broken relations: {problem}")
    });
    for (name, broken) in [
        ("clients.public.rate_burst", 7),
        ("clients.public.bytes_burst", 524_287),
        ("clients.public.answer_bytes", 524_289),
        ("clients_http.public_calls", 5),
        ("clients.public.max_age", 61),
        ("clients.public.answer_bytes", 1),
        ("core.public.lines", 257),
    ] {
        let mut bad = value.clone();
        bad["engine"]["limits"]["overrides"][name] = broken.into();
        let Err(problem) = start(&bad) else {
            panic!("broken relation {name}={broken} started");
        };
        assert!(problem.contains(name), "broken relation {name} must name its path: {problem}");
    }
    super::ACTIVE.with(|active| active.set(false));
}
