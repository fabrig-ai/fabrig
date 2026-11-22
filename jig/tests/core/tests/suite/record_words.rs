//! A run records its own directing words by a decided act (domain/tasks/messages.md, 2).
use jig_core_world::{fixtures, stories::per_wake, world::World};
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_notes as notes;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{List, ReplyTo, Token};

fn goal(seed: u64, message_family: bool) -> (World, (u64, u64)) {
    let mut setup = per_wake::setup(seed, true);
    let config = &mut setup.configuration.as_mut().expect("per-wake settings").core;
    config.defaults.charters[0].policy.resume = false;
    config.defaults.rules.ceiling.notes = authority::Scopes::PROJECT;
    let policy = &mut config.seed_projects[0].policy.policy;
    policy.ceiling.notes = authority::Scopes::PROJECT;
    policy.roles[0].authority.notes = authority::Scopes::PROJECT;
    let mut checked = authority::Domain::new(config.defaults.rules.clone(), setup.limits.core.authority)
        .expect("bounded note policy");
    let mut out = skein_lib::Queue::uncapped();
    authority::step(&mut checked, authority::Event::Policy { project: 1, policy: policy.clone() }, &mut out);
    assert_eq!(out.pop(), Some(authority::PolicyFact::Added { project: 1 }));
    *policy = checked.policy(1).expect("checked policy").clone();
    config.defaults.rules = checked.rules().clone();
    let mut world = World::new(setup);
    let requester = fixtures::run(&world);
    let mut member = fixtures::member(&world, tasks::Executor::Agent { charter: 2 }, b"keep directing words");
    member.authority.budget.spend = 60;
    member.authority.notes = tasks::Scopes(authority::Scopes::PROJECT.0);
    if !message_family {
        member.authority.tools.0 &= !core::ToolFamilies::JIG.message.0;
    }
    let task = fixtures::delegate(&mut world, requester, 1, member);
    let run = (task, fixtures::live_row(&world, task).attempt);
    per_wake::park(&mut world, requester, None);
    directing_words(&mut world, run);
    (world, run)
}

fn directing_words(world: &mut World, run: (u64, u64)) {
    world.store_checks(jig_core_world::world::StoreChecks::none("authorized words fixture names missing Person(42)"));
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::AuthorizedMessage {
        reply_to: ReplyTo::new(Token::new(4200)),
        project: 1,
        operation: tasks::Message {
            task: run.0,
            word: tasks::Word {
                number: 77,
                from: tasks::Party::Person(42),
                kind: tasks::MessageKind::Words,
                words: b"carry this into the plan".as_slice().into(),
                at: world.env().wall,
                hits: 1,
                eligible: false,
            },
        },
    })));
    assert_eq!(fixtures::live_row(world, run.0).inbox[0].number, 77);
    per_wake::turn(world, run, Some(77));
    assert!(fixtures::live_row(world, run.0).inbox.is_empty());
    let kept = &fixtures::live_row(world, run.0).unrecorded;
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].number, 77);
    assert_eq!(kept[0].from, tasks::Party::Person(42));
}

fn call(world: &mut World, run: (u64, u64), completion: u32, action: core::NamedAction) -> core::CallPart {
    let key = core::CallKey { task: run.0, attempt: run.1, completion, position: 0 };
    let to = Token::new(4300 + u64::from(completion));
    world.send(root::Event::Core(core::Event::NamedAction { to: ReplyTo::new(to), key, action }));
    let (found, named, part) = world.answers.last().expect("decided answer");
    assert_eq!((*found, *named), (to, key));
    let root::Record::Core(core::Record::Core(core::CoreRecord::Call(record))) =
        &world.store.rows[&root::Key::Core(core::Key::Core(core::CoreKey::Call(key)))]
    else {
        panic!("answer is durable")
    };
    assert_eq!(&record.part, part);
    part.clone()
}

fn note(name: u64, recalled: Option<u32>, records: Box<[u64]>, lines: bool) -> core::NamedAction {
    let entry = notes::New {
        name,
        scope: notes::Scope::Project { project: 1 },
        description: b"plan".as_slice().into(),
        body: b"carry this into the plan".as_slice().into(),
        references: List::with_capacity(0),
        author: notes::Author::Task { task: 0, attempt: 0 },
    };
    if lines {
        core::NamedAction::NoteLines {
            entry,
            recalled: recalled.expect("line edit names a revision"),
            range: notes::LineRange { first: 1, remove: 1, join: false },
            records,
        }
    } else {
        core::NamedAction::Note { entry, recalled, records }
    }
}

