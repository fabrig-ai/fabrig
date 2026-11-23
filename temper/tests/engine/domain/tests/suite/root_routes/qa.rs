//! Milestone messages and report-only testers (testing/reference-case.md).
use super::*;

const M1: &[u8] = b"M1 ended.\nDesign followed.\nChanges landed.\nCI green.\nReviews passed.\nSTATUS updated.\nNo blocker.\nM2 may start.\nQA follows.\nA1 stays live.";
const TESTER_SPEC: &[u8] = b"Report test path and content from the named M1 input.";

fn read_inbox(driver: &mut Driver, run: &engine::Assignment) {
    let key = Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(run.task)));
    let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) =
        driver.store.rows.get(&key)
    else {
        panic!("run proof")
    };
    let Some(read) = proof.outstanding.last().copied() else { return };
    let turn = live(driver, run.task).turn + 1;
    driver.send(protocol::turn(
        Token::new(7),
        run.task,
        run.attempt,
        turn,
        0,
        Some(read),
        b"Read the offered messages",
    ));
    driver.settle();
}

fn batch(driver: &mut Driver, run: &engine::Assignment, call: u64, members: Box<[engine::Delegate]>) -> Box<[u64]> {
    read_inbox(driver, run);
    call_batch(driver, run, call, members)
}

fn fixture(configuration: [engine::Config; 2], bounds: &engine::Limits) -> (Driver, engine::Assignment) {
    let mut driver = Driver::configured(Store::with_limits(bounds), configuration, bounds);
    let mut hello = protocol::hello_slots(Token::new(7), bounds.fleet.slots, Box::new([]));
    if let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut hello
    {
        hello.bounds.message_bytes = bounds.tasks.message_bytes;
    }
    driver.send(hello);
    driver.settle();
    driver.sign_in();
    driver.settle();
    chat(&mut driver, 12);
    driver.settle();
    let run = driver.last_assignment();
    (driver, run)
}

fn message(driver: &mut Driver, from: &engine::Assignment, to: u64, call: u64, words: &[u8]) {
    read_inbox(driver, from);
    tool_call(
        driver,
        from,
        call,
        engine::Tool::Message { target: to, form: engine::MessageForm::Words, words: words.into() },
    );
    assert!(
        matches!(driver.answered(call), Some(engine::CallAnswer::Sent { .. })),
        "message {call}: {:?}",
        driver.answered(call)
    );
}

fn milestone_owners() -> (Driver, engine::Assignment, engine::Assignment, engine::Assignment) {
    // Director, A, A1, tester, A2; all five may be assigned together.
    let (mut configuration, mut bounds) = batch_settings(5, 3, 500, false, false, false);
    for config in &mut configuration {
        config.core.defaults.rules.ceiling.delegation.tasks = 5;
        let policy = &mut config.core.seed_projects[0].policy.policy;
        policy.ceiling.delegation.tasks = 5;
        policy.roles[0].authority.delegation.tasks = 5;
    }
    bounds.tasks.message_bytes = 256; // M1 and its relayed header.
    bounds.call_records = 8; // Four delegations, the milestone message and its relay.
    bounds.tasks.tasks = 5;
    bounds.tasks.project_tasks = 5;
    bounds.tasks.tree_tasks = 5;
    bounds.people.inbox_tasks = 10;
    bounds.people.inbox_recipients = 21;
    let (mut driver, director) = fixture(configuration, &bounds);
    let mut owner = report_delegate(b"A", Box::new([]));
    owner.authority.budget.spend = 100;
    owner.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 3, depth: 2 };
    let a = batch(&mut driver, &director, 70, Box::new([owner]))[0];
    let owner = assigned_task(&driver, a);
    let mut subowner = report_delegate(b"A1", Box::new([]));
    subowner.authority.budget.spend = 50;
    subowner.authority.delegation =
        tasks::Delegation { kinds: Box::new([tasks::AuthorityExecutor::Charter(1)]), tasks: 1, depth: 1 };
    let a1 = batch(&mut driver, &owner, 71, Box::new([subowner]))[0];
    let subowner = assigned_task(&driver, a1);
    (driver, director, owner, subowner)
}

#[test]
fn milestone_words_start_a_sibling_beside_live_qa_and_relay_upward() {
    let (mut driver, director, owner, subowner) = milestone_owners();
    let tester = batch(&mut driver, &subowner, 72, Box::new([plain_delegate(TESTER_SPEC)]))[0];
    park_task(&mut driver, &owner);
    message(&mut driver, &subowner, owner.task, 73, M1);
    let receiving = driver.last_assignment();
    let next = batch(&mut driver, &receiving, 74, Box::new([plain_delegate(b"M2 sub-owner")]))[0];
    let relay = [b"A1's M1 report:\n".as_slice(), M1].concat();
    message(&mut driver, &receiving, director.task, 75, &relay);
    assert_eq!(receiving.task, owner.task);
    assert!(
        receiving
            .messages
            .iter()
            .any(|word| word.from == tasks::Party::Task(subowner.task) && word.words.as_ref() == M1)
    );
    assert_eq!(live(&driver, owner.task).attempt, receiving.attempt, "delegation and relay stay in the receiving run");
    assert_eq!(live(&driver, next).requester, tasks::Party::Task(owner.task));
    assert_eq!(live(&driver, next).spec.words.as_ref(), b"M2 sub-owner");
    for task in [subowner.task, tester, next] {
        assert!(matches!(live(&driver, task).phase, tasks::Phase::Active(tasks::Active::Running { .. })));
    }
    assert!(
        live(&driver, director.task)
            .inbox
            .iter()
            .any(|word| word.from == tasks::Party::Task(owner.task) && word.words.as_ref() == relay)
    );
    assert!(
        messages_to(&driver, &director).iter().any(|(_, words)| words.windows(relay.len()).any(|part| part == relay))
    );
}

