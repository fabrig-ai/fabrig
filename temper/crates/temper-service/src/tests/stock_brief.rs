//! Stock admission and measured construction (temper/docs/design/domain/agent.md, 4.3).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::print_stderr,
    clippy::use_debug,
    reason = "the fixture fills embedded documents and reports counting-allocator measurements"
)]
use super::{engine, fixtures, generated};
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use skein_world::domain::heap::Meter;
use smith_domain_run as run;

fn configured() -> smith_agent_shell::config::Configuration {
    let bytes = include_str!("../../../temper-init/template/agent.json")
        .replace("$llm", "127.0.0.1:8370")
        .replace("\"$programs\"", "{}")
        .replace("api.openai.com:443", "127.0.0.1:8372")
        .replace("\"transport\": \"tls\"", "\"transport\": \"plaintext\"")
        .replace("\"server_name\": \"api.openai.com\",", "");
    smith_agent_shell::config::parse(bytes.as_bytes()).expect("stock agent with local fake destinations")
}

fn assignment(number: u32, open: bool, body: usize) -> temper_engine_domain::Assignment {
    let engine = engine(&generated());
    let policy = engine.charters.iter().find(|charter| charter.number == number).unwrap().policy.clone();
    let mut assignment = fixtures::assignment_with(fixtures::repository(1, b"org", b"repo"));
    assignment.charter = number;
    assignment.run.policy = policy;
    assignment.run.sources = Box::new([None, None]);
    assignment.run.budget = engine.charters.iter().find(|charter| charter.number == number).unwrap().spend;
    assignment.run.authority.tools = jig_domain_tasks::Tools(crate::stock::defaults(&engine.llm).rules.ceiling.tools.0);
    if number == 4 {
        assignment.run.authority.tools = jig_domain_tasks::Tools(jig_domain_core::ToolFamilies::JIG.read.0);
    }
    assignment.run.ended_by =
        if open { jig_domain_tasks::EndedBy::Requester } else { jig_domain_tasks::EndedBy::Result };
    assignment.run.contract = if open {
        jig_domain_tasks::Contract::Report { words: 32 }
    } else if number == 4 {
        jig_domain_tasks::Contract::Verdict {
            choices: [1, 2]
                .map(|code| jig_domain_tasks::Verdict { code, words: 1024, followups: 0, fields: Box::default() })
                .into(),
        }
    } else {
        jig_domain_tasks::Contract::Change { connector: 0, kind: 1, words: 1024 }
    };
    assignment.sections = (0..64)
        .map(|index| temper_engine_domain::BriefSection {
            kind: temper_engine_domain::BriefKind::Core(jig_domain_brief::Core::Attempts),
            body: temper_engine_domain::BriefBody::Text(
                vec![b'x'; body.checked_div(64).unwrap().checked_add(usize::from(index < body % 64)).unwrap()]
                    .into_boxed_slice(),
            ),
        })
        .collect();
    assignment
}

fn workspace(writable: bool) -> run::Workspace {
    run::Workspace {
        directories: Box::new([run::Directory {
            name: b"work".as_slice().into(),
            host: b"/work".as_slice().into(),
            root: Token::new(1),
            writable,
            git: true,
            conflicts: Box::default(),
        }]),
    }
}

#[test]
fn every_stock_charter_admits_its_whole_maximal_brief() {
    let configuration = configured();
    let limits = configuration.service.limits.domain.run;
    assert_eq!(limits.host_part, 52_000);
    assert_eq!(limits.room, 125_634);
    assert_eq!(limits.offer_bytes, 26_000);
    assert_eq!(configuration.service.limits.channel.bodies.turn_body, 1_067_414);
    for (number, open) in [(1, false), (1, true), (2, false), (4, false)] {
        let assignment = assignment(number, open, 14_592);
        let charter = temper_engine_smith::charter(&assignment);
        assert_eq!(charter.brief.sections.len(), 64);
        assert_eq!(charter.brief.sections.iter().map(|section| section.text.len()).sum::<usize>(), 14_592);
        let workspace = workspace(number != 4 && !open);
        let room = run::brief_room(&charter, Some(&workspace), &limits).expect("stock declarations fit");
        eprintln!(
            "stock charter {number} open={open}: non-brief={} rendered brief={} remaining={}",
            limits.host_part.checked_sub(u64::from(room)).unwrap(),
            run::brief_bytes(&charter.brief).unwrap(),
            u64::from(room).checked_sub(run::brief_bytes(&charter.brief).unwrap()).unwrap()
        );
        let mut domain = run::Domain::new(&limits, 7);
        let mut out = Queue::uncapped();
        run::step(
            &mut domain,
            &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
            run::Event::Start {
                reply_to: ReplyTo::new(Token::new(1)),
                host_run: Token::new(1),
                activation: 1,
                identity: None,
                window: run::Window { turns: 8, bytes: 16_777_216, largest_turn: 2_097_152 },
                messages: Box::default(),
                charter,
                workspace: Some(workspace),
                transcript: None,
                resumed: false,
            },
            &mut out,
        );
        let mut admitted = false;
        for request in &out {
            if let run::Request::Admitted { .. } = request {
                admitted = true;
            }
        }
        assert!(admitted, "charter {number} open={open}: {out:?}");
        eprintln!("stock fit: charter {number} open={open}: admitted whole");
    }
}

