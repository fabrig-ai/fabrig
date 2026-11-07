//! A run reads the instructions for why it runs (`jig/docs/design/domain/tasks.md`, 8.6).
use jig_core_world::{fixtures, world::World};
use jig_core_world::{fixtures::live_row, stories::per_wake as story};
use jig_domain_people as people;
use jig_domain_tasks as tasks;

fn assert_assignment(
    world: &World,
    task: u64,
    instructions: &[u8],
    model: tasks::ModelEffort,
    messages: &[tasks::Word],
) {
    let assignment = world.assigned.as_ref().expect("delegate assigned");
    assert_eq!(assignment.task, task);
    assert_eq!(assignment.run.policy.instructions.as_ref(), instructions);
    let (name, effort, account) = if model == story::M1 {
        (b"scripted".as_slice(), b"low".as_slice(), 1)
    } else {
        assert_eq!(model, story::M2);
        (b"research".as_slice(), b"high".as_slice(), 2)
    };
    assert_eq!(assignment.run.policy.model.name.as_ref(), name);
    assert_eq!(assignment.run.policy.model.effort.as_deref(), Some(effort));
    assert_eq!(assignment.grants[0].account, account);
    assert_eq!(assignment.messages.as_ref(), messages);
    assert!(!assignment.run.policy.resume);
}

fn first(seed: u64, parts: bool, later: bool) -> (World, (u64, u64)) {
    let mut world = story::world(seed, parts);
    world.grant_account(2);
    let task = story::delegate(&mut world, later);
    assert_assignment(&world, task, if parts { b"S\n\nF" } else { b"S" }, story::M1, &[]);
    assert!(world.inbound.is_empty());
    let row = live_row(&world, task);
    assert_eq!(row.reasons, tasks::Reasons::FIRST);
    assert_eq!(row.offered, if parts { tasks::Reasons::EMPTY } else { tasks::Reasons::ALL });
    let choice = row.llm.as_ref().expect("stored choice");
    assert_eq!(choice.main, story::M1);
    assert_eq!(choice.later.as_ref().map(|later| later.main), later.then_some(story::M2));
    let attempt = row.attempt;
    (world, (task, attempt))
}

fn words_and_an_amendment_are_always_relayed() {
    let (mut world, run) = first(864, true, true);
    fixtures::ask(
        &mut world,
        81,
        people::Ask::Say { project: 1, task: run.0, words: b"person's words".as_slice().into() },
    );
    let mut amendment = jig_test_profile::fixtures::person_unchanged(b"new goal");
    let mut spec = live_row(&world, run.0).spec.clone();
    spec.words = b"amended goal".as_slice().into();
    amendment.spec =
        Some(people::Spec { asks: false, words: spec.words.clone(), parameters: Box::new([]), inputs: Box::new([]) });
    fixtures::ask(
        &mut world,
        82,
        people::Ask::Amend { project: 1, task: run.0, revision: 0, release: None, amendment },
    );
    assert_eq!(
        world.people_answers.last(),
        Some(&people::Reply::Outcome(people::Outcome::Amended { task: run.0, released: false }))
    );
    let row = live_row(&world, run.0);
    assert_eq!(row.spec.words, spec.words);
    assert_eq!(row.revisions.spec, 1);
    assert_eq!(row.reasons, tasks::Reasons::FIRST);
    assert_eq!(row.offered, tasks::Reasons::EMPTY);
    assert_eq!(row.inbox.len(), 2);
    assert_eq!(world.inbound, row.inbox.as_ref());
    let [words, amendment] = world.inbound.as_slice() else { panic!("both messages relayed") };
    assert!(matches!(words.from, tasks::Party::Person(_)));
    assert_eq!(words.kind, tasks::MessageKind::Words);
    assert_eq!(words.words.as_ref(), b"person's words");
    assert!(matches!(amendment.kind, tasks::MessageKind::Amendment { .. }));
    assert!(words.number < amendment.number);
    assert_assignment(&world, run.0, b"S\n\nF", story::M1, &[]);
    let read = amendment.number;
    story::turn(&mut world, run, Some(read));
    assert!(live_row(&world, run.0).inbox.is_empty());
    world.close();
}

