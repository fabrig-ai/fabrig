#![expect(clippy::disallowed_methods, reason = "ordinary tests edit fixture text and inspect diagnostics")]
use super::fixtures;
use crate::{figures, startup, worker_alone::Note};
use jig_protocol_config as config;
use jig_protocol_worker_wire as w;
use skein_lib::{Duration, List};
fn record(places: u32, values: &[(&str, u64)]) -> w::Figures {
    let mut limits = List::with_capacity(u32::try_from(values.len()).unwrap());
    for (name, value) in values {
        limits.push(w::Limit { name: name.as_bytes().into(), value: *value }).unwrap();
    }
    w::Figures {
        places,
        limits,
        agents: w::Agents { profile: List::with_capacity(0), endpoints: List::with_capacity(0) },
    }
}
#[test]
fn figures_apply_named_limits_and_skip_unknown_names_once() {
    let mut source = jig_protocol_config_fixtures::small();
    source.hub.slots = 9;
    source.link.ping = Duration::from_millis(4321);
    let mut own = temper_limits_fixtures::shared();
    own.checkout.workspaces = 7;
    let mut values = config::overrides::list(&source).into_boxed().into_vec();
    values.extend(temper_service_core::limits::list(&own).into_boxed().into_vec());
    values.extend([("future.limit", 99), ("future.limit", 100)]);
    let mut part =
        startup::worker_part(&fixtures::worker_alone_document(), &config::Directories::working(), None, &[]).unwrap();
    part.worker.jig.slots = 4;
    let unknown = figures::apply(&mut part, record(3, &values)).unwrap();
    assert_eq!(unknown.into_boxed().as_ref(), [Box::<[u8]>::from(b"future.limit".as_slice())]);
    assert_eq!(part.worker.jig.slots, 3);
    source.hub.slots = 3;
    source.hub.source_interval = part.worker.jig.check_interval;
    own.checkout.workspaces = 3;
    own.git_component.operations = 3;
    assert_eq!(config::overrides::list(part.worker.jig.limits.as_ref().unwrap()), config::overrides::list(&source));
    assert_eq!(part.limits, crate::worker_settings::Limits::shared(own));
    assert_eq!(
        crate::worker_settings::root_limits(&part.worker, &fixtures::startup_agent(), &part.limits).host.slots,
        3
    );
}
#[test]
fn figures_places_replace_document_slots_and_size_worker_capacity() {
    for (slots, places) in [(1_u32, 6), (12, 3)] {
        let input =
            fixtures::with(&fixtures::worker_alone_document(), b"workers[0].slots", slots.to_string().as_bytes());
        let mut part = startup::worker_part(&input, &config::Directories::working(), None, &[]).unwrap();
        assert_eq!(part.worker.jig.slots, slots);
        figures::apply(&mut part, record(places, &[])).unwrap();
        assert_eq!(part.worker.jig.slots, places);
        assert_eq!(part.worker.jig.limits.as_ref().unwrap().hub.slots, places);
        assert_eq!(part.limits.checkout.workspaces, places);
        assert_eq!(part.limits.git_component.operations, places);
        let root = crate::worker_settings::root_limits(&part.worker, &fixtures::startup_agent(), &part.limits);
        assert_eq!(root.host.slots, places);
        assert_eq!(root.agent.agents, places);
        assert_eq!(root.tooling.jobs, places);
        let payload = crate::worker_settings::payload_limits(&part.limits).unwrap();
        let link = crate::worker_settings::link_limits(&part.worker.jig, &payload, part.limits.peer.forges).unwrap();
        assert_eq!(link.sizes.slots, places);
    }
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
            "\"application\":{\"limits\":null",
            config::Path::root().member(b"application").member(b"limits"),
            startup::worker::MOVED_LIMITS,
        ),
    ] {
        let text = core::str::from_utf8(&base).unwrap().replacen(needle, addition, 1);
        let found = startup::worker_part(text.as_bytes(), &config::Directories::working(), None, &[]).unwrap_err();
        assert_eq!(found.problems.len(), 1);
        assert_eq!(found.problems.get(0).unwrap().path, path);
        assert_eq!(found.problems.get(0).unwrap().what, startup::What::Jig(config::What::Because(words)));
    }
}
#[test]
#[expect(clippy::disallowed_macros, reason = "the migration stories construct supplied machine members as JSON")]
fn application_workers_is_refused_as_a_whole() {
    let mut input: serde_json::Value = serde_json::from_slice(&fixtures::worker_alone_document()).unwrap();
    input["application"]["workers"] = serde_json::json!([
        {"root":"worker","git":null,"claude":"/bin/claude","environment":[{"name":"A","value":"one"}]}
    ]);
    let found = startup::worker_part(&serde_json::to_vec(&input).unwrap(), &config::Directories::working(), None, &[])
        .unwrap_err();
    assert_eq!(found.problems.len(), 1);
    assert_eq!(found.problems.get(0).unwrap().path, config::Path::root().member(b"application").member(b"workers"));
    assert_eq!(found.problems.get(0).unwrap().what, startup::What::Jig(config::What::Fixed));
}

