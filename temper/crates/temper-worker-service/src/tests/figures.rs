#![expect(clippy::disallowed_methods, reason = "ordinary tests edit fixture text and inspect diagnostics")]
use super::fixtures;
use crate::{figures, startup};
use jig_protocol_config as config;
use jig_protocol_worker_wire as w;
use skein_lib::{Duration, List};
fn record(places: u32, values: &[(&str, u64)]) -> w::Figures {
    let mut limits = List::with_capacity(u32::try_from(values.len()).unwrap());
    for (name, value) in values {
        limits
            .push(w::Limit::new(&w::CEILINGS, w::LimitParts { name: name.as_bytes().into(), value: *value }).unwrap())
            .unwrap();
    }
    w::Figures::new(
        &w::CEILINGS,
        w::FiguresParts {
            places,
            limits,
            agents: w::Agents::new(
                &w::CEILINGS,
                w::AgentsParts { profile: List::with_capacity(0), endpoints: List::with_capacity(0) },
            )
            .unwrap(),
        },
    )
    .unwrap()
}
#[test]
fn figures_apply_named_limits_cap_slots_and_skip_unknown_names_once() {
    let mut source = jig_protocol_config_fixtures::small();
    source.hub.slots = 9;
    source.link.ping = Duration::from_millis(4321);
    let mut own = temper_limits_fixtures::shared();
    own.checkout.workspaces = 7;
    let mut values = config::overrides::list(&source).into_boxed().into_vec();
    values.extend(temper_service_core::limits::list(&own).into_boxed().into_vec());
    values.extend([("future.limit", 99), ("future.limit", 100)]);
    let mut part = startup::worker_part(&fixtures::worker_alone_document(), &config::Directories::working()).unwrap();
    part.worker.jig.slots = 4;
    let unknown = figures::apply(&mut part, record(3, &values)).unwrap();
    assert_eq!(unknown.into_boxed().as_ref(), [Box::<[u8]>::from(b"future.limit".as_slice())]);
    assert_eq!(part.worker.jig.slots, 3);
    assert_eq!(config::overrides::list(part.worker.jig.limits.as_ref().unwrap()), config::overrides::list(&source));
    assert_eq!(part.limits, crate::worker_settings::Limits::shared(own));
    assert_eq!(
        crate::worker_settings::root_limits(&part.worker, &fixtures::startup_agent(), &part.limits).host.slots,
        3
    );
}
#[test]
fn each_moved_member_is_refused_with_its_migration_words() {
    let base = fixtures::worker_alone_document();
    for (needle, addition, path, words) in [
        (
            "\"workers\":[{",
            "\"workers\":[{\"limits\":null,",
            config::Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"limits"),
            startup::worker::MOVED_LIMITS,
        ),
        (
            "\"workers\":[{",
            "\"workers\":[{\"agent\":null,",
            config::Path::root().member(b"workers").child(config::Segment::Index(0)).member(b"agent"),
            startup::worker::MOVED_AGENT,
        ),
        (
            "\"application\":{",
            "\"application\":{\"limits\":null,",
            config::Path::root().member(b"application").member(b"limits"),
            startup::worker::MOVED_LIMITS,
        ),
    ] {
        let text = core::str::from_utf8(&base).unwrap().replacen(needle, addition, 1);
        let found = startup::worker_part(text.as_bytes(), &config::Directories::working()).unwrap_err();
        assert_eq!(found.problems.len(), 1);
        assert_eq!(found.problems.get(0).unwrap().path, path);
        assert_eq!(found.problems.get(0).unwrap().what, startup::What::Jig(config::What::Because(words)));
    }
}
#[test]
fn machine_members_check_absolute_claude_and_unique_environment_names() {
    for (entry, member) in [
        (br#"{"root":"worker","git":null,"claude":"claude"}"#.as_slice(), b"claude".as_slice()),
        (br#"{"root":"worker","git":null,"claude":null}"#, b"claude"),
        (
            br#"{"root":"worker","git":null,"environment":[{"name":"A","value":"one"},{"name":"A","value":"two"}]}"#,
            b"environment",
        ),
        (br#"{"root":"worker","git":null,"environment":{}}"#, b"environment"),
        (br#"{"root":"worker","git":null,"environment":[{"name":"A"}]}"#, b"environment"),
    ] {
        let input = fixtures::with(&fixtures::worker_alone_document(), b"application.workers[0]", entry);
        let found = startup::worker_part(&input, &config::Directories::working()).expect_err("invalid member");
        let prefix = config::Path::root()
            .member(b"application")
            .member(b"workers")
            .child(config::Segment::Index(0))
            .member(member);
        assert!(found.problems.get(0).unwrap().path.segments.iter().zip(prefix.segments.iter()).all(|(a, b)| a == b));
    }
    let input = fixtures::with(
        &fixtures::worker_alone_document(),
        b"application.workers[0]",
        br#"{"root":"worker","git":null,"claude":"/bin/claude","environment":[{"name":"A","value":"one"}]}"#,
    );
    let part = startup::worker_part(&input, &config::Directories::working()).unwrap();
    assert_eq!(part.claude.as_deref(), Some(b"/bin/claude".as_slice()));
    assert_eq!(part.environment.as_ref(), [(Box::from(b"A".as_slice()), Box::from(b"one".as_slice()))]);
}
#[test]
fn changed_figures_are_not_applied_and_repeated_records_are_quiet() {
    let mut worker = fixtures::worker();
    let original = record(3, &[("hub.turns", 1)]);
    let changed = record(2, &[("hub.turns", 2)]);
    worker.built_from(original.clone());
    let limits = worker.env.limits.clone();
    for figures in [original.clone(), changed.clone(), changed.clone(), original, changed] {
        worker.link(fixtures::NOW, jig_protocol_worker_link::Request::Figures { figures });
    }
    assert_eq!(worker.env.limits, limits);
    assert_eq!(worker.notes().pop(), Some(crate::worker_alone::Note::FiguresChanged));
    assert_eq!(worker.notes().pop(), Some(crate::worker_alone::Note::FiguresChanged));
    assert_eq!(worker.notes().pop(), None);
}
#[test]
fn a_short_figure_names_the_engine_scalar_and_keeps_the_check() {
    let mut part = fixtures::part(&fixtures::worker_alone_document());
    let mut values = config::overrides::list(part.worker.jig.limits.as_ref().unwrap()).into_boxed().into_vec();
    for (name, value) in &mut values {
        if *name == "hub.turn_bytes" {
            *value = 1;
        }
    }
    figures::apply(&mut part, record(1, &values)).unwrap();
    let found =
        startup::worker::check_worker_part(&mut part, &fixtures::files(), &fixtures::startup_agent()).unwrap_err();
    let lines = startup::lines(&found);
    let text = core::str::from_utf8(&lines).unwrap();
    assert!(text.contains("the engine's figures: hub.turn_bytes 1:"), "{text}");
    assert!(text.contains("the agent's largest turn"), "{text}");
}
