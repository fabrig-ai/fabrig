//! A run reads the instructions for why it runs (`jig/docs/design/domain/tasks/messages.md`, 6).
use jig_core_world::{fixtures, world::World};
use jig_core_world::{fixtures::live_row, stories::per_wake as story};
use jig_domain_people as people;
use jig_domain_tasks as tasks;

const LANDING_BASE: &[u8] = b"BASE: complete the goal.";
const LANDING_FIRST: &[u8] = b"FIRST: delegate the work and await its results.";
const LANDING_RESULT: &[u8] = b"RESULT: when landed, create producer-7 close-out; pass all landed reports and Left lists; retire the draft and its ten citation allowances through main CI; name the close-out and main-CI selection in the final report.";

fn landing_goal(seed: u64) -> (World, (u64, u64)) {
    let mut setup = story::setup(seed, true);
    let charter = &mut setup.configuration.as_mut().expect("per-wake configuration").core.defaults.charters[1];
    charter.policy.instructions = LANDING_BASE.into();
    charter.parts = [(tasks::Reasons::FIRST, LANDING_FIRST), (tasks::Reasons::RESULT, LANDING_RESULT)]
        .into_iter()
        .map(|(on, instructions)| jig_domain_core::CharterPart { on, instructions: instructions.into() })
        .collect();
    let mut world = World::new(setup);
    let caller = fixtures::run(&world);
    let mut member = fixtures::member(&world, tasks::Executor::Agent { charter: 2 }, b"complete the goal");
    member.authority.budget.spend = 60;
    member.authority.delegation.tasks = 2;
    let task = fixtures::delegate(&mut world, caller, 1, member);
    let run = (task, live_row(&world, task).attempt);
    assert_assignment(&world, task, &landing_instructions(LANDING_FIRST), story::M1, &[]);
    assert!(live_row(&world, task).instructions.is_none(), "the charter supplies the instructions");
    assert_eq!(live_row(&world, task).reasons, tasks::Reasons::FIRST);
    assert_eq!(live_row(&world, task).offered, tasks::Reasons::EMPTY);
    story::park(&mut world, caller, None);
    (world, run)
}

fn landing_instructions(part: &[u8]) -> Vec<u8> {
    [LANDING_BASE, b"\n\n", part].concat()
}

fn landing_delegates(world: &mut World, goal: (u64, u64)) -> [(u64, u64); 2] {
    let mut member = fixtures::member(world, tasks::Executor::Agent { charter: 1 }, b"complete the landing");
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    [1, 2].map(|completion| {
        let task = fixtures::delegate(world, goal, completion, member.clone());
        (task, live_row(world, task).attempt)
    })
}

fn landed(world: &mut World, run: (u64, u64), report: &[u8]) {
    world.send(jig_test_domain::Event::Answer {
        channel: skein_lib::Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Finished {
            result: tasks::TaskResult::Report { words: report.into() },
            cancel_delegates: false,
        },
        read: None,
        refused: Box::new([]),
    });
}

fn assert_landing_result(world: &World, goal: u64, reports: &[tasks::Word]) {
    assert_assignment(world, goal, &landing_instructions(LANDING_RESULT), story::M1, reports);
    let row = live_row(world, goal);
    assert_eq!(row.reasons, tasks::Reasons::RESULT);
    assert_eq!(row.offered, tasks::Reasons::RESULT);
    assert_eq!(row.inbox.as_ref(), reports);
    assert!(row.delegates.is_empty());
}