#[test]
fn mark_read_records_and_replays_without_an_idle_words_wake() {
    let (mut world, run) = goal(941, true);
    world.tool_call_at(2, b"mark_read", br#"{"messages":[77,5]}"#, true);
    let expected = core::SettledAnswer::Host {
        error: false,
        body: b"recorded: 77\nnamed none of your unrecorded words: 5".as_slice().into(),
    };
    assert_eq!(world.host_answers.last().expect("rendered answer").1.answer, expected);
    assert_eq!(
        world.answers.last().expect("core answer").2,
        core::CallPart::WordsRecorded { recorded: Box::new([77]), unknown: Box::new([5]) }
    );
    assert!(fixtures::live_row(&world, run.0).unrecorded.is_empty());
    let commits = world.store.applied;
    world.tool_call_at(2, b"mark_read", br#"{"messages":[77,5]}"#, true);
    assert_eq!(world.host_answers.last().expect("replayed answer").1.answer, expected);
    assert_eq!(world.store.applied, commits, "replay decides nothing again");
    let assignments = world.assignments.len();
    per_wake::park(&mut world, run, None);
    assert_eq!(world.assignments.len(), assignments);
    assert_eq!(fixtures::live_row(&world, run.0).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(fixtures::live_row(&world, run.0).words_woken, 0);
    world.close();
}

#[test]
fn note_records_only_after_written_in_whole_and_line_forms() {
    for lines in [false, true] {
        let (mut world, run) = goal(942, true);
        let first = call(&mut world, run, 1, note(0, None, Box::new([]), false));
        let core::CallPart::NoteWritten { name, revision: 1, .. } = first else { panic!("initial note: {first:?}") };
        assert_eq!(fixtures::live_row(&world, run.0).unrecorded.len(), 1);
        let answer = call(&mut world, run, 2, note(name, Some(1), Box::new([77, 5]), lines));
        assert!(
            matches!(&answer, core::CallPart::NoteWritten { revision: 2, recorded, unknown, .. }
            if recorded.as_ref() == [77] && unknown.as_ref() == [5]),
            "{answer:?}"
        );
        assert!(fixtures::live_row(&world, run.0).unrecorded.is_empty());
        assert_eq!(call(&mut world, run, 2, note(name, Some(1), Box::new([77, 5]), lines)), answer);
        world.close();
    }
}

#[test]
fn a_stale_note_records_nothing_in_whole_and_line_forms() {
    for lines in [false, true] {
        let (mut world, run) = goal(943, true);
        let first = call(&mut world, run, 1, note(0, None, Box::new([]), false));
        let core::CallPart::NoteWritten { name, .. } = first else { panic!("initial note") };
        call(&mut world, run, 2, note(name, Some(1), Box::new([]), false));
        assert_eq!(
            call(&mut world, run, 3, note(name, Some(1), Box::new([77]), lines)),
            core::CallPart::NoteRefused(notes::Refusal::Moved { revision: 2 })
        );
        assert_eq!(fixtures::live_row(&world, run.0).unrecorded[0].number, 77);
        world.close();
    }
}

#[test]
fn mark_read_requires_the_message_family_but_no_reference() {
    let (mut world, run) = goal(944, false);
    let marked = call(&mut world, run, 1, core::NamedAction::MarkRead { messages: Box::new([77]) });
    assert!(
        matches!(&marked, core::CallPart::ToolDenied { findings, .. }
        if findings.contains(&authority::Finding::Tool)),
        "{marked:?}"
    );
    assert_eq!(fixtures::live_row(&world, run.0).unrecorded.len(), 1);
    let requester = fixtures::live_row(&world, run.0).requester;
    let tasks::Party::Task(target) = requester else { panic!("delegate requester") };
    assert_eq!(
        call(
            &mut world,
            run,
            2,
            core::NamedAction::Message {
                target,
                kind: tasks::MessageKind::Words,
                words: b"question".as_slice().into()
            }
        ),
        marked
    );
    world.close();
}

#[test]
fn mark_read_records_only_its_runs_own_task() {
    let (mut world, run) = goal(946, true);
    let mut member = fixtures::member(&world, tasks::Executor::Agent { charter: 1 }, b"another task's words");
    member.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    let child = fixtures::delegate(&mut world, run, 2, member);
    let other = (child, fixtures::live_row(&world, child).attempt);
    directing_words(&mut world, other);
    assert!(!fixtures::live_row(&world, run.0).references.contains(&run.0));
    assert_eq!(
        call(&mut world, run, 3, core::NamedAction::MarkRead { messages: Box::new([77]) }),
        core::CallPart::WordsRecorded { recorded: Box::new([77]), unknown: Box::new([]) }
    );
    assert!(fixtures::live_row(&world, run.0).unrecorded.is_empty());
    assert_eq!(fixtures::live_row(&world, other.0).unrecorded[0].number, 77);
    world.close();
}

#[test]
fn mark_read_and_note_refuse_more_names_than_the_inbox_holds() {
    let (mut world, run) = goal(945, true);
    let limit = usize::try_from(world.env().limits.core.tasks.inbox_messages).unwrap();
    for (completion, messages) in [(1, vec![]), (2, vec![0]), (3, vec![77; limit + 1])] {
        let answer = call(&mut world, run, completion, core::NamedAction::MarkRead { messages: messages.into() });
        assert!(matches!(answer, core::CallPart::MessageRefused(tasks::Problem { why: tasks::Refusal::Read, .. })));
        assert_eq!(fixtures::live_row(&world, run.0).unrecorded.len(), 1);
    }
    for lines in [false, true] {
        let answer =
            call(&mut world, run, if lines { 5 } else { 4 }, note(1, Some(1), vec![77; limit + 1].into(), lines));
        assert!(matches!(answer, core::CallPart::MessageRefused(tasks::Problem { why: tasks::Refusal::Read, .. })));
        assert_eq!(fixtures::live_row(&world, run.0).unrecorded.len(), 1);
    }
    world.close();
}