fn an_undecided_escalation_wakes_its_part() {
    let (mut world, first) = first(863, true, true);
    let task = first.0;
    let child = story::escalation(&mut world, first);
    let entry = world.inbound.last().expect("escalation relayed despite first part").clone();
    assert_eq!(
        entry.kind,
        tasks::MessageKind::Escalation { task: child, revision: live_row(&world, child).revisions.hold }
    );
    assert!(matches!(live_row(&world, child).escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), .. } if holder == task));
    assert!(live_row(&world, task).inbox.is_empty(), "entries take no inbox room");
    story::turn(&mut world, first, Some(entry.number));
    story::park(&mut world, first, Some(entry.number));
    let next = world.assigned.as_ref().expect("undecided escalation wakes next run");
    assert_eq!(next.task, task);
    assert!(next.attempt > first.1);
    assert_eq!(next.run.policy.instructions.as_ref(), b"S\n\nH");
    assert_eq!(next.run.policy.model.name.as_ref(), b"research");
    assert_eq!(next.run.policy.model.effort.as_deref(), Some(b"high".as_slice()));
    assert_eq!(next.messages.as_ref(), std::slice::from_ref(&entry));
    assert_eq!(world.inbound, [entry]);
    let row = live_row(&world, task);
    assert_eq!(row.reasons, tasks::Reasons::ESCALATION);
    assert_eq!(row.offered, tasks::Reasons::ESCALATION);
    assert!(row.inbox.is_empty());
    assert!(matches!(live_row(&world, child).escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), .. } if holder == task));
    world.close();
}

fn a_question_waits_for_its_part() {
    question_row(862, true);
}

fn a_charter_without_parts_relays_the_question_and_keeps_its_model() {
    question_row(865, false);
}

fn question_row(seed: u64, parts: bool) {
    let (mut world, first) = first(seed, parts, parts);
    let task = first.0;
    let question = story::question(&mut world, first);
    assert_eq!(question.from, tasks::Party::Task(world.assigned.as_ref().expect("asking delegate").task));
    assert_eq!(
        world.inbound.as_slice(),
        if parts { &[] } else { std::slice::from_ref(&question) },
        "only an offered question is relayed"
    );
    assert_eq!(live_row(&world, task).inbox.as_ref(), std::slice::from_ref(&question));
    // A later offered fence passes the question without taking it when its kind is withheld.
    let read = if parts {
        fixtures::ask(&mut world, 83, people::Ask::Say { project: 1, task, words: b"carry on".as_slice().into() });
        let words = world.inbound.last().expect("person's words are always offered");
        assert_eq!(words.kind, tasks::MessageKind::Words);
        assert!(words.number > question.number);
        Some(words.number)
    } else {
        None
    };
    story::turn(&mut world, first, read);
    assert_eq!(live_row(&world, task).inbox.as_ref(), std::slice::from_ref(&question));
    let relayed = world.inbound.clone();
    story::park(&mut world, first, read);
    let next = world.assigned.as_ref().expect("question wakes next run");
    assert!(next.attempt > first.1);
    assert_assignment(
        &world,
        task,
        if parts { b"S\n\nQ" } else { b"S" },
        if parts { story::M2 } else { story::M1 },
        std::slice::from_ref(&question),
    );
    assert_eq!(world.inbound, relayed, "the next start carries the question without a duplicate relay");
    let row = live_row(&world, task);
    assert_eq!(row.reasons, tasks::Reasons::QUESTION);
    assert_eq!(row.offered, if parts { tasks::Reasons::QUESTION } else { tasks::Reasons::ALL });
    assert_eq!(row.inbox.as_ref(), [question]);
    assert_eq!(row.llm.as_ref().expect("original choice retained").main, story::M1);
    world.close();
}

fn first_run() {
    let (mut world, _) = first(861, true, true);
    world.close();
}