#[test]
fn stock_small_heap_through_construction_admission_and_first_turn() {
    let configuration = configured();
    let meter = Meter::new();
    meter.start();
    let service = smith_agent_service::Service::new(configuration.service, 7).expect("stock service");
    let measured = meter.end();
    eprintln!("heap agent Service::new held={} peak={}", measured.held(), measured.peak());
    drop(service);
    measure_worker();
    for number in [1, 2, 4] {
        measure_turn(number);
    }
}

fn worker_agent(config: smith_agent_shell::config::Configuration) -> crate::settings::Agent {
    let limits = config.service.limits;
    let host_limits = smith_local_service::profile::host_limits(&config.profile, &limits).unwrap();
    let mut protocol = smith_host_protocol::Limits {
        bodies: limits.channel.bodies,
        charter: limits.channel.charter,
        transcript: limits.channel.transcript,
        endpoints: limits.channel.endpoints,
        channel: limits.channel.channel,
        start_bytes: limits.channel.start_bytes,
        calls: host_limits.calls,
    };
    protocol.channel.output_bytes = smith_host_protocol::output_bytes(&protocol).unwrap();
    crate::settings::Agent {
        run_limits: limits.domain.run,
        tool_payload: config.profile.declared.tool_payload,
        programs: Box::default(),
        endpoints: config.service.channel_endpoints,
        limits: protocol,
        host_limits,
        bounds: jig_domain_host::Bounds {
            models: config
                .declarations
                .endpoints
                .iter()
                .flat_map(|endpoint| endpoint.models.iter())
                .map(|model| jig_domain_host::ModelBound {
                    name: model.name.clone(),
                    window: model.window,
                    output: model.output,
                })
                .collect(),
            program_endpoints: Box::default(),
            turns: config.profile.policy.max_turns,
            spend: config.profile.policy.max_spend,
            time: config.profile.policy.max_time,
            waiting: config.profile.policy.max_waiting,
            checks: config.profile.policy.check_timeout,
            message_bytes: limits.domain.run.message_bytes.checked_sub(2).unwrap(),
            inbox: config.profile.policy.inbox,
            unacknowledged: config.profile.policy.unacknowledged,
        },
    }
}

fn measure_worker() {
    let bytes = fixtures::value(&generated(), b"workers[0]");
    let document = jig_protocol_config::document(
        &bytes,
        &jig_protocol_config::DocumentLimits { bytes: 1_048_576, tokens: 65_536, depth: 32 },
    )
    .unwrap();
    let jig = jig_protocol_config::worker(&document, 0, &jig_protocol_config::Directories::working()).unwrap();
    let worker =
        crate::settings::Worker { jig, root: b"/worker".as_slice().into(), git: Some(b"/bin/git".as_slice().into()) };
    let agent = worker_agent(configured());
    let mut files = crate::settings::Files { entries: skein_lib::List::with_capacity(1) };
    files
        .entries
        .push((worker.jig.secret.clone(), skein_lib::Secret::new(b"worker-secret-value".as_slice().into())))
        .unwrap();
    let overrides = fixtures::value(&generated(), b"application.limits.overrides");
    let mut own = crate::limits::profile(crate::limits::Profile::Small);
    let overrides = jig_protocol_config::document(
        &overrides,
        &jig_protocol_config::DocumentLimits { bytes: 16_384, tokens: 4096, depth: 4 },
    )
    .unwrap();
    for at in (1..overrides.len().checked_sub(1).unwrap()).step_by(2) {
        let name = overrides.text(overrides.token(at).unwrap()).unwrap();
        let value = jig_protocol_config::read::integer(&overrides, at.checked_add(1).unwrap(), u64::MAX).unwrap();
        crate::limits::apply(&mut own, name, value).unwrap();
    }
    let engine = engine(&generated());
    let turn = u64::from(agent.limits.bodies.turn_body);
    let resume = turn.checked_add(4).unwrap();
    let transcript = resume.checked_add(12_u64.checked_mul(resume / 4).unwrap()).unwrap();
    let loaded = turn.checked_add(u64::try_from(size_of::<temper_engine_domain::Record>()).unwrap()).unwrap();
    assert_eq!(engine.limits.hub.turn_bytes, turn);
    assert_eq!(worker.jig.limits.hub.turn_bytes, turn);
    assert_eq!(u64::from(engine.limits.core.resume_bytes), resume);
    assert_eq!(u64::from(own.root.journal.transcript_bytes), resume);
    assert_eq!(engine.limits.hub.transcript_bytes, transcript);
    assert_eq!(worker.jig.limits.hub.transcript_bytes, transcript);
    assert_eq!(engine.limits.hub.turn_queue_bytes, 8 * turn);
    assert_eq!(worker.jig.limits.hub.turn_queue_bytes, 8 * turn);
    assert_eq!(u64::from(own.root.loads.bytes), loaded);
    assert_eq!(u64::from(own.root.loads.reply_bytes), loaded);
    assert_eq!(own.root.loads.read_bytes, 2 * turn);
    assert_eq!(u64::from(own.root.loads.transcript_bytes), turn);
    let mut problems = crate::startup::Problems::empty();
    crate::startup::checks::turns(
        Some(&engine.limits),
        core::slice::from_ref(&worker.jig),
        &own,
        Some(&agent),
        &mut problems,
    );
    assert!(problems.is_empty(), "{:?}", crate::startup::lines(&problems));
    let roots = crate::worker_settings::root_limits(&worker, &agent, &own);
    assert!(temper_worker_domain::valid(&roots), "{:?}", temper_worker_domain::unfit(&roots));
    let meter = Meter::new();
    meter.start();
    let worker = crate::Worker::new(
        &worker,
        &agent,
        &own,
        &files,
        crate::WorkerRuntime { root: skein_io::kernel::Fd::new(9), path: b"/worker".as_slice().into() },
        7,
    )
    .unwrap();
    let measured = meter.end();
    eprintln!("heap temper Worker::new held={} peak={}", measured.held(), measured.peak());
    drop(worker);
}

