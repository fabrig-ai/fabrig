//! Live wait windows and their durable release (jig/docs/design/client/domain/contract/reads.md, 1.2).
use jig_core_world::fixtures::{PARTY, ask_as, delegate, member, second_party, sign_in};
use jig_core_world::reads::{Parties, ask, latest};
use jig_core_world::world::{Cut, Setup, World};
use jig_domain_contract as jig;
use jig_domain_core::{self as core, AtBound, Bound, Unbounded, WaitKind, WaitObject, documents, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_connector::ProcedureAction;
use jig_test_domain as root;
use skein_lib::{Duration, List, ReplyTo, Time, Token, Wall};

const PROJECT: u32 = 1;
const QUERY: reads::WaitQuery = reads::WaitQuery { project: PROJECT, kind: None, party: None, due: None };
const AT: Wall = Wall::from_nanos(1_000_000_000);

fn world(seed: u64, rows: u32) -> World {
    let (mut configuration, mut limits) = jig_core_world::faults::plan_fixture_with_document_rows(seed, rows);
    configuration.first.procedures[0].actions = Box::new([ProcedureAction::Stall]);
    limits.core.tasks.proposal_stall = Duration::from_secs(2);
    limits.core.waits.question = Duration::from_secs(10);
    limits.core.tasks.dependencies = 2;
    limits.core.tasks.inbox_messages = 16;
    limits.core.tasks.inbox_bytes = 4096;
    let mut world = World::new(Setup {
        seed,
        configuration: Some(configuration),
        limits,
        clock: Some((Time::from_nanos(AT.as_nanos()), AT)),
        ..Setup::default()
    });
    world.reads = Some(Parties::new(world.store.applied));
    world
}

fn time(seconds: u64) -> Wall {
    Wall::from_nanos(Duration::from_secs(seconds).as_nanos())
}

fn page(world: &mut World, read: u64, query: reads::WaitQuery, after: Option<reads::Cursor>) -> reads::WaitPage {
    ask(world, read, reads::Read::Waits { query, after });
    let reads::Answer::Waits(page) = latest(world) else { panic!("wait page: {:?}", latest(world)) };
    assert!(world.reads.as_ref().expect("read parties").loads.is_empty(), "waits never use the store");
    assert!(page.rows.iter().all(|row| row.chip.number == row.wait.task));
    page.as_ref().clone()
}

fn agent(world: &World, title: &[u8]) -> core::Delegate {
    member(world, tasks::Executor::Agent { charter: 1 }, title)
}

fn question(world: &mut World, asker: u64, requester: u64) -> u64 {
    let &(task, attempt) = world.assignments.iter().find(|run| run.0 == asker).expect("asker's run");
    world.send(root::Event::Core(core::Event::NamedAction {
        to: ReplyTo::new(Token::new(401)),
        key: core::CallKey { task, attempt, completion: 1, position: 0 },
        action: core::NamedAction::Message {
            target: requester,
            kind: tasks::MessageKind::Question,
            words: b"what next?".as_slice().into(),
        },
    }));
    let Some((_, _, core::CallPart::Sent { message })) = world.answers.last() else {
        panic!("question was not admitted: {:?}", world.answers.last())
    };
    *message
}

fn report_step(world: &mut World, task: u64) {
    let waits = [
        core::Reported {
            kind: WaitKind::Step,
            since: AT,
            bound: core::ReportedBound::At { at: time(2), then: AtBound::GiveUp },
            object: WaitObject::Procedure { step: 2, things: List::with_capacity(0) },
        },
        core::Reported {
            kind: WaitKind::SignOff,
            since: AT,
            bound: core::ReportedBound::At { at: time(5), then: AtBound::Overdue },
            object: WaitObject::Procedure { step: 3, things: List::with_capacity(0) },
        },
        core::Reported {
            kind: WaitKind::Step,
            since: AT,
            bound: core::ReportedBound::At { at: time(6), then: AtBound::Hold },
            object: WaitObject::Procedure { step: 4, things: List::with_capacity(0) },
        },
    ];
    world.send(root::Event::connector_up(
        1,
        jig::Up::Procedures(jig::procedures::Up::Waits { task, waits: List::from_boxed(Box::new(waits)) }),
    ));
}

struct Project {
    world: World,
    root: u64,
    asker: u64,
    procedure: u64,
    bounded: u64,
    unbounded: u64,
    proposal: u64,
    question: u64,
    first: Wall,
}

fn project(seed: u64, rows: u32) -> Project {
    let mut world = world(seed, rows);
    let root = world.assignments[0];
    let proposal = world.propose(1);
    let owner = world.sign_in.expect("signed-in owner");
    ask_as(
        &mut world,
        owner,
        41,
        people::Ask::DecideProposal {
            project: PROJECT,
            proposer: people::Proposer::Task(root.0),
            proposal,
            decision: people::ProposalDecision::Pass,
            running: None,
        },
    );
    let asker = agent(&world, b"ask the requester");
    let asker = delegate(&mut world, root, 2, asker);
    let message = question(&mut world, asker, root.0);
    let procedure = member(&world, tasks::Executor::Procedure { connector: 1, code: 1 }, b"report two kinds");
    let procedure = delegate(&mut world, root, 3, procedure);
    report_step(&mut world, procedure);
    let mut bounded = agent(&world, b"wait for two dependencies");
    bounded.authority.budget.deadline = Some(time(6));
    bounded.dependencies = Box::new([core::Dependency::Existing(procedure), core::Dependency::Existing(asker)]);
    let bounded = delegate(&mut world, root, 4, bounded);
    let asker_run = world.assignments.iter().copied().find(|run| run.0 == asker).expect("asker has a run");
    let mut unbounded = agent(&world, b"wait without a clock");
    unbounded.authority.budget.spend = 1;
    unbounded.authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    unbounded.dependencies = Box::new([core::Dependency::Existing(procedure)]);
    let unbounded = delegate(&mut world, asker_run, 2, unbounded);
    let first = Wall::from_nanos(time(3).as_nanos() + 1);
    world.set_clock(Time::from_nanos(first.as_nanos()), first);
    Project { world, root: root.0, asker, procedure, bounded, unbounded, proposal, question: message, first }
}

fn order(rows: &[reads::WaitRow]) -> Vec<(u64, WaitKind)> {
    rows.iter().map(|row| (row.wait.task, row.wait.kind)).collect()
}

impl Project {
    fn expected(&self) -> [(u64, WaitKind); 8] {
        [
            (self.root, WaitKind::Party),
            (self.procedure, WaitKind::Step),
            (self.procedure, WaitKind::SignOff),
            (self.procedure, WaitKind::Step),
            (self.bounded, WaitKind::Dependency),
            (self.bounded, WaitKind::Dependency),
            (self.asker, WaitKind::Question),
            (self.unbounded, WaitKind::Dependency),
        ]
    }
}

#[test]
fn a_projects_waits_order_whole_own_records_by_group_bound_and_task() {
    let mut story = project(18401, 16);
    let page = page(&mut story.world, 1, QUERY, None);
    assert_eq!(page.now, story.first);
    assert_eq!(order(page.rows.as_slice()), story.expected());
    assert!(page.after.is_none());
    let rows = page.rows.as_slice();
    let overdue = &rows[0].wait;
    assert_eq!(overdue.bound, Bound::At { at: time(3), then: AtBound::Overdue });
    assert_eq!(
        overdue.object,
        WaitObject::Entry {
            kind: people::EntryKind::Proposal { proposer: people::Proposer::Task(story.root), number: story.proposal },
            at: tasks::EscalationHolder::Role { project: PROJECT, role: people::Role::Owner.number() },
        }
    );
    for (wait, dependency) in [(&rows[4].wait, story.asker), (&rows[5].wait, story.procedure)] {
        assert_eq!(wait.bound, Bound::At { at: time(6), then: AtBound::Hold });
        assert_eq!(wait.object, WaitObject::Task(dependency));
    }
    assert_eq!(rows[6].wait.object, WaitObject::Question { number: story.question, answerer: story.root });
    assert!(matches!(rows[6].wait.bound, Bound::At { then: AtBound::PassUp, .. }));
    assert_eq!(rows[7].wait.bound, Bound::None(Unbounded::NoDeadline));
    let due = time(6).as_nanos() - story.first.as_nanos();
    let due_query = reads::WaitQuery { due: Some(due), ..QUERY };
    let due_page = self::page(&mut story.world, 2, due_query, None);
    assert_eq!(order(due_page.rows.as_slice()), story.expected()[..6]);

    second_party(&mut story.world, b"another-owner", people::Role::Owner);
    let Some(people::Reply::SignedIn { person, .. }) =
        story.world.people_answers.iter().rev().find(|reply| matches!(reply, people::Reply::SignedIn { .. }))
    else {
        panic!("second owner signed in")
    };
    assert_ne!(*person, PARTY);
    assert_eq!(overdue.hears, Some(tasks::EscalationHolder::Person(PARTY)));
    let role_query = reads::WaitQuery { party: Some(*person), ..due_query };
    let role_page = self::page(&mut story.world, 3, role_query, None);
    assert_eq!(
        order(role_page.rows.as_slice()),
        [(story.root, WaitKind::Party)],
        "entry waits at a role, not at its hearer"
    );
}

#[test]
fn waits_pages_keep_their_first_wall_and_reject_a_foreign_cursor() {
    let mut story = project(18402, 2);
    let query = QUERY;
    let mut first = page(&mut story.world, 1, query, None);
    assert_eq!(first.now, story.first);
    let mut rows = order(first.rows.as_slice());
    let mut after = first.after.take();
    assert_eq!(rows.as_slice(), &story.expected()[..2]);
    assert!(after.is_some());
    story.world.set_clock(Time::from_nanos(time(5).as_nanos()), time(5));
    let next = page(&mut story.world, 2, query, after.take());
    assert_eq!(next.now, story.first, "the sign-off crossed its bound between pages");
    rows.extend(order(next.rows.as_slice()));
    assert!(next.after.is_some());
    for (read, after, query) in [
        (3, next.after.clone(), reads::WaitQuery { due: Some(0), ..QUERY }),
        (
            4,
            Some(reads::Cursor::Live {
                scope: reads::Scope::Tasks {
                    query: documents::TaskQuery {
                        project: Some(PROJECT),
                        requester: None,
                        root: None,
                        parent: None,
                        label: None,
                        phase: None,
                        executor: None,
                        tracked: None,
                        order: documents::QueryOrder::Number,
                        parts: List::with_capacity(0),
                    },
                },
                after: documents::Cursor::Number { number: story.root },
                store_after: None,
            }),
            QUERY,
        ),
    ] {
        ask(&mut story.world, read, reads::Read::Waits { query, after });
        assert_eq!(latest(&story.world), &reads::Answer::None(reads::Why::Malformed));
    }
    let mut after = next.after;
    for read in 5..=6 {
        let next = page(&mut story.world, read, query, after.take());
        assert_eq!(next.now, story.first);
        rows.extend(order(next.rows.as_slice()));
        after = next.after;
    }
    assert_eq!(rows, story.expected());
    assert!(after.is_none());
}

#[test]
fn a_delegates_party_wait_appears_once_and_obeys_the_filters_and_visibility() {
    let mut world = world(18403, 16);
    world.park_for_message(0);
    let news_query = reads::WaitQuery { kind: Some(WaitKind::News), party: Some(PARTY), ..QUERY };
    let heard = page(&mut world, 90, news_query, None);
    let [row] = heard.rows.as_slice() else { panic!("one news wait for its escalation holder") };
    assert!(matches!(row.wait.object, WaitObject::News { .. }));
    assert_eq!(row.wait.hears, Some(tasks::EscalationHolder::Person(PARTY)));
    world.say(42);
    let root = *world.assignments.last().expect("requester's resumed run");
    let child = agent(&world, b"ask for a decision");
    let child = delegate(&mut world, root, 1, child);
    let message = question(&mut world, child, root.0);
    let task = world.domain.core().waits(child).own;
    let Bound::At { at, then: AtBound::PassUp } = task.as_slice()[0].bound else { panic!("task-held question") };
    let (now, wall) = world.clocks();
    world.fire_due(now.saturating_add(Duration::from_nanos(at.as_nanos() - wall.as_nanos())), at);
    let who = jig_core_world::fixtures::PARTY;
    let party = reads::WaitQuery { party: Some(who), kind: Some(WaitKind::Party), ..QUERY };
    let answer = page(&mut world, 1, party, None);
    let [row] = answer.rows.as_slice() else { panic!("exactly one holder wait: {:?}", answer.rows) };
    assert_eq!(row.wait.task, child);
    assert_eq!(
        row.wait.object,
        WaitObject::Entry { kind: people::EntryKind::Question { message }, at: tasks::EscalationHolder::Person(who) }
    );
    let goal = world.domain.core().waits(root.0);
    assert_eq!(goal.first, Some(row.wait.clone()), "the ancestor summarizes the same wait");
    for (i, query) in [
        reads::WaitQuery { kind: Some(WaitKind::Question), ..party },
        reads::WaitQuery { party: Some(u64::MAX), ..party },
        reads::WaitQuery { due: Some(0), ..party },
    ]
    .into_iter()
    .enumerate()
    {
        assert!(page(&mut world, 2 + u64::try_from(i).expect("three filter cases"), query, None).rows.is_empty());
    }
    world.send(sign_in(Token::new(8300), 0, b"guest", b"guest", b"Guest"));
    let Some(people::Reply::SignedIn { sign_in, person, .. }) = world.people_answers.last() else {
        panic!("party signed in: {:?}", world.people_answers.last())
    };
    assert!(world.domain.core().people.role(*person, PROJECT).is_none());
    world.sign_in = Some(*sign_in);
    ask(&mut world, 5, reads::Read::Waits { query: QUERY, after: None });
    assert_eq!(latest(&world), &reads::Answer::None(reads::Why::Unknown));
}

#[test]
fn a_waits_page_is_held_until_its_preceding_commit_is_durable() {
    let mut world = world(18404, 2);
    let task = world.assignments[0].0;
    let sign_in = world.sign_in.expect("signed-in owner");
    let number = world.store.applied + 1;
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    world.cut = Some(Cut::Submitted(number));
    ask_as(
        &mut world,
        sign_in,
        1,
        people::Ask::Say { project: PROJECT, task, words: b"not yet durable".as_slice().into() },
    );
    assert_eq!(world.reached_cut, Some(Cut::Submitted(number)));
    ask(&mut world, 1, reads::Read::Waits { query: QUERY, after: None });
    assert_eq!(world.reads.as_ref().expect("read parties").receipts.len(), 0);
    let committed = world.store.release_held().expect("one held commit completion");
    world.send(root::Event::Committed { number: committed, fill: 0 });
    let receipts = &world.reads.as_ref().expect("read parties").receipts;
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].1, number);
    assert!(matches!(&receipts[0].2, reads::Answer::Waits(_)));
}