#[test]
fn next_start_figures_wait_and_repeated_records_are_quiet() {
    let original = stock_figures();
    let mut worker = stock_service(&original).worker;
    let limits = worker.env.limits.clone();
    let git = *worker.git.component.limits();
    let mut turn = original.clone();
    set_limit(&mut turn, "hub.turn_bytes", 2_097_152);
    let mut places = original.clone();
    places.places = 5;
    let mut agent = original.clone();
    agent.agents.profile.iter_mut().find(|row| row.name.as_ref() == b"declared.tool_payload").unwrap().value = 63_000;
    for (changed, name, value) in [
        (turn, "hub.turn_bytes", 2_097_152),
        (places, "places", 5),
        (agent, "agents.profile.declared.tool_payload", 63_000),
    ] {
        hear(&mut worker, changed.clone());
        assert_eq!(worker.notes().pop(), Some(Note::FiguresWaiting { figures: Box::new([scalar(name, value)]) }));
        hear(&mut worker, changed);
        hear(&mut worker, original.clone());
        assert!(worker.notes().is_empty(), "repeated record and return to the figures in force");
        assert_eq!(worker.env.limits, limits);
        assert_eq!(*worker.git.component.limits(), git);
    }
}

pub(crate) fn scalar(name: &str, value: u64) -> w::Limit {
    w::Limit { name: name.as_bytes().into(), value }
}

pub(crate) fn set_limit(figures: &mut w::Figures, name: &str, value: u64) {
    if let Some(figure) = figures.limits.iter_mut().find(|row| row.name.as_ref() == name.as_bytes()) {
        figure.value = value;
    } else {
        let mut limits = List::uncapped();
        for figure in &figures.limits {
            limits.push(figure.clone()).unwrap();
        }
        limits.push(scalar(name, value)).unwrap();
        figures.limits = limits;
    }
}

fn hear(worker: &mut crate::Worker, figures: w::Figures) {
    worker.link(fixtures::NOW, jig_protocol_worker_link::Request::Figures { figures });
}

fn stock_service(figures: &w::Figures) -> crate::worker_alone::Service {
    let (part, agent) = stock_worker(figures.clone());
    startup::worker_alone(part, &fixtures::files(), &agent, fixtures::worker_runtime(), 7).unwrap()
}

// figures.md, 9: take at least one accepted move for every live row and check
// both sides of the classification, using the published rows as the oracle.
#[test]
fn the_names_tables_and_the_workers_takes_agree() {
    use temper_engine_service_startup::limits as engine;
    let base = stock_figures();
    let (part, mut agent) = stock_worker(base.clone());
    // Stock's hub event bytes exactly fit both the pinned hello and the
    // agent's message ceiling. Leave one byte of headroom above the hello
    // so this row can move alone without changing the profile it checks.
    agent.host_limits.message_bytes = agent.host_limits.message_bytes.checked_add(1).unwrap();
    let mut worker =
        startup::worker_alone(part, &fixtures::files(), &agent, fixtures::worker_runtime(), 7).unwrap().worker;
    let jig = config::stock();
    let own = engine::stock();
    for (name, applies, default, known) in config::names::NAMES
        .iter()
        .map(|row| (row.name, row.applies, (row.value)(&jig), true))
        .chain(engine::NAMES.iter().map(|row| {
            (row.name, row.applies, (row.value)(&own), temper_service_core::limits::known(row.name.as_bytes()))
        }))
    {
        let stock =
            base.limits.iter().find(|row| row.name.as_ref() == name.as_bytes()).map_or(default, |row| row.value);
        check_row(&mut worker, &base, name, stock, applies, known);
    }
    check_row(&mut worker, &base, "future.worker_limit", 0, config::names::Applies::NextStart, false);
}