fn measure_turn(number: u32) {
    let config = configured();
    let body = usize::try_from(crate::limits::profile(crate::limits::Profile::Small).root.brief.brief_bytes).unwrap();
    let assignment = assignment(number, false, body);
    let limits = config.service.limits.domain;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let meter = Meter::new();
    meter.start();
    let mut domain = smith_domain::Domain::new(&limits, config.service.domain, 7);
    let charter = temper_engine_smith::charter(&assignment);
    let mut events = Queue::uncapped();
    let mut out = Queue::uncapped();
    events.push(smith_domain::Event::Start {
        reply_to: ReplyTo::new(Token::new(1)),
        host_run: Token::new(1),
        activation: 1,
        identity: None,
        window: smith_domain::Window {
            turns: 8,
            bytes: smith_domain::max_turn_bytes(&limits).unwrap().checked_mul(8).unwrap(),
        },
        messages: Box::default(),
        charter,
        workspace: Some(workspace(number != 4)),
        transcript: None,
        answered: Box::default(),
        grants: Box::default(),
    });
    let mut turns = 0_u32;
    let mut refused = None;
    for _ in 0_u32..1000 {
        for _ in 0..events.len() {
            smith_domain::step(&mut domain, &env, events.pop().unwrap(), &mut out);
        }
        for _ in 0_u32..1000 {
            if !domain.is_ready() {
                break;
            }
            smith_domain::resume(&mut domain, &env, &mut out);
        }
        for _ in 0..out.len() {
            let request = out.pop().unwrap();
            if let smith_domain::Request::Read { owner, .. } = request {
                events.push(smith_domain::Event::Read { owner, read: run::Read::Missing });
            } else if let smith_domain::Request::Probe { owner, .. } = request {
                events.push(smith_domain::Event::Probed { owner, executable: false });
            } else if let smith_domain::Request::Complete { owner, .. } = request {
                events.push(smith_domain::Event::Completed {
                    owner,
                    completion: smith_domain::llm::Completion {
                        reasoning_dropped: Box::default(),
                        content: Box::new([smith_domain::llm::Said::Text {
                            text: b"measured first turn".as_slice().into(),
                            replay: None,
                        }]),
                        stop: smith_domain::llm::Stop::EndTurn,
                        usage: smith_domain::llm::Usage {
                            input_tokens: Some(1),
                            output_tokens: Some(1),
                            cache_read_tokens: Some(0),
                            cache_write_tokens: Some(0),
                            reasoning_tokens: Some(0),
                        },
                    },
                });
            } else if let smith_domain::Request::Turn { .. } = request {
                turns = turns.checked_add(1).unwrap();
            } else if let smith_domain::Request::Answer { answer: run::Answer::Refused(reason), .. } = request {
                refused = Some(reason);
            }
        }
        domain.reclaim();
        if turns > 0 || refused.is_some() {
            break;
        }
    }
    let measured = meter.end();
    eprintln!(
        "heap charter {number} body={body} turns={turns} refused={refused:?} held={} peak={}",
        measured.held(),
        measured.peak()
    );
    if limits.run.host_part == 52_000 {
        assert_eq!(turns, 1, "stock charter {number} reaches a first turn: {refused:?}");
    }
}