#[test]
fn a_report_only_tester_keeps_its_named_input_and_narrow_authority() {
    let (mut driver, parent) = batch_fixture_with(2, 1);
    let source = batch(&mut driver, &parent, 70, Box::new([plain_delegate(b"M1 report")]))[0];
    let source_run = assigned_task(&driver, source);
    finish_task(&mut driver, &source_run, tasks::TaskResult::Report { words: b"M1 report ready".as_slice().into() });
    let mut tester = plain_delegate(TESTER_SPEC);
    tester.spec.inputs = Box::new([source]);
    let task = batch(&mut driver, &parent, 71, Box::new([tester]))[0];
    let row = live(&driver, task);
    assert_eq!(row.spec.words.as_ref(), TESTER_SPEC);
    assert_eq!(row.spec.inputs.as_ref(), [source]);
    assert_eq!(row.contract, tasks::Contract::Report { words: 128 });
    assert_eq!(row.authority.tools, tasks::Tools(0));
    assert!(row.authority.grants.is_empty());
    assert_eq!(row.authority.delegation, tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 });
    let run = assigned_task(&driver, task);
    assert!(run.sections.iter().any(|section| section.kind == engine::BriefKind::Core(brief::Core::Results)
        && matches!(&section.body, engine::BriefBody::Text(words) if words.windows(b"M1 report ready".len()).any(|part| part == b"M1 report ready"))));
    let charter = temper_engine_smith::charter(&run);
    for forbidden in [b"effect_forge".as_slice(), b"delegate"] {
        assert!(!charter.grants.host_tools.iter().any(|tool| tool.name.as_ref() == forbidden));
    }
}

#[test]
fn finished_subtree_documents_keep_parents_results_and_complete_activity() {
    let (mut driver, director, owner, subowner) = milestone_owners();
    let tester = batch(&mut driver, &subowner, 72, Box::new([plain_delegate(TESTER_SPEC)]))[0];
    let tester_run = assigned_task(&driver, tester);
    for (run, words) in
        [(&tester_run, b"test".as_slice()), (&subowner, b"M1"), (&owner, b"owner"), (&director, b"director")]
    {
        read_inbox(&mut driver, run);
        let number = live(&driver, run.task).turn + 1;
        turn(&mut driver, run, number, 0);
        driver.settle();
        finish_task(&mut driver, run, tasks::TaskResult::Report { words: words.into() });
    }
    for (task, requester, words) in [
        (director.task, tasks::Party::Person(1), b"director".as_slice()),
        (owner.task, tasks::Party::Task(director.task), b"owner"),
        (subowner.task, tasks::Party::Task(owner.task), b"M1"),
        (tester, tasks::Party::Task(subowner.task), b"test"),
    ] {
        let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
            driver.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
        else {
            panic!("ended task {task}")
        };
        let row = row.clone();
        driver.send(engine::Event::Party(engine::PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(901)),
            sign_in: driver.session(),
            read: Box::new(jig_domain_core::reads::Read::Task { task }),
        }));
        driver.settle();
        let document = driver
            .delivered
            .iter()
            .rev()
            .find_map(|delivery| match delivery {
                Delivery::Read { answer: jig_domain_core::reads::Answer::Ended(ended), .. } => Some(ended),
                _ => None,
            })
            .expect("ended task document");
        let jig_domain_core::reads::EndedValue::Task { document, .. } = &document.value else {
            panic!("task document")
        };
        assert_eq!(document.chip.number, task);
        assert_eq!(document.chip.phase, jig_domain_core::documents::PhaseLine::Ended);
        assert_eq!(document.requester, requester);
        assert!(document.delegates.is_empty());
        assert!(document.result.is_some());
        assert!(!document.cut);
        assert_eq!(row.requester, requester);
        assert_eq!(row.project, 1);
        assert_eq!(row.revisions.spec, 0);
        assert_eq!(row.executor, tasks::Executor::Agent { charter: 1 });
        assert_eq!(
            row.phase,
            tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: words.into() }))
        );
        assert!(row.delegates.is_empty());
        assert!(driver.store.rows.values().any(|record| matches!(record,
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Terminal(end)))
                if end.task == task && matches!(end.end, tasks::End::Finished { .. }))));
    }
    assert!(driver.root.tasks().view_tasks().is_empty(), "no live roots or delegates");
}