fn check_row(
    worker: &mut crate::Worker,
    base: &w::Figures,
    name: &str,
    stock: u64,
    applies: config::names::Applies,
    known: bool,
) {
    use config::names::Applies;
    let mut refusal = None;
    for value in [stock.checked_add(1), stock.checked_sub(1)].into_iter().flatten() {
        let mut changed = base.clone();
        set_limit(&mut changed, name, value);
        let root = worker.env.limits.clone();
        let git = *worker.git.component.limits();
        hear(worker, changed);
        let expected = match applies {
            Applies::Live => Some(Note::FiguresTaken { figures: Box::new([scalar(name, value)]) }),
            Applies::AtOnce | Applies::NextStart if known => {
                Some(Note::FiguresWaiting { figures: Box::new([scalar(name, value)]) })
            }
            Applies::AtOnce | Applies::NextStart => None,
        };
        let note = worker.notes().pop();
        assert!(worker.notes().is_empty(), "{name} has at most one disposition");
        if let Some(Note::FiguresRefused { problem }) = &note {
            assert_eq!(applies, Applies::Live, "only live figures can be refused");
            refusal = Some(problem.clone());
            continue;
        }
        assert_eq!(note, expected, "{name} {stock} -> {value}");
        hear(worker, base.clone());
        if applies == Applies::Live {
            assert_eq!(worker.notes().pop(), Some(Note::FiguresTaken { figures: Box::new([scalar(name, stock)]) }));
        }
        assert!(worker.notes().is_empty(), "{name} back at stock");
        assert_eq!(worker.env.limits, root, "{name} leaves the root at stock");
        assert_eq!(*worker.git.component.limits(), git, "{name} leaves git at stock");
        return;
    }
    let why = refusal.as_deref().map_or("no value", |text| core::str::from_utf8(text).unwrap());
    panic!("no accepted move for {name} at {stock}: {why}");
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

/// The engine's stock resolved without document overrides, then composed as
/// its opening does (configuration/figures.md, 2–3).
pub(crate) fn stock_figures() -> w::Figures {
    use temper_engine_service_startup::{limits, stock};
    let mut input: serde_json::Value = serde_json::from_slice(&temper_config_shell::init_document(true)).unwrap();
    input["engine"]["clients"]["provider"] = serde_json::Value::Null;
    let input = serde_json::to_vec(&input["engine"]).unwrap();
    let document =
        config::document(&input, &config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 }).unwrap();
    let given = stock::Given::Stock
        .with_providers(stock::providers(), Box::new([stock::scripted(([127, 0, 0, 1], 8370).into())]));
    let engine =
        temper_engine_service_startup::startup::engine_with(&document, 0, &config::Directories::working(), &given)
            .unwrap();
    let mut own = limits::stock();
    limits::derive(&mut own, temper_service_core::derived::STOCK).unwrap();
    let mut figures = temper_engine_service_startup::figures::Composer::new(&engine, &own).compose(&stock::agents());
    figures.places = 4;
    figures
}

pub(crate) fn stock_worker(figures: w::Figures) -> (startup::WorkerPart, crate::settings::Agent) {
    let input = fixtures::worker_alone_document();
    let mut part = startup::worker_part(&input, &config::Directories::working(), None, &[]).unwrap();
    let bytes = temper_worker_shell::agent::figures::configuration(
        &figures,
        Some(std::path::Path::new("/fixture/claude")),
        std::path::Path::new("/fixture/data"),
        &[],
    )
    .unwrap();
    let mut agent = fixtures::parsed_agent(smith_agent_shell::config::parse(&bytes).unwrap());
    // The standalone shell advertises the parsed host's reply allowance.
    agent.bounds.reply_bytes = Some(agent.limits.bodies.host_reply_text);
    figures::apply(&mut part, figures).unwrap();
    (part, agent)
}