#[test]
fn a_final_landing_result_supplies_the_result_part() {
    let (mut world, goal) = landing_goal(868);
    let delegates = landing_delegates(&mut world, goal);
    story::turn(&mut world, goal, None);
    story::park(&mut world, goal, None);
    let assignments = world.assignments.len();
    landed(&mut world, delegates[0], b"landed 75; Left: retire the draft");
    assert_eq!(world.assignments.len(), assignments, "a non-last success does not wake the goal");
    assert_eq!(live_row(&world, goal.0).phase, tasks::Phase::Active(tasks::Active::Idle));
    landed(&mut world, delegates[1], b"landed 77; Left: remove ten citation allowances");
    assert_eq!(world.assignments.len(), assignments + 1);
    let reports = live_row(&world, goal.0).inbox.clone();
    assert_eq!(reports.len(), 2);
    for (word, (delegate, report)) in reports.iter().zip(delegates.iter().zip([
        b"landed 75; Left: retire the draft".as_slice(),
        b"landed 77; Left: remove ten citation allowances".as_slice(),
    ])) {
        assert_eq!(word.from, tasks::Party::Task(delegate.0));
        assert_eq!(word.kind, tasks::MessageKind::Result(tasks::ResultKind::Report));
        assert_eq!(word.words.as_ref(), report);
    }
    assert!(reports[0].number < reports[1].number);
    assert_landing_result(&world, goal.0, &reports);
    let final_run = fixtures::run(&world);
    assert!(final_run.1 > goal.1);
    story::turn(&mut world, final_run, Some(reports[1].number));
    landed(&mut world, final_run, b"close-out and main CI selected");
    world.close();
}

#[test]
fn a_landing_result_is_relayed_and_taken_without_selecting_its_part() {
    let (mut world, goal) = landing_goal(869);
    let child = story::child(&mut world, goal);
    story::turn(&mut world, goal, None);
    story::park(&mut world, goal, None);
    fixtures::ask(
        &mut world,
        85,
        people::Ask::Say { project: 1, task: goal.0, words: b"review progress".as_slice().into() },
    );
    let words = live_row(&world, goal.0).inbox[0].clone();
    let review = fixtures::run(&world);
    assert_assignment(&world, goal.0, LANDING_BASE, story::M1, std::slice::from_ref(&words));
    assert_eq!(live_row(&world, goal.0).reasons, tasks::Reasons::WORDS);
    assert_eq!(live_row(&world, goal.0).offered, tasks::Reasons::EMPTY);
    let assignments = world.assignments.len();
    let mut relayed = world.inbound.clone();
    landed(&mut world, child, b"landed; Left: retire the draft");
    let report = live_row(&world, goal.0).inbox[1].clone();
    assert_eq!(report.kind, tasks::MessageKind::Result(tasks::ResultKind::Report));
    assert_eq!(report.from, tasks::Party::Task(child.0));
    assert!(report.number > words.number);
    relayed.push(report.clone());
    assert_eq!(world.inbound, relayed, "the live run hears its wake-worthy result without a result part");
    assert_eq!(world.assignments.len(), assignments);
    fixtures::ask(
        &mut world,
        86,
        people::Ask::Say { project: 1, task: goal.0, words: b"finish reviewing".as_slice().into() },
    );
    let fence = live_row(&world, goal.0).inbox[2].number;
    assert!(fence > report.number);
    story::turn(&mut world, review, Some(fence));
    assert!(live_row(&world, goal.0).inbox.is_empty(), "the reading turn takes the relayed result");
    assert_eq!(live_row(&world, goal.0).reasons, tasks::Reasons::WORDS);
    assert_eq!(live_row(&world, goal.0).offered, tasks::Reasons::EMPTY);
    assert_eq!(world.assignments.len(), assignments, "no result part is selected during the run");
    world.close();
}

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

fn waiting(world: &World) -> Option<&[u8]> {
    world.assigned.as_ref().expect("assignment").brief.iter().find_map(|section| match section {
        jig_domain_brief::GatherPlaced::Core { kind: jig_domain_brief::Core::Waiting, text } => Some(text.as_ref()),
        _ => None,
    })
}

