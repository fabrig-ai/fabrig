//! The world's fault ledger (testing/fakes.md, 2).
use skein_fake_forge_domain::{self as forge, api as raw};
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use skein_world::{
    faults::{Kind, Struck},
    streams::{Stream, Streams},
};
use temper_engine_forge_world::{World, fake_config, translate};
use temper_faults::{
    forge::{LATE, UNAVAILABLE},
    stale::MOVED_BASE,
};

#[test]
fn unavailable_is_copied_before_the_connector_sees_its_answer() {
    let mut config = fake_config();
    config.unavailable = 1000;
    let mut world = World::new_in(&Streams::new(17), config);
    world.adopt();
    let start = world.now();
    let end = start.saturating_add(Duration::from_secs(3));
    while world.step_until(end) {
        assert!(world.forge_mut().pop_fell().is_none(), "every step drains the fake");
    }
    let entries = world.ledger().entries();
    assert!(!entries.is_empty(), "the unavailable forge was called");
    assert_eq!(entries.len(), usize::try_from(world.calls()).expect("calls fit"));
    let mut requests = std::collections::BTreeSet::new();
    for entry in entries {
        assert_eq!(entry.kind, UNAVAILABLE);
        assert_eq!(entry.until, Some(entry.from), "unavailable is an instant");
        assert!(entry.from <= end);
        let request = entry.struck.request.as_ref().expect("call's reply token");
        assert!(request.parse::<u64>().is_ok(), "request is decimal");
        assert!(requests.insert(request), "each request is recorded once");
    }
}

fn probe(domain: &mut forge::Domain, env: &Env<forge::Config>) {
    forge::step(
        domain,
        env,
        forge::Event::Call {
            reply_to: ReplyTo::new(Token::new(123)),
            user: 1,
            repository: b"org/repo".as_slice().into(),
            op: raw::Op::Read(raw::Read::Locate),
        },
        &mut Queue::uncapped(),
    );
    let at = domain.next_deadline().expect("call due");
    forge::fire(domain, &Env { now: at, wall: env.wall, limits: env.limits.clone() }, &mut Queue::uncapped());
}

#[test]
fn late_intervals_use_the_fakes_times_and_faults_stream() {
    let streams = Streams::new(29);
    let mut config = fake_config();
    config.late = 1000;
    let mut world = World::new_in(&streams, config.clone());
    let env = Env { now: world.now(), wall: Wall::EPOCH, limits: config.clone() };
    let mut reference = forge::Domain::new(&config, streams.seed_of(Stream::Faults, "forge"));
    probe(&mut reference, &env);
    let fell = reference.pop_fell().expect("late call");
    assert!(reference.pop_fell().is_none());
    probe(world.forge_mut(), &env);
    world.step_until(world.now());
    let entries = world.ledger().entries();
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(entry.kind, LATE);
    assert_eq!(entry.from, fell.at);
    assert_eq!(entry.until, fell.until);
    assert!(entry.until.expect("late answer ends") > entry.from);
    assert_eq!(entry.struck.request.as_deref(), Some("123"));
}

#[test]
fn lost_ci_names_the_gate_run_and_engine_candidate() {
    let mut config = fake_config();
    config.lost = 1000;
    let mut world = World::batch_turn_in(&Streams::new(41), None, config);
    world.push_file(b"main", b"file", b"candidate");
    let head = world.branch(b"main");
    world.external(
        raw::Op::Write(raw::Write::Dispatch {
            workflow: b"gate.yml".as_slice().into(),
            branch: b"main".as_slice().into(),
            inputs: Box::new([]),
        }),
        1,
    );
    world.run_for(3);
    let raw::Answer::Runs(runs) = world.inspect(&raw::Read::Runs {
        workflow: Some(b"gate.yml".as_slice().into()),
        commit: Some(head),
        number: None,
        page: 1,
        limit: 0,
    }) else {
        panic!("workflow runs")
    };
    let run = runs.first().expect("dispatched run").id;
    let entries: Vec<_> =
        world.ledger().entries().iter().filter(|entry| entry.kind == Kind { family: "ci", name: "lost" }).collect();
    assert_eq!(entries.len(), 1);
    let entry = entries[0];
    assert_eq!(entry.struck.gate_run, Some(run.to_string()));
    let candidate =
        String::from_utf8(temper_engine_domain_forge::commit_text(&translate::commit(head)).into()).expect("hex");
    assert_eq!(entry.struck.candidate, Some(candidate));
    assert_eq!(entry.until, Some(entry.from));
    assert!(world.forge_mut().pop_fell().is_none(), "fire drains before delivery");
}

