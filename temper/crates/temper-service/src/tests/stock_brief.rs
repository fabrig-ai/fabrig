//! Stock admission and measured construction (temper/docs/design/domain/agent/tools.md).
#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    clippy::print_stderr,
    clippy::use_debug,
    reason = "the fixture fills embedded documents and reports counting-allocator measurements"
)]
use super::{engine, fixtures, generated};
use fixtures::parsed_agent as worker_agent;
use skein_lib::{Env, Queue, ReplyTo, Time, Token, Wall};
use skein_world::domain::heap::Meter;
use smith_domain_run as run;

fn configured() -> smith_agent_shell::config::Configuration {
    let mut configuration =
        fixtures::stock_configuration(10_662, 8192).expect("stock agent with local fake destinations");
    // These profile measurements use inert direct endpoints, with no program process.
    configuration.service.program = None;
    configuration.service.domain.programs = Box::default();
    configuration
}

fn assignment(number: u32, open: bool, body: usize) -> temper_engine_domain::Assignment {
    let engine = engine(&generated());
    let charter = engine.charters.iter().find(|charter| charter.number == number).unwrap();
    let mut policy =
        jig_domain_core::settings::charters::resolved(charter, &engine.llm.models, &engine.limits.core).unwrap().policy;
    // The maximal wake selects every part; jig joins them after the shared instructions.
    let mut instructions = policy.instructions.into_vec();
    for part in &charter.parts {
        instructions.extend_from_slice(b"\n\n");
        instructions.extend_from_slice(&part.instructions);
    }
    policy.instructions = instructions.into_boxed_slice();
    let mut assignment = fixtures::assignment_with(fixtures::repository(1, b"org", b"repo"));
    assignment.charter = number;
    assignment.run.policy = policy;
    // A source per model the policy names: its model, then each alternative.
    assignment.run.sources =
        vec![None; assignment.run.policy.alternatives.len().checked_add(1).unwrap()].into_boxed_slice();
    assignment.run.budget = engine.charters.iter().find(|charter| charter.number == number).unwrap().spend;
    assignment.run.authority.tools = jig_domain_tasks::Tools(
        crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes)
            .rules
            .ceiling
            .tools
            .0
            & assignment.run.policy.tools.0,
    );
    assert_eq!(
        crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes)
            .rules
            .ceiling
            .tools
            .0,
        1 | 2 | 4 | 32 | 128 | 256 | 512 | 1024 | 2048
    );
    assignment.run.ended_by = if number >= 13 {
        jig_domain_tasks::EndedBy::Standing
    } else if open {
        jig_domain_tasks::EndedBy::Requester
    } else {
        jig_domain_tasks::EndedBy::Result
    };
    assignment.run.contract = if open {
        jig_domain_tasks::Contract::Report { words: 32 }
    } else if number == 5 || number >= 13 {
        jig_domain_tasks::Contract::Report { words: engine.limits.core.tasks.result_bytes }
    } else if number == 10 {
        jig_domain_tasks::Contract::Report { words: 64 }
    } else if number == 9 {
        jig_domain_tasks::Contract::Report { words: 800 }
    } else if number == 4 {
        jig_domain_tasks::Contract::Verdict {
            choices: (1..=2)
                .map(|code| jig_domain_tasks::Verdict { code, words: 1024, followups: 0, fields: Box::default() })
                .collect(),
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
fn red_is_offered_only_to_stock_delegators_with_room_for_their_briefs() {
    let limits = configured().service.limits.domain.run;
    for charter in &engine(&generated()).charters {
        let number = charter.number;
        let charter = temper_engine_smith::charter(&assignment(number, false, 0));
        assert!(run::brief_room(&charter, None, &limits).unwrap() >= 2_048, "charter {number}");
        if number == 5 {
            let room = run::brief_room(&charter, Some(&workspace(true)), &limits).unwrap();
            eprintln!("red charter 5 with workspace: room={room}");
            assert!(room >= 335_619 + 64 * (16 + 7) + 195, "charter 5's pinned whole-brief margin: {room}");
        }
        assert_eq!(
            charter.grants.host_tools.iter().any(|tool| tool.name.as_ref() == b"red"),
            [1, 5, 13, 14, 15, 16].contains(&number),
            "charter {number}"
        );
    }
}

#[test]
fn finding_is_offered_only_to_stock_delegators_with_room_for_their_briefs() {
    let limits = configured().service.limits.domain.run;
    let mut declarations = skein_lib::List::uncapped();
    for number in engine(&generated()).charters.iter().map(|charter| charter.number) {
        let charter = temper_engine_smith::charter(&assignment(number, false, 0));
        let room = run::brief_room(&charter, None, &limits).expect("stock host part fits");
        eprintln!(
            "finding host part: charter {number}: non-brief={}, room={room}, bound={}",
            limits.host_part - u64::from(room),
            limits.host_part
        );
        assert!(room >= 2_048, "charter {number}");
        declarations
            .push((number, charter.grants.host_tools.iter().any(|tool| tool.name.as_ref() == b"finding")))
            .expect("stock declarations");
    }
    for (number, offered) in declarations.into_boxed() {
        assert_eq!(offered, [1, 5, 13, 14, 15, 16].contains(&number), "charter {number}");
    }
}

#[test]
fn every_stock_charter_admits_its_maximal_brief_with_receiving_fitting() {
    let configuration = configured();
    let limits = configuration.service.limits.domain.run;
    assert_eq!(limits.host_part, 399_961);
    assert_eq!(limits.room, 822_476);
    assert_eq!(limits.offer_bytes, 199_981);
    assert_eq!(configuration.service.limits.channel.bodies.turn_body, 16_803_810);

    for (number, open) in [
        (1, false),
        (1, true),
        (2, false),
        (4, false),
        (5, false),
        (9, false),
        (10, false),
        (11, true),
        (12, true),
        (13, false),
        (14, false),
        (15, false),
        (16, false),
    ] {
        let assignment = assignment(number, open, 14_592);
        let mut charter = temper_engine_smith::charter(&assignment);
        assert_eq!(charter.brief.sections.len(), 64);
        assert_eq!(charter.brief.sections.iter().map(|section| section.text.len()).sum::<usize>(), 14_592);
        let workspace = if number == 9 || number == 10 || number == 11 || number == 12 || number >= 13 {
            None
        } else {
            Some(workspace(number != 4 && !open))
        };
        let room = run::brief_room(&charter, workspace.as_ref(), &limits).expect("stock declarations fit");
        eprintln!(
            "host-part row: ({number}, {open}, {}, {room}),",
            limits.host_part.checked_sub(u64::from(room)).unwrap()
        );
        eprintln!(
            "stock charter {number} open={open}: non-brief={} rendered brief={} remaining={}",
            limits.host_part.checked_sub(u64::from(room)).unwrap(),
            run::brief_bytes(&charter.brief).unwrap(),
            i64::from(room).checked_sub(i64::try_from(run::brief_bytes(&charter.brief).unwrap()).unwrap()).unwrap()
        );
        let original = temper_engine_smith::charter(&assignment);
        let receiving = run::charter_room(workspace.as_ref(), &limits).unwrap();
        eprintln!(
            "receiving row: charter {number} open={open}: owned={} room={receiving}",
            charter.owned_bytes().unwrap()
        );
        temper_protocol_brief::fit_brief(&mut charter, workspace.as_ref(), &limits);
        if number == 5 {
            assert_eq!(original.owned_bytes().unwrap(), 82_979);

            assert_eq!(receiving, 65_463);
            assert!(original.owned_bytes().unwrap() > receiving, "the goal's larger charter needs receiving fitting");
        }
        if original.owned_bytes().unwrap() > receiving {
            assert_ne!(charter.brief, original.brief, "charter {number} needs receiving fitting");
        } else {
            assert_eq!(charter.brief, original.brief, "charter {number} keeps its whole maximal brief");
        }
        let sections = core::mem::take(&mut charter.brief.sections);
        assert_eq!(&charter, &without_brief(original), "fixed declarations stay whole");
        charter.brief.sections = sections;
        let fitted = run::brief_bytes(&charter.brief).unwrap();
        if number == 5 {
            assert_eq!(fitted, 122, "the goal's brief fits the joint receiving bound");
        }
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
                workspace,
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
        eprintln!("stock fit: charter {number} open={open}: admitted brief={fitted}");
    }
}

#[test]
fn the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds() {
    const FRAMING: u32 = 64 * (16 + 7);
    // Task words and contract, then the task identity and requester.
    const TASK_PARTS: u32 = 2;
    const MARGIN: u32 = 195;
    const OTHER: u32 = 768;
    let configuration = configured();
    let limits = configuration.service.limits.domain.run;
    let brief = crate::limits::profile(crate::limits::Profile::Standard).root.brief;
    assert!(brief.parts >= TASK_PARTS);
    let largest = temper_engine_smith::charter(&assignment(5, false, 0));
    let room = run::brief_room(&largest, Some(&workspace(true)), &limits).unwrap();
    assert_eq!(room, 337_286, "charter 5's maximal non-brief host part leaves this room");

    assert_eq!(brief.brief_bytes, 335_619);
    let full = brief.budgets.task + brief.budgets.pull + brief.budgets.reviews + OTHER;
    assert_eq!(full, 395_520);
    assert!(brief.brief_bytes <= full.min(room - FRAMING));
    for (number, open) in [
        (1, false),
        (1, true),
        (2, false),
        (4, false),
        (5, false),
        (9, false),
        (10, false),
        (11, true),
        (12, true),
        (13, false),
        (14, false),
        (15, false),
        (16, false),
    ] {
        let mut assignment = assignment(number, open, 0);
        assignment.sections = whole_spec_sections(&brief, OTHER);
        let mut charter = temper_engine_smith::charter(&assignment);
        assert_eq!(charter.brief.sections.len(), 64);
        assert_eq!(
            charter.brief.sections.iter().map(|section| section.text.len()).sum::<usize>(),
            usize::try_from(brief.brief_bytes).unwrap()
        );
        let mounted = if number == 9 || number == 10 || number == 11 || number == 12 || number >= 13 {
            None
        } else {
            Some(workspace(number != 4 && !open))
        };
        let remaining = run::brief_room(&charter, mounted.as_ref(), &limits).unwrap();
        if number == 2 || number == 5 {
            eprintln!(
                "stock host-part: charter {number}, non-brief={}, room={remaining}",
                limits.host_part - u64::from(remaining)
            );
        }
        let rendered = run::brief_bytes(&charter.brief).unwrap();
        eprintln!(
            "whole brief charter {number} open={open}: host bound={} non-brief={} room={remaining} rendered={rendered}",
            limits.host_part,
            limits.host_part - u64::from(remaining)
        );
        assert!(rendered <= u64::from(brief.brief_bytes + FRAMING));
        assert!(limits.host_part - u64::from(remaining) + rendered <= limits.host_part, "charter {number} open={open}");
        assert!(u64::from(remaining) - rendered >= u64::from(MARGIN));
        let whole = temper_engine_smith::charter(&assignment);
        temper_protocol_brief::fit_brief(&mut charter, mounted.as_ref(), &limits);
        assert!(charter.owned_bytes().unwrap() <= run::charter_room(mounted.as_ref(), &limits).unwrap());
        assert!(run::brief_bytes(&charter.brief).unwrap() <= u64::from(remaining));
        assert!(charter.brief.sections[0].text.ends_with(b" bytes; whole: the task's activity]\n"));
        assert_eq!(
            without_brief(charter),
            without_brief(whole),
            "charter {number} open={open} keeps fixed declarations"
        );
    }
    reference_spec_receiving_rows();
}

// The whole-spec table, including these receiving rows, measures 0.104 s CPU.
fn reference_spec_receiving_rows() {
    let configuration = fixtures::stock_configuration(63_740, 48_000).expect("reference agent");
    let limits = configuration.service.limits.domain.run;
    let mut assignment = assignment(2, false, 0);
    let engine = engine(&generated());
    let defaults = crate::stock::with_models(engine.llm.models.clone(), engine.limits.core.tasks.result_bytes);
    assignment.run.policy = crate::tests::reference_settings::charter(&defaults, 2).policy;
    assignment.sections = Box::new([temper_engine_domain::BriefSection {
        kind: temper_engine_domain::BriefKind::Core(jig_domain_brief::Core::Task),
        body: temper_engine_domain::BriefBody::Text(Box::default()),
    }]);
    let empty = temper_engine_smith::charter(&assignment);
    let mounted = workspace(true);
    let owned_room = run::charter_room(Some(&mounted), &limits).unwrap();
    let rendered_room = u64::from(run::brief_room(&empty, Some(&mounted), &limits).unwrap());
    let room = owned_room
        .checked_sub(empty.owned_bytes().unwrap())
        .unwrap()
        .min(rendered_room.checked_sub(run::brief_bytes(&empty.brief).unwrap()).unwrap());
    let room = usize::try_from(room).unwrap();
    eprintln!("reference coding section room={room}");
    assert!(room >= 63_740, "the reference coding charter must hold a whole spec");
    let past_room = room.checked_add(1).unwrap();
    for (size, whole) in [(63_740, true), (room, true), (past_room, false), (131_072, false)] {
        let mut charter = temper_engine_smith::charter(&assignment);
        charter.brief.sections[0].text = vec![b's'; size].into();
        let mut original = temper_engine_smith::charter(&assignment);
        original.brief.sections[0].text = vec![b's'; size].into();
        temper_protocol_brief::fit_brief(&mut charter, Some(&mounted), &limits);
        assert!(charter.owned_bytes().unwrap() <= owned_room);
        assert!(run::brief_bytes(&charter.brief).unwrap() <= rendered_room);
        if whole {
            assert_eq!(charter, original, "section of {size} stays whole");
        } else {
            let text = &charter.brief.sections[0].text;
            let kept = text.iter().take_while(|byte| **byte == b's').count();
            let lost = size.checked_sub(kept).unwrap();
            assert_eq!(&text[kept..], format!("\n[cut {lost} bytes; whole: the task's activity]\n").as_bytes());
            assert!(kept < size);
        }
        assert_eq!(without_brief(charter), without_brief(original));
    }
}

fn whole_spec_sections(
    brief: &temper_engine_domain::BriefLimits,
    other: u32,
) -> Box<[temper_engine_domain::BriefSection]> {
    use temper_engine_domain::{BriefBody, BriefKind, BriefSection, ForgeBriefKind};
    let spec = vec![b's'; 131_072];
    let mut task = spec.clone();
    task.extend_from_slice(b"\nContract: Change\n[This task: 1, in project 1]\nRequester: task 1\n");
    task.resize(usize::try_from(brief.budgets.task).unwrap(), b' ');
    assert_eq!(&task[..spec.len()], spec);
    // Gathering keeps the whole task and pull, then the reviews within the total.
    let reviews = brief
        .brief_bytes
        .checked_sub(brief.budgets.task)
        .unwrap()
        .checked_sub(brief.budgets.pull)
        .unwrap()
        .checked_sub(other)
        .unwrap();
    assert!(reviews <= brief.budgets.reviews);
    let mut sections = vec![
        BriefSection {
            kind: BriefKind::Core(jig_domain_brief::Core::Task),
            body: BriefBody::Text(task.clone().into()),
        },
        BriefSection {
            kind: BriefKind::Forge(ForgeBriefKind::Pull),
            body: BriefBody::Text(vec![b'p'; usize::try_from(brief.budgets.pull).unwrap()].into()),
        },
        BriefSection {
            kind: BriefKind::Forge(ForgeBriefKind::Reviews),
            body: BriefBody::Text(vec![b'r'; usize::try_from(reviews).unwrap()].into()),
        },
    ];
    for index in 0..61 {
        sections.push(BriefSection {
            kind: BriefKind::Core(jig_domain_brief::Core::Attempts),
            body: BriefBody::Text(
                vec![b'x'; usize::try_from((other / 61).checked_add(u32::from(index < other % 61)).unwrap()).unwrap()]
                    .into(),
            ),
        });
    }
    sections.into()
}

fn without_brief(mut charter: run::Charter) -> run::Charter {
    charter.brief.sections = Box::default();
    charter
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
        Some(&agent.declaration()),
        &mut problems,
    );
    assert!(problems.is_empty(), "{:?}", crate::startup::lines(&problems));
    let roots = crate::worker_settings::root_limits(&worker, &agent, &crate::limits::worker(&own));
    assert!(temper_worker_domain::valid(&roots), "{:?}", temper_worker_domain::unfit(&roots));
    let meter = Meter::new();
    meter.start();
    let worker = crate::Worker::new(
        &worker,
        &agent,
        &crate::limits::worker(&own),
        &files,
        crate::WorkerRuntime {
            wires: None,
            root: skein_io::kernel::Fd::new(9),
            path: b"/worker".as_slice().into(),
            agent: fixtures::pinned_agent(&worker.jig.agent),
        },
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
    let workspace = workspace(number != 4);
    let mut events = Queue::uncapped();
    let mut out = Queue::uncapped();
    events.push(smith_domain::Event::Start(smith_domain::Start {
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
        workspace: Some(workspace),
        transcript: None,
        answered: Box::default(),
        grants: Box::default(),
    }));
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
            if let smith_domain::Request::Lower(smith_domain::Lower::Read { owner, .. }) = request {
                events.push(smith_domain::Event::Below(smith_domain::Below::Read { owner, read: run::Read::Missing }));
            } else if let smith_domain::Request::Lower(smith_domain::Lower::Probe { owner, .. }) = request {
                events.push(smith_domain::Event::Below(smith_domain::Below::Probed { owner, executable: false }));
            } else if let smith_domain::Request::Lower(smith_domain::Lower::Complete { owner, .. }) = request {
                events.push(smith_domain::Event::Below(smith_domain::Below::Completed {
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
                }));
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
    assert_eq!(turns, 1, "stock charter {number} reaches a first turn: {refused:?}");
}