fn read_question(seed: u64, wakes: bool) -> (World, (u64, u64), tasks::Word) {
    let mut world = story::world(seed, true);
    world.grant_account(2);
    let caller = fixtures::run(&world);
    let mut member = fixtures::member(&world, tasks::Executor::Agent { charter: 2 }, b"answer questions");
    member.authority.budget.spend = 60;
    member.wake = Some(tasks::WakePolicy { questions: wakes, ..tasks::WakePolicy::DEFAULT });
    let task = fixtures::delegate(&mut world, caller, 1, member);
    let first = (task, live_row(&world, task).attempt);
    assert_eq!(waiting(&world), None);
    let question = story::question(&mut world, first);
    story::turn(&mut world, first, None);
    story::park(&mut world, first, None);
    if !wakes {
        fixtures::ask(
            &mut world,
            84,
            people::Ask::Say { project: 1, task, words: b"read the question".as_slice().into() },
        );
    }
    let next = world.assigned.as_ref().expect("question run");
    let run = (next.task, next.attempt);
    assert!(next.messages.contains(&question));
    let read = next.messages.last().expect("question or words").number;
    assert_eq!(waiting(&world), None, "a start message is not repeated");
    story::turn(&mut world, run, Some(read));
    story::park(&mut world, run, Some(read));
    assert!(live_row(&world, run.0).inbox.is_empty());
    fixtures::ask(
        &mut world,
        85,
        people::Ask::Say { project: 1, task: run.0, words: b"person's words".as_slice().into() },
    );
    (world, run, question)
}

#[test]
fn an_open_question_survives_its_read_in_the_next_brief() {
    let (mut world, run, question) = read_question(866, true);
    let tasks::Party::Task(asker) = question.from else { panic!("delegate asks") };
    let expected = format!(
        "Questions asked of you, still open, oldest first:\nTask {asker} asks (question {0}); answer with message target={asker}, form=answer, question={0}:\nWhich next?\n",
        question.number
    );
    assert_eq!(waiting(&world), Some(expected.as_bytes()));
    assert!(world.assigned.as_ref().expect("fresh run").attempt > run.1);
    world.close();
}

#[test]
fn an_open_question_selects_its_part_when_person_words_wake_the_run() {
    for wakes in [true, false] {
        let (mut world, run, _) = read_question(867, wakes);
        assert_eq!(live_row(&world, run.0).reasons, tasks::Reasons::WORDS.union(tasks::Reasons::QUESTION));
        assert_eq!(world.assigned.as_ref().expect("words run").run.policy.instructions.as_ref(), b"S\n\nQ");
        assert!(live_row(&world, run.0).offered.contains(tasks::Reasons::QUESTION));
        world.close();
    }
}

fn question_action(
    world: &mut World,
    run: (u64, u64),
    completion: u32,
    target: u64,
    kind: tasks::MessageKind,
    words: &[u8],
) {
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::NamedAction {
        to: skein_lib::ReplyTo::new(skein_lib::Token::new(9300 + u64::from(completion))),
        key: jig_domain_core::CallKey { task: run.0, attempt: run.1, completion, position: 0 },
        action: jig_domain_core::NamedAction::Message { target, kind, words: words.into() },
    }));
}

fn next_brief_after_read(world: &mut World, run: (u64, u64), key: u8) -> (u64, u64) {
    let read = live_row(world, run.0).inbox.last().map(|word| word.number);
    story::turn(world, run, read);
    story::park(world, run, read);
    fixtures::ask(world, key, people::Ask::Say { project: 1, task: run.0, words: b"consider again".as_slice().into() });
    let next = world.assigned.as_ref().expect("fresh brief");
    assert_eq!(next.task, run.0);
    assert!(next.attempt > run.1);
    (next.task, next.attempt)
}