#[test]
fn stock_engine_figures_start_a_standalone_worker() {
    let (mut part, agent) = stock_worker(stock_figures());
    let limits = crate::worker_settings::root_limits(&part.worker, &agent, &part.limits);
    assert!(temper_worker_domain::unfit(&limits).is_empty(), "{:?}", temper_worker_domain::unfit(&limits));
    assert!(temper_worker_domain::valid(&limits));
    startup::worker::check_worker_part(&mut part, &fixtures::files(), &agent).unwrap();
    startup::worker_alone(part, &fixtures::files(), &agent, fixtures::worker_runtime(), 7).unwrap();
}

#[test]
fn figures_places_past_the_worker_link_cap_refuse_the_start_naming_the_engine() {
    let mut figures = stock_figures();
    figures.places = 32;
    let (part, agent) = stock_worker(figures);
    let payload = crate::worker_settings::payload_limits(&part.limits).unwrap();
    let link = crate::worker_settings::link_limits(&part.worker.jig, &payload, part.limits.peer.forges).unwrap();
    assert!(w::sizes::worker_cap(&link.sizes).is_none());
    let found = startup::worker_alone(part, &fixtures::files(), &agent, fixtures::worker_runtime(), 7)
        .err()
        .expect("figures pass the worker link's cap");
    assert_eq!(found.problems.len(), 1);
    assert_eq!(found.problems.get(0).unwrap().path, config::Path::root().member(b"the engine's figures"));
    assert_eq!(found.problems.get(0).unwrap().what, startup::What::Jig(config::What::Bound));
}

#[test]
fn old_agent_payload_refuses_the_stock_event_carrier() {
    // The former agents setting's payload, before its 2-byte separator.
    const OLD_PAYLOAD: u64 = 10_662;
    let mut figures = stock_figures();
    let mut agents = figures.agents;
    for figure in agents.profile.iter_mut() {
        if figure.name.as_ref() == b"declared.tool_payload" {
            *figure = w::Limit { name: b"declared.tool_payload".as_slice().into(), value: OLD_PAYLOAD };
        }
    }
    figures.agents = agents;
    let (mut part, agent) = stock_worker(figures);
    let limits = crate::worker_settings::root_limits(&part.worker, &agent, &part.limits);
    assert!(temper_worker_domain::unfit(&limits).iter().any(|unfit| *unfit == temper_worker_domain::Unfit::Events));
    assert!(!temper_worker_domain::valid(&limits));
    let found = startup::worker::check_worker_part(&mut part, &fixtures::files(), &agent).unwrap_err();
    let lines = startup::lines(&found);
    let text = core::str::from_utf8(&lines).unwrap();
    assert!(text.contains("the engine's figures: hub.event_bytes 63738:"), "{text}");
}

#[test]
fn stock_places_are_the_largest_round_engine_channel_ceiling() {
    let mut jig = temper_service_core::derived::stock();
    temper_service_core::derived::engine(&mut jig, temper_service_core::derived::STOCK).unwrap();
    let mut own = temper_engine_service_startup::limits::stock();
    temper_engine_service_startup::limits::derive(&mut own, temper_service_core::derived::STOCK).unwrap();
    let payload = temper_engine_service_startup::translate::limits(&own).unwrap();
    let mut sizes =
        temper_service_core::derived::worker_sizes(&jig, jig.core.fleet.slots, &payload, own.forge.forges).unwrap();
    let cap = w::sizes::engine_cap(&sizes);
    // 32 = 2⁵; 40 is the next number of form d × 10ⁿ, 2ⁿ or 3 × 2ⁿ.
    assert_eq!(sizes.slots, 32);
    assert!(cap.is_some(), "{sizes:?}");
    sizes.slots = 40;
    assert!(w::sizes::engine_cap(&sizes).is_none(), "{cap:?}; next: {:?}", w::sizes::engine_cap(&sizes));
}