#[test]
fn fault_records_drain_at_capacity_and_count_overflow() {
    let mut config = fake_config();
    config.limits.observations = 1;
    config.unavailable = 1000;
    let mut world = World::new_in(&Streams::new(53), config.clone());
    let env = Env { now: world.now(), wall: Wall::EPOCH, limits: config };
    for count in 1..=2 {
        probe(world.forge_mut(), &env);
        assert_eq!(world.forge_mut().fell_lost(), 0, "the record fits at capacity");
        world.step_until(world.now());
        assert_eq!(world.ledger().entries().len(), count);
        assert!(world.forge_mut().pop_fell().is_none(), "the world drains each record");
    }
    probe(world.forge_mut(), &env);
    probe(world.forge_mut(), &env);
    assert_eq!(world.forge_mut().fell_lost(), 1, "one record exceeds capacity");
}

#[test]
fn world_injections_start_and_stop_at_world_time_and_can_be_taken() {
    let mut world = World::new_in(&Streams::new(67), fake_config());
    world.run_for(1);
    let start = world.now();
    let kind = MOVED_BASE;
    let struck = Struck { base: Some("main".into()), ..Struck::default() };
    let injection = world.inject(kind, struck.clone());
    assert_eq!(world.ledger().get(injection).kind, kind);
    assert_eq!(world.ledger().get(injection).struck, struck);
    assert_eq!(world.ledger().get(injection).from, start);
    assert_eq!(world.ledger().get(injection).until, None);
    world.run_for(2);
    world.stop(injection);
    let end = world.now();
    assert!(end > start);
    let ledger = world.take_ledger();
    assert_eq!(ledger.get(injection).until, Some(end));
    assert!(world.ledger().entries().is_empty());
}

fn retry_trace(mut world: World) -> Vec<(Time, u32)> {
    world.call_latencies(Duration::ZERO, Duration::ZERO);
    let start = world.wall();
    let until = Wall::from_nanos(start.as_nanos() + Duration::from_secs(10).as_nanos());
    world.unavailable(start, until);
    world.adopt();
    let end = world.now().saturating_add(Duration::from_secs(15));
    let mut calls = 0;
    let mut trace = Vec::new();
    while world.step_until(end) {
        if calls != world.calls() {
            calls = world.calls();
            trace.push((world.now(), calls));
        }
    }
    assert!(calls > 1, "connector retries its unavailable forge");
    trace
}

#[test]
fn streams_replay_and_keep_the_engine_on_its_scheduling_seed() {
    let streams = Streams::new(79);
    let first = retry_trace(World::new_in(&streams, fake_config()));
    let second = retry_trace(World::new_in(&streams, fake_config()));
    assert_eq!(first, second, "one seed replays its scheduling");
    let legacy = retry_trace(World::new(streams.seed_of(Stream::Scheduling, "engine")));
    assert_eq!(first, legacy, "connector and its retries use Scheduling/engine");

    let replay = || {
        let mut config = fake_config();
        config.unavailable = 500;
        config.late = 500;
        let mut world = World::new_in(&streams, config);
        world.adopt();
        world.run_for(30);
        assert!(!world.ledger().entries().is_empty(), "replay exercises injected faults");
        format!("{:?}", world.take_ledger())
    };
    assert_eq!(replay(), replay(), "one seed replays its nonempty ledger");
}