#[test]
fn waiting_keeps_whole_questions_oldest_first_and_drops_answers_and_ended_questions() {
    let mut setup = story::setup(868, false);
    // Two whole questions, their answer forms and the required task/tail fit together.
    setup.limits.core.brief.read_bytes = 512;
    setup.limits.core.brief.brief_bytes = 1024;
    let mut world = World::new(setup);
    world.grant_account(2);
    let task = story::delegate(&mut world, false);
    let run = (task, live_row(&world, task).attempt);
    let first = story::question(&mut world, run);
    let asker = world.assigned.as_ref().expect("asking child");
    let child = (asker.task, asker.attempt);
    let whole = b"Which form?\nTwo lines, kept whole.";
    question_action(&mut world, child, 2, run.0, tasks::MessageKind::Question, whole);
    let second = live_row(&world, run.0).inbox.last().expect("second question").number;
    let run = next_brief_after_read(&mut world, run, 87);
    let expected = format!(
        "Questions asked of you, still open, oldest first:\nTask {} asks (question {}); answer with message target={}, form=answer, question={}:\nWhich next?\nTask {} asks (question {second}); answer with message target={}, form=answer, question={second}:\n{}\n",
        child.0,
        first.number,
        child.0,
        first.number,
        child.0,
        child.0,
        core::str::from_utf8(whole).expect("question UTF-8")
    );
    assert_eq!(waiting(&world), Some(expected.as_bytes()));
    question_action(
        &mut world,
        run,
        1,
        child.0,
        tasks::MessageKind::Answer { question: first.number },
        b"the first form",
    );
    let run = next_brief_after_read(&mut world, run, 88);
    let expected = format!(
        "Questions asked of you, still open, oldest first:\nTask {} asks (question {second}); answer with message target={}, form=answer, question={second}:\n{}\n",
        child.0,
        child.0,
        core::str::from_utf8(whole).expect("question UTF-8")
    );
    assert_eq!(waiting(&world), Some(expected.as_bytes()));
    world.send(jig_test_domain::Event::Core(jig_domain_core::Event::Tasks(tasks::Event::CancelQuestion {
        reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(9400)),
        task: child.0,
        question: second,
    })));
    next_brief_after_read(&mut world, run, 89);
    assert_eq!(waiting(&world), None);
    world.close();
}

fn first_run() {
    let (mut world, _) = first(861, true, true);
    world.close();
}

#[derive(Clone, Copy)]
enum Arriving {
    Question,
    Result,
}