fn a_question_arriving_after_preparation_waits_for_the_next_run() {
    let (mut world, run) = first(866, true, true);
    let child = story::child(&mut world, run);
    story::turn(&mut world, run, None);
    story::park(&mut world, run, None);
    assert_eq!(live_row(&world, run.0).phase, tasks::Phase::Active(tasks::Active::Idle));
    world.domain.script_workspaces();
    let assignments = world.assignments.len();
    fixtures::ask(
        &mut world,
        84,
        people::Ask::Say { project: 1, task: run.0, words: b"prepare another run".as_slice().into() },
    );
    let prepared = world.domain.take_workspace().expect("brief prepared, workspace completion held before claim");
    assert_eq!(world.assignments.len(), assignments);
    assert_eq!(live_row(&world, run.0).phase, tasks::Phase::Active(tasks::Active::Preparing));
    let words = live_row(&world, run.0).inbox[0].clone();
    let relayed = world.inbound.clone();
    let question = story::question_from(&mut world, child, run.0);
    assert_eq!(world.inbound, relayed, "no run has claimed the question");
    assert_eq!(live_row(&world, run.0).inbox.as_ref(), [words.clone(), question.clone()]);
    world.send(jig_test_domain::Event::Core(prepared));
    if let Some(refreshed) = world.domain.take_workspace() {
        world.send(jig_test_domain::Event::Core(refreshed));
    }
    assert_eq!(
        world.assignments.len(),
        assignments + 1,
        "a question arriving after preparation must leave the prepared run claimable; phase: {:?}, refusals: {}",
        live_row(&world, run.0).phase,
        live_row(&world, run.0).refusals
    );
    assert_assignment(&world, run.0, b"S", story::M2, std::slice::from_ref(&words));
    let second = world.assigned.as_ref().expect("prepared run claimed");
    let second = (second.task, second.attempt);
    assert_eq!(live_row(&world, run.0).offered, tasks::Reasons::EMPTY);
    assert_eq!(world.inbound, relayed);
    story::turn(&mut world, second, Some(words.number));
    assert_eq!(live_row(&world, run.0).inbox.as_ref(), std::slice::from_ref(&question));
    let assignments = world.assignments.len();
    story::park(&mut world, second, Some(words.number));
    let prepared = world.domain.take_workspace().expect("question's workspace completion held before claim");
    world.send(jig_test_domain::Event::Core(prepared));
    if let Some(refreshed) = world.domain.take_workspace() {
        world.send(jig_test_domain::Event::Core(refreshed));
    }
    assert_eq!(world.assignments.len(), assignments + 1, "the kept question claims a new run");
    let next = world.assigned.as_ref().expect("question wakes next run");
    assert_eq!(next.task, run.0);
    assert!(next.attempt > second.1);
    assert_assignment(&world, run.0, b"S\n\nQ", story::M2, std::slice::from_ref(&question));
    assert_eq!(live_row(&world, run.0).reasons, tasks::Reasons::QUESTION);
    assert_eq!(live_row(&world, run.0).offered, tasks::Reasons::QUESTION);
    world.close();
}

fn a_goal_interrupted_in_its_first_cut_keeps_its_parts_and_choice() {
    const SEED: u64 = 867;
    let setup = || {
        let mut setup = story::setup(SEED, true);
        setup.configuration.as_mut().expect("per-wake configuration").core.workers_in_process = true;
        setup
    };
    let mut world = World::new(setup());
    world.grant_account(2);
    let caller = fixtures::run(&world);
    let mut goal = fixtures::member(&world, tasks::Executor::Agent { charter: 2 }, b"Cut the goal into work");
    goal.label = 2;
    goal.tracked = Some(1);
    goal.authority.budget.spend = 60;
    goal.llm = Some(tasks::LlmChoice {
        account: None, main: story::M1, subagents: Box::new([]),
        later: Some(tasks::LlmLater { main: story::M2, subagents: Box::new([]) }),
    });
    let task = fixtures::delegate(&mut world, caller, 1, goal);
    assert_assignment(&world, task, b"S\n\nF", story::M1, &[]);
    let attempt = live_row(&world, task).attempt;
    story::park(&mut world, caller, None);
    story::turn(&mut world, (task, attempt), None);
    world.stop();
    assert_eq!(live_row(&world, task).reasons, tasks::Reasons::FIRST, "Recheck retains the first cut's reasons");
    assert_eq!(live_row(&world, task).tries, tasks::Tries::NONE);
    assert!(world.store.rows.values().any(|row| matches!(row,
        jig_test_domain::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Terminal(end)))
            if end.task == task && end.attempt == attempt && end.end == tasks::End::Recheck
    )));
    world.restart_with(setup().configuration.expect("same configuration"));
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0, channel: skein_lib::Token::new(7),
        hello: jig_domain_link::Hello {
            jobs: Box::new([]), bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: skein_lib::Duration::from_millis(100), slots: 2,
            workstreams: Box::new([]), hosting: Box::new([]),
        },
    })));
    world.grant_account(1);
    world.grant_account(2);
    assert!(live_row(&world, task).attempt > attempt);
    assert_assignment(&world, task, b"S\n\nF", story::M1, &[]);
    assert_eq!(live_row(&world, task).reasons, tasks::Reasons::FIRST);
    let next = live_row(&world, task).attempt;
    story::park(&mut world, (task, next), None);
    world.close();
}

#[test]
fn per_wake_scenarios() {
    let mut bundle = super::bundle::Bundle::default();
    crate::bundle::case!(bundle, 861, first_run);
    crate::bundle::case!(bundle, 862, a_question_waits_for_its_part);
    crate::bundle::case!(bundle, 863, an_undecided_escalation_wakes_its_part);
    crate::bundle::case!(bundle, 864, words_and_an_amendment_are_always_relayed);
    crate::bundle::case!(bundle, 865, a_charter_without_parts_relays_the_question_and_keeps_its_model);
    crate::bundle::case!(bundle, 866, a_question_arriving_after_preparation_waits_for_the_next_run);
    crate::bundle::case!(bundle, 867, a_goal_interrupted_in_its_first_cut_keeps_its_parts_and_choice);
    bundle.finish().unwrap_or_else(|failures| panic!("{failures}"));
}
