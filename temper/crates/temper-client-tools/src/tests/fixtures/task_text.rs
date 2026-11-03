//! Core-produced text windows for ctl source reads.
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Time, Wall};

/// A synthetic stored task projected with the shipped 128-byte text window.
pub(crate) fn source_task(words: &[u8]) -> (core::Core, core::Limits, tasks::TaskRecord) {
    let mut limits = jig_test_profile::limits().core;
    limits.documents.text_bytes = 128;
    limits.tasks.spec_bytes = 4096;
    let config = jig_test_profile::config(17).core;
    let mut authority = core::task_authority(&config.seed_projects[0].policy.policy.roles[0].authority);
    authority.budget.spend = 100;
    let rows = jig_test_profile::fixtures::seeded_project_rows(&config);
    let mut core = core::Core::new(config, &limits).unwrap();
    core.start_begin(true, Wall::EPOCH);
    let env = Env { limits, now: Time::ZERO, wall: Wall::EPOCH };
    for row in rows {
        assert_ne!(core.restore(&env, row), core::start::Restore::Refused);
    }
    let mut out = Queue::uncapped();
    jig_domain_people::step(
        &mut core.people,
        &Env { limits: limits.people, now: Time::ZERO, wall: Wall::EPOCH },
        jig_domain_people::Event::Restore {
            record: jig_domain_people::Stored::Roles {
                project: 1,
                holdings: Box::new([jig_domain_people::Holding { person: 1, role: jig_domain_people::Role::Owner }]),
            },
        },
        &mut out,
    );
    let task_env = Env { limits: limits.tasks, now: Time::ZERO, wall: Wall::EPOCH };
    let mut task_out = Queue::uncapped();
    for event in [
        tasks::Event::Check,
        tasks::Event::Restored,
        tasks::Event::OpenPeriod {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
            project: 1,
            period: 1,
            budget: 1000,
        },
    ] {
        tasks::step(&mut core.tasks, &task_env, event, &mut task_out);
    }
    let mut made = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &task_env,
        tasks::Event::Make {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
            creator: tasks::Party::Person(1),
            batch: Box::new([new_task(authority, words)]),
        },
        &mut made,
    );
    let Some(row) = core.tasks.task(1) else { panic!("synthetic task: {made:?}") };
    let mut row = row.clone();
    row.instructions = Some(Box::from(words));
    (core, limits, row)
}

fn new_task(authority: tasks::Authority, words: &[u8]) -> tasks::New {
    tasks::New {
        deadline: tasks::Deadline::Named,
        label: 0,
        flags: tasks::Flags::PLAIN,
        number: 1,
        project: 1,
        executor: tasks::Executor::Agent { charter: 1 },
        llm: None,
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: Box::from(words),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 64 },
        authority,
        numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Period { project: 1, period: 1 },
        dependencies: Box::new([]),
        holdings: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        recurring: None,
        tracked: None,
    }
}

/// The core's authentic bounded summary after both client wire translations.
pub(crate) fn source_summary(
    core: &core::Core,
    limits: &core::Limits,
    row: &tasks::TaskRecord,
) -> jig_client::contract::Task {
    let task = core::documents::task_from_row(core, limits, 1, row, Wall::EPOCH).unwrap();
    let answer = core::reads::Answer::Document(Box::new(core::documents::Document::Task(Box::new(task))));
    let jig_client_protocol_wire::ReadValue::Task(wire) =
        jig_protocol_engine_clients::translate::objects::document(&answer, &[], &jig_client_protocol_wire::CEILINGS)
            .unwrap()
    else {
        panic!("task summary")
    };
    let limits = crate::limits::client().client.domain;
    let limits = jig_client_protocol_link::Limits::new(limits, 4).unwrap();
    jig_client_protocol_link::translate::documents::task(wire, &limits).unwrap()
}

/// Move an authentic core chunk into the client's independently bounded vocabulary.
pub(crate) fn source_chunk(
    core: &core::Core,
    limits: &core::Limits,
    row: &tasks::TaskRecord,
    field: core::reads::TaskTextField,
    from: u64,
) -> jig_client::contract::TaskText {
    let page = core::reads::task_text_from_row(core, limits, 1, row, field, from).unwrap();
    jig_client::contract::TaskText {
        task: page.task,
        field: match field {
            core::reads::TaskTextField::Specification => jig_client::contract::TaskTextField::Specification,
            core::reads::TaskTextField::Instructions => jig_client::contract::TaskTextField::Instructions,
        },
        from: page.from,
        total: page.total,
        next: page.next,
        text: jig_client::contract::Text::new(page.bytes, 128).unwrap(),
    }
}

/// Serve task show through its bounded projection and every supplemental runner read.
pub(crate) fn source_show(
    words: &[u8],
    tools: &crate::jig::Limits,
    json: bool,
) -> (crate::jig::Printed, skein_lib::List<u64>) {
    let (core, limits, row) = source_task(words);
    let mut root = super::signed_in(super::PARTY);
    root.limits.tools = *tools;
    let argv = if json { &["task", "show", "1", "--json"][..] } else { &["task", "show", "1"][..] };
    let call = root.submit(argv);
    root.serve(call, super::task_result(source_summary(&core, &limits, &row)));
    let mut offsets = skein_lib::List::with_capacity(tools.text);
    for _ in 0..tools.text {
        let Some((_, jig_client::contract::Read::TaskText { task, field, from })) = root.reads.get(&call) else {
            break;
        };
        assert_eq!(*task, 1);
        assert_eq!(*field, jig_client::contract::TaskTextField::Specification);
        offsets.push(*from).expect("one offset per source byte");
        let page = source_chunk(&core, &limits, &row, core::reads::TaskTextField::Specification, *from);
        root.serve(
            call,
            jig_client::contract::ReadResult::Value(jig_client::contract::ReadValue::TaskText(Box::new(page))),
        );
    }
    assert!(!root.reads.contains_key(&call), "supplemental reads finish within the byte bound");
    (root.answer(call).expect("task show printed"), offsets)
}