fn arriving_after_preparation(seed: u64, kind: Arriving) {
    let (mut world, run) = first(seed, true, true);
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
    let (arriving, instructions, reason) = match kind {
        Arriving::Question => (story::question_from(&mut world, child, run.0), b"S\n\nQ", tasks::Reasons::QUESTION),
        Arriving::Result => (story::result_from(&mut world, child), b"S\n\nR", tasks::Reasons::RESULT),
    };
    assert_eq!(world.inbound, relayed, "no run has claimed the arriving word");
    assert_eq!(live_row(&world, run.0).inbox.as_ref(), [words.clone(), arriving.clone()]);
    world.retry_due();
    let refreshed = world.domain.take_workspace();
    world.send(jig_test_domain::Event::Core(prepared));
    if let Some(refreshed) = refreshed {
        world.send(jig_test_domain::Event::Core(refreshed));
    }
    assert_eq!(
        world.assignments.len(),
        assignments + 1,
        "an arriving kind must join the claimed run; phase: {:?}, refusals: {}",
        live_row(&world, run.0).phase,
        live_row(&world, run.0).refusals
    );
    assert_assignment(&world, run.0, instructions, story::M2, &[words, arriving.clone()]);
    let second = world.assigned.as_ref().expect("prepared run claimed");
    let second = (second.task, second.attempt);
    let row = live_row(&world, run.0);
    assert_eq!(row.reasons, tasks::Reasons::WORDS.union(reason));
    assert_eq!(row.offered, reason);
    assert_eq!(world.inbound, relayed);
    story::turn(&mut world, second, Some(arriving.number));
    assert!(live_row(&world, run.0).inbox.is_empty());
    let assignments = world.assignments.len();
    story::park(&mut world, second, Some(arriving.number));
    assert_eq!(world.assignments.len(), assignments, "the arriving kind needs no later run");
    assert_eq!(live_row(&world, run.0).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert!(world.domain.take_workspace().is_none());
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
        account: None,
        main: story::M1,
        subagents: Box::new([]),
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
        worker: 0,
        channel: skein_lib::Token::new(7),
        hello: jig_domain_link::Hello {
            jobs: Box::new([]),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: skein_lib::Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([]),
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

fn covered_escalation(seed: u64) -> (World, (u64, u64), u64, tasks::Word) {
    let (mut world, first) = first(seed, true, true);
    let child = story::escalation(&mut world, first);
    let entry = world.inbound.last().expect("escalation relayed").clone();
    story::turn(&mut world, first, Some(entry.number));
    story::park(&mut world, first, Some(entry.number));
    assert_assignment(&world, first.0, b"S\n\nH", story::M2, std::slice::from_ref(&entry));
    assert_eq!(live_row(&world, first.0).offered, tasks::Reasons::ESCALATION);
    let second = (first.0, live_row(&world, first.0).attempt);
    story::turn(&mut world, second, Some(entry.number));
    park_with_entry(&mut world, second, child, &entry);
    (world, second, child, entry)
}

fn park_with_entry(world: &mut World, run: (u64, u64), child: u64, entry: &tasks::Word) {
    let assignments = world.assignments.len();
    story::park(world, run, Some(entry.number));
    assert_eq!(world.assignments.len(), assignments, "a covered undecided entry must not assign another run");
    assert_waiting_entry(world, run.0, child, entry);
}

fn assert_waiting_entry(world: &World, holder: u64, child: u64, entry: &tasks::Word) {
    assert_eq!(live_row(world, holder).phase, tasks::Phase::Active(tasks::Active::Idle));
    let row = live_row(world, child);
    assert!(matches!(row.phase, tasks::Phase::Held { .. }));
    assert!(matches!(row.escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(task), entry: number, .. }
            if task == holder && number == entry.number));
}

fn an_escalation_its_part_left_waits_for_a_change() {
    let (mut world, run, child, entry) = covered_escalation(868);
    fixtures::ask(
        &mut world,
        85,
        people::Ask::Say { project: 1, task: run.0, words: b"Please check".as_slice().into() },
    );
    let words = live_row(&world, run.0).inbox[0].clone();
    assert_eq!(words.kind, tasks::MessageKind::Words);
    assert_assignment(&world, run.0, b"S\n\nH", story::M2, &[entry.clone(), words.clone()]);
    assert_eq!(live_row(&world, run.0).reasons, tasks::Reasons::WORDS.union(tasks::Reasons::ESCALATION));
    let next = (run.0, live_row(&world, run.0).attempt);
    story::turn(&mut world, next, Some(words.number));
    park_with_entry(&mut world, next, child, &entry);
    let revision = live_row(&world, child).revisions.hold;
    story::rehold(&mut world, child);
    let new_revision = live_row(&world, child).revisions.hold;
    assert!(new_revision > revision);
    let changed = world.assigned.as_ref().expect("changed entry wakes holder");
    assert_eq!(changed.task, run.0);
    assert!(changed.attempt > next.1);
    let [new_entry] = changed.messages.as_ref() else { panic!("new entry at start") };
    assert!(new_entry.number > entry.number);
    assert_eq!(new_entry.kind, tasks::MessageKind::Escalation { task: child, revision: new_revision });
    assert_eq!(live_row(&world, run.0).reasons, tasks::Reasons::ESCALATION);
    world.close();
}

fn a_covered_escalation_stays_parked_across_a_restart() {
    const SEED: u64 = 869;
    let (mut world, run, child, entry) = covered_escalation(SEED);
    let caller = fixtures::run(&world);
    story::park(&mut world, caller, None);
    world.stop();
    let assignments = world.assignments.len();
    world.restart_with(story::setup(SEED, true).configuration.expect("same configuration"));
    assert_eq!(world.assignments.len(), assignments, "restoring a covered entry must not assign a run");
    assert_waiting_entry(&world, run.0, child, &entry);
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
    bundle.case("arriving_after_preparation_question", 866, || arriving_after_preparation(866, Arriving::Question));
    bundle.case("arriving_after_preparation_result", 870, || arriving_after_preparation(870, Arriving::Result));
    crate::bundle::case!(bundle, 867, a_goal_interrupted_in_its_first_cut_keeps_its_parts_and_choice);
    crate::bundle::case!(bundle, 868, an_escalation_its_part_left_waits_for_a_change);
    crate::bundle::case!(bundle, 869, a_covered_escalation_stays_parked_across_a_restart);
    bundle.finish().unwrap_or_else(|failures| panic!("{failures}"));
}
