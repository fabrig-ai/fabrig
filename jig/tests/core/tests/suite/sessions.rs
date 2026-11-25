//! Standing sessions through the real preparation, claim and store barrier (standing.md, 5).
use jig_core_world::{
    fixtures,
    fixtures::live_row,
    stories::per_wake as story,
    world::{Setup, World},
};
use jig_domain_core as core;
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

const SEED: u64 = 925;
const FIRST: &[u8] = b"S\n\nF";
const LATER: &[u8] = b"S\n\nWR";

fn setup() -> Setup {
    let mut setup = story::setup(SEED, true);
    let config = setup.configuration.as_mut().expect("session configuration");
    config.core.settings.models[0].cache_life_ms = 300_000;
    config.core.defaults.charters[0].policy.model.cache_life_ms = 300_000;
    let charter = &mut config.core.defaults.charters[1];
    charter.policy.resume = true;
    charter.policy.model.cache_life_ms = 300_000;
    charter.parts =
        [(tasks::Reasons::FIRST, b"F".as_slice()), (tasks::Reasons::WORDS.union(tasks::Reasons::RESULT), b"WR")]
            .into_iter()
            .map(|(on, instructions)| core::CharterPart { on, instructions: instructions.into() })
            .collect();
    setup
}

fn start(standing: bool) -> (World, (u64, u64)) {
    start_with(standing, setup())
}

fn start_with(standing: bool, setup: Setup) -> (World, (u64, u64)) {
    let mut world = World::new(setup);
    let caller = fixtures::run(&world);
    story::park(&mut world, caller, None);
    let people::Ask::Create { mut batch, .. } = fixtures::chat(&world, 1, b"keep watch".as_slice().into()) else {
        unreachable!()
    };
    let member = &mut batch[0];
    member.executor = people::TaskExecutor::Agent { charter: 2 };
    member.flags.conversation = !standing;
    member.flags.ended_by = if standing { people::EndedBy::Standing } else { people::EndedBy::Requester };
    member.authority.spend = 60;
    member.budget = people::Budget::Named;
    fixtures::ask(&mut world, 79, people::Ask::Create { project: 1, batch });
    let people::Reply::Outcome(people::Outcome::Created { first: task, .. }) =
        world.people_answers.last().expect("task creation reply")
    else {
        panic!("standing task creation: {:?}", world.people_answers.last());
    };
    let run = (*task, live_row(&world, *task).attempt);
    (world, run)
}

pub(super) fn opened() -> (World, (u64, u64), (u64, u64)) {
    let (mut world, run) = start(true);
    assert_eq!(world.assigned.as_ref().expect("standing assignment").run.policy.instructions.as_ref(), FIRST);
    assert_eq!(live_row(&world, run.0).session.expect("claimed session").first, run.1);
    let child = story::child(&mut world, run);
    story::turn(&mut world, run, None);
    assert_eq!(live_row(&world, run.0).session.expect("committed turn").last, turned_at(&world, run));
    (world, run, child)
}

pub(super) fn gap(world: &mut World, seconds: u64) {
    world.advance(Duration::from_secs(seconds));
    world.send(fixtures::sign_in(Token::new(10_000), 0, &7_u64.to_be_bytes(), b"person", b"Person"));
    world.grant_account(1);
}

pub(super) fn say(world: &mut World, task: u64, key: u8) {
    fixtures::ask(world, key, people::Ask::Say { project: 1, task, words: b"check now".as_slice().into() });
}

fn assignment(world: &World, task: u64, instructions: &[u8], resumed: bool) -> (u64, u64) {
    let assigned = world.assigned.as_ref().expect("task assigned");
    assert_eq!(
        assigned.task,
        task,
        "task phase: {:?}, replies: {:?}",
        live_row(world, task).phase,
        world.people_answers
    );
    assert_eq!(assigned.run.policy.instructions.as_ref(), instructions);
    assert_eq!(!assigned.transcript.is_empty(), resumed, "session ends: {:?}", ends(world, task));
    assert_eq!(assigned.run.policy.resume, resumed);
    if !resumed {
        assert_eq!(assigned.conversation, assigned.attempt);
    }
    (task, assigned.attempt)
}

pub(super) fn ends(world: &World, task: u64) -> Vec<(u64, tasks::SessionEnd)> {
    world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == task => {
                match line.line {
                    tasks::Line::SessionEnded { attempt, end } => Some((attempt, end)),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect()
}

fn activity(world: &World, task: u64, after: u64) -> impl DoubleEndedIterator<Item = &tasks::ActivityLine> {
    use std::ops::Bound::{Excluded, Included};
    let key = |position| root::Key::Core(core::Key::Tasks(tasks::Key::Activity { task, position }));
    world.store.rows.range((Excluded(key(after)), Included(key(u64::MAX)))).map(|(_, row)| {
        let root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) = row else {
            panic!("activity range")
        };
        line
    })
}

pub(super) fn claimed_at(world: &World, task: u64, attempt: u64) -> skein_lib::Wall {
    activity(world, task, 0)
        .rev()
        .find_map(|line| match line.line {
            tasks::Line::RunBegan { attempt: began, .. } if began == attempt => Some(line.at),
            _ => None,
        })
        .expect("claim's activity time")
}

fn turned_at(world: &World, run: (u64, u64)) -> skein_lib::Wall {
    activity(world, run.0, 0)
        .rev()
        .find_map(|line| match line.line {
            tasks::Line::Turn { attempt, turn: 1, .. } if attempt == run.1 => Some(line.at),
            _ => None,
        })
        .expect("turn's activity time")
}

pub(super) struct TalkPairs {
    task: u64,
    after: u64,
    open: Option<u64>,
    ended: Vec<(u64, bool)>,
    pub(super) saw_lapse: bool,
}

impl TalkPairs {
    pub(super) fn new(task: u64) -> Self {
        Self { task, after: 0, open: None, ended: Vec::new(), saw_lapse: false }
    }

    pub(super) fn check(&mut self, world: &World) {
        for line in activity(world, self.task, self.after) {
            match line.line {
                tasks::Line::TalkOpened { party } => assert_eq!(self.open.replace(party), None),
                tasks::Line::TalkEnded { party, lapsed } => {
                    assert_eq!(self.open.take(), Some(party), "each opened talk ends exactly once");
                    self.ended.push((party, lapsed));
                    self.saw_lapse |= lapsed;
                }
                _ => {}
            }
            self.after = line.position;
        }
        assert_eq!(self.open, live_row(world, self.task).talk.map(|talk| talk.party));
    }
}

pub(super) fn consume_and_park(world: &mut World, run: (u64, u64)) {
    let read = live_row(world, run.0).inbox.last().map(|word| word.number);
    story::turn(world, run, read);
    if let Some(session) = live_row(world, run.0).session {
        assert_eq!(session.last, turned_at(world, run));
    }
    story::park(world, run, read);
}

#[test]
fn incremental_talk_pairs_and_activity_times_match_a_whole_scan() {
    let (mut world, mut run, _) = opened();
    let mut pairs = TalkPairs::new(run.0);
    let mut rng = skein_lib::Rng::new(51);
    for step in 0..4_u8 {
        consume_and_park(&mut world, run);
        gap(&mut world, 1_801 + rng.below(60));
        talk(&mut world, run.0, 100 + step);
        run = fixtures::run(&world);
        pairs.check(&world);
        pairs.check(&world); // No new activity must leave the evidence unchanged.
    }
    let mut open = None;
    let mut ended = Vec::new();
    let mut claims = std::collections::BTreeMap::new();
    let mut turns = std::collections::BTreeMap::new();
    let mut positions = Vec::new();
    for row in world.store.rows.values() {
        if let root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) = row
            && line.task == run.0
        {
            positions.push(line.position);
            match line.line {
                tasks::Line::TalkOpened { party } => assert_eq!(open.replace(party), None),
                tasks::Line::TalkEnded { party, lapsed } => {
                    assert_eq!(open.take(), Some(party));
                    ended.push((party, lapsed));
                }
                tasks::Line::RunBegan { attempt, .. } => {
                    claims.entry(attempt).or_insert(line.at);
                }
                tasks::Line::Turn { attempt, turn: 1, .. } => {
                    turns.entry(attempt).or_insert(line.at);
                }
                _ => {}
            }
        }
    }
    assert_ne!(ended.len(), 0);
    assert_eq!(pairs.ended, ended);
    assert_eq!(pairs.open, open);
    assert_eq!(pairs.saw_lapse, ended.iter().any(|(_, lapsed)| *lapsed));
    assert_eq!(Some(pairs.after), positions.last().copied());
    assert_eq!(activity(&world, run.0, 0).map(|line| line.position).collect::<Vec<_>>(), positions);
    for (attempt, at) in claims {
        assert_eq!(claimed_at(&world, run.0, attempt), at);
    }
    for (attempt, at) in turns {
        assert_eq!(turned_at(&world, (run.0, attempt)), at);
    }
    world.close();
}

#[test]
fn a_warm_wake_keeps_the_transcript_instructions_and_session() {
    let (mut world, run, _) = opened();
    assert_began(&world, run, None, None);
    let session = live_row(&world, run.0).session;
    story::park(&mut world, run, None);
    assert!(!live_row(&world, run.0).activation_open);
    gap(&mut world, 269);
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, FIRST, true);
    assert_began(&world, next, None, None);
    assert_eq!(live_row(&world, run.0).session, session);
    assert_eq!(ends(&world, run.0).len(), 0);
    assert!(live_row(&world, run.0).activation_open);
    world.close();
}

#[test]
fn a_quiet_wake_commits_the_end_and_restarts_the_session_and_tries() {
    let (mut world, run, _) = opened();
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Failed(tasks::Class::Transient),
        read: None,
        refused: Box::new([]),
    });
    world.retry_due();
    let retry = (run.0, live_row(&world, run.0).attempt);
    assert_eq!(live_row(&world, run.0).tries.transient, 1);
    consume_and_park(&mut world, retry);
    assert_eq!(live_row(&world, run.0).tries.transient, 1, "a standing park keeps the session's tries");
    gap(&mut world, 271);
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, LATER, false);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Quiet)]);
    assert_began(&world, next, Some(tasks::Fresh::Session), Some(tasks::SessionEnd::Quiet));
    assert_eq!(
        live_row(&world, run.0).session,
        Some(tasks::Session {
            first: next.1,
            reasons: tasks::Reasons::WORDS,
            last: claimed_at(&world, next.0, next.1)
        })
    );
    assert_eq!(live_row(&world, run.0).tries, tasks::Tries::NONE);
    world.close();
}

#[test]
fn an_unoffered_result_starts_a_fresh_session_with_its_part() {
    let (mut world, run, child) = opened();
    story::park(&mut world, run, None);
    let result = story::result_from(&mut world, child);
    assignment(&world, run.0, LATER, false);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Unoffered)]);
    assert_eq!(live_row(&world, run.0).session.expect("new session").reasons, tasks::Reasons::RESULT);
    assert!(world.assigned.as_ref().unwrap().messages.contains(&result));
    world.close();
}

#[test]
fn closed_work_ends_at_the_park_despite_a_timer_and_subscription() {
    let (mut world, run) = start(true);
    story::turn(&mut world, run, None);
    world.send(root::Event::Core(core::Event::Tasks(tasks::Event::Subscribe {
        reads_project: false,
        reply_to: ReplyTo::new(Token::new(985)),
        task: run.0,
        subscription: tasks::Subscription {
            number: 985,
            kind: tasks::SubscriptionKind::Task { target: 1, held: true, result: true },
        },
    })));
    story::park(&mut world, run, None);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Closed)]);
    assert!(live_row(&world, run.0).session.is_none());
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, LATER, false);
    assert_eq!(live_row(&world, run.0).session.expect("new session").first, next.1);
    world.close();
}

#[test]
fn retries_and_an_interrupted_attempt_are_not_judged_as_wakes() {
    let (mut world, run, _) = opened();
    gap(&mut world, 271);
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: run.0,
        attempt: run.1,
        cumulative: 0,
        end: tasks::End::Failed(tasks::Class::Transient),
        read: None,
        refused: Box::new([]),
    });
    world.retry_due();
    let retry = assignment(&world, run.0, FIRST, true);
    assert_eq!(live_row(&world, run.0).session.expect("same session").first, run.1);
    assert_eq!(live_row(&world, run.0).tries.transient, 1);
    assert_eq!(ends(&world, run.0).len(), 0);
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: retry.0,
        attempt: retry.1,
        cumulative: 0,
        end: tasks::End::Recheck,
        read: None,
        refused: Box::new([]),
    });
    assignment(&world, run.0, FIRST, true);
    assert_eq!(live_row(&world, run.0).session.expect("interrupted session").first, run.1);
    assert_eq!(live_row(&world, run.0).tries.transient, 1);
    assert_eq!(ends(&world, run.0).len(), 0);
    world.close();
}

pub(super) fn restart(world: &mut World, child: (u64, u64)) {
    world.restart_with(setup().configuration.expect("same configuration"));
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: jig_domain_link::Hello {
            fresh: false,
            reads_usage: true,
            jobs: Box::new([]),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([jig_domain_link::Hosting {
                run: Token::new(child.0),
                attempt: Token::new(child.1),
                phase: jig_domain_link::Phase::Active,
            }]),
        },
    })));
    world.grant_account(1);
}

#[test]
fn a_restart_between_parks_preserves_the_record_and_resumes_within_the_span() {
    let (mut world, run, child) = opened();
    story::park(&mut world, run, None);
    let record = live_row(&world, run.0).clone();
    restart(&mut world, child);
    assert_eq!(live_row(&world, run.0), &record);
    gap(&mut world, 1);
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, FIRST, true);
    consume_and_park(&mut world, next);
    assert_eq!(live_row(&world, run.0).session.expect("same session").first, run.1);
    assert_eq!(ends(&world, run.0).len(), 0);
    world.close();
}

#[test]
fn an_instruction_amendment_ends_rules_only_when_it_commits() {
    let (mut world, run, _) = opened();
    story::park(&mut world, run, None);
    let mut amendment = jig_test_profile::fixtures::person_unchanged(b"new instructions");
    amendment.instructions = Some(b"own".as_slice().into());
    let replies = world.people_answers.len();
    let assignments = world.assignments.len();
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    fixtures::ask(
        &mut world,
        80,
        people::Ask::Amend { project: 1, task: run.0, revision: 0, release: None, amendment },
    );
    assert_eq!(world.people_answers.len(), replies, "no acknowledgment ahead of the commit");
    assert_eq!(world.assignments.len(), assignments, "no next session ahead of the commit");
    world.release_commits();
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Rules)]);
    assignment(&world, run.0, b"own", false);
    world.close();
}

#[test]
fn moving_to_new_settings_ends_rules_and_the_next_run_is_fresh() {
    let (mut world, run, _) = opened();
    story::park(&mut world, run, None);
    let mut charter = setup().configuration.unwrap().core.defaults.charters[1].clone();
    charter.policy.instructions = b"T".as_slice().into();
    world.send(root::Event::Core(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(986)),
        sign_in: world.sign_in.unwrap(),
        key: [81; 16],
        digest: [81; 32],
        change: Box::new(core::settings::Change {
            against: 1,
            items: Box::new([core::settings::Item {
                setting: core::settings::Setting::Charter(2),
                edit: core::settings::Edit::Set(core::settings::Value::Charter(Box::new(charter))),
            }]),
            running: core::settings::Running::Stays,
        }),
    }));
    assert_eq!(world.domain.settings().newest(), 2, "settings reply: {:?}", world.people_answers.last());
    fixtures::ask(&mut world, 82, people::Ask::MoveToNewest { revision: 1 });
    assert_eq!(live_row(&world, run.0).settings.task, 2);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Rules)]);
    assert!(live_row(&world, run.0).session.is_none());
    say(&mut world, run.0, 83);
    assignment(&world, run.0, b"T\n\nWR", false);
    world.close();
}

#[test]
fn an_always_offered_only_wake_is_judged_but_its_empty_reason_retry_is_not() {
    let (mut world, run, _) = opened();
    story::park(&mut world, run, None);
    gap(&mut world, 271);
    let amendment = jig_test_profile::fixtures::person_unchanged(b"check status");
    fixtures::ask(
        &mut world,
        80,
        people::Ask::Amend { project: 1, task: run.0, revision: 0, release: None, amendment },
    );
    let fresh = assignment(&world, run.0, b"S", false);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Quiet)]);
    assert_eq!(live_row(&world, run.0).reasons, tasks::Reasons::EMPTY);
    let session = live_row(&world, run.0).session.expect("empty-reason session");
    assert_eq!(session.reasons, tasks::Reasons::EMPTY);
    let read = live_row(&world, run.0).inbox.last().map(|word| word.number);
    story::turn(&mut world, fresh, read);
    gap(&mut world, 271);
    world.send(root::Event::Answer {
        channel: Token::new(7),
        task: fresh.0,
        attempt: fresh.1,
        cumulative: 0,
        end: tasks::End::Failed(tasks::Class::Transient),
        read,
        refused: Box::new([]),
    });
    world.retry_due();
    assignment(&world, run.0, b"S", true);
    assert_eq!(live_row(&world, run.0).session.expect("same session").first, fresh.1);
    assert_eq!(live_row(&world, run.0).tries.transient, 1);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Quiet)]);
    world.close();
}

#[test]
fn an_engine_stop_preserves_the_activation_and_session() {
    let mut initial = setup();
    initial.configuration.as_mut().unwrap().core.workers_in_process = true;
    let mut config = setup().configuration.unwrap();
    config.core.workers_in_process = true;
    let (mut world, run) = start_with(true, initial);
    story::turn(&mut world, run, None);
    gap(&mut world, 271);
    world.stop();
    assert!(live_row(&world, run.0).activation_open);
    assert_eq!(ends(&world, run.0).len(), 0);
    world.restart_with(config);
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0,
        channel: Token::new(7),
        hello: jig_domain_link::Hello {
            fresh: false,
            reads_usage: true,
            jobs: Box::new([]),
            bounds: jig_test_profile::fixtures::bounds(),
            stop_bound: Duration::from_millis(100),
            slots: 2,
            workstreams: Box::new([]),
            hosting: Box::new([]),
        },
    })));
    world.grant_account(1);
    let next = assignment(&world, run.0, FIRST, true);
    assert!(next.1 > run.1);
    assert_eq!(live_row(&world, run.0).session.expect("interrupted session").first, run.1);
    assert_eq!(ends(&world, run.0).len(), 0);
    consume_and_park(&mut world, next);
    world.close();
}

#[test]
fn a_nonstanding_task_under_a_resuming_charter_reads_no_parts() {
    let (mut world, run) = start(false);
    assert_eq!(world.assigned.as_ref().unwrap().run.policy.instructions.as_ref(), b"S");
    assert_eq!(live_row(&world, run.0).offered, tasks::Reasons::ALL);
    assert!(live_row(&world, run.0).session.is_none());
    consume_and_park(&mut world, run);
    say(&mut world, run.0, 80);
    assignment(&world, run.0, b"S", true);
    assert_eq!(live_row(&world, run.0).offered, tasks::Reasons::ALL);
    world.close();
}

pub(super) fn talk(world: &mut World, task: u64, key: u8) {
    fixtures::ask(world, key, people::Ask::Talk { project: 1, task, words: b"let us talk".as_slice().into() });
    assert!(
        matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::Said { task: answered, .. })) if *answered == task)
    );
}
#[test]
fn talk_words_start_fresh_with_now_and_resume_past_quiet_until_leave() {
    let (mut world, run, _) = opened();
    story::park(&mut world, run, None);
    talk(&mut world, run.0, 80);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Talk)]);
    let first = assignment(&world, run.0, LATER, false);
    let assigned = world.assigned.as_ref().unwrap();
    assert!(assigned.brief.iter().any(|section| matches!(
        section,
        jig_domain_brief::GatherPlaced::Core { kind: jig_domain_brief::Core::Now, .. }
    )));
    assert!(!assigned.brief.iter().any(|section| matches!(
        section,
        jig_domain_brief::GatherPlaced::Core { kind: jig_domain_brief::Core::TranscriptTail, .. }
    )));
    consume_and_park(&mut world, first);
    assert_eq!(live_row(&world, run.0).session.unwrap().first, first.1, "work is closed but the talk stays open");
    gap(&mut world, 600);
    talk(&mut world, run.0, 82);
    let next = assignment(&world, run.0, LATER, true);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Talk)]);
    fixtures::ask(&mut world, 83, people::Ask::Leave { project: 1, task: run.0 });
    assert!(
        matches!(world.people_answers.last(), Some(people::Reply::Outcome(people::Outcome::Left { task })) if *task == run.0)
    );
    assert!(live_row(&world, run.0).talk.is_none());
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Talk), (first.1, tasks::SessionEnd::Left)]);
    consume_and_park(&mut world, next);
    say(&mut world, run.0, 84);
    assignment(&world, run.0, LATER, false);
    world.close();
}

#[test]
fn a_talk_lapses_at_a_wake_and_a_settings_move_keeps_the_talk_not_the_session() {
    let (mut world, run, _) = opened();
    story::park(&mut world, run, None);
    talk(&mut world, run.0, 80);
    let first = assignment(&world, run.0, LATER, false);
    consume_and_park(&mut world, first);
    let talk_record = live_row(&world, run.0).talk;
    let document = core::documents::task_from_row(
        world.domain.core(),
        &setup().limits.core,
        1,
        live_row(&world, run.0),
        world.wall_time(),
    )
    .unwrap();
    assert_eq!(document.talk, talk_record);
    assert_eq!(document.session, live_row(&world, run.0).session);
    for act in [core::acts::Act::Talk, core::acts::Act::Leave] {
        assert!(
            document
                .acts
                .iter()
                .any(|offered| offered.act == act && offered.admission == core::acts::Admission::Allowed)
        );
    }

    let mut charter = setup().configuration.unwrap().core.defaults.charters[1].clone();
    charter.policy.instructions = b"T".as_slice().into();
    world.send(root::Event::Core(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(986)),
        sign_in: world.sign_in.unwrap(),
        key: [82; 16],
        digest: [82; 32],
        change: Box::new(core::settings::Change {
            against: 1,
            items: Box::new([core::settings::Item {
                setting: core::settings::Setting::Charter(2),
                edit: core::settings::Edit::Set(core::settings::Value::Charter(Box::new(charter))),
            }]),
            running: core::settings::Running::Stays,
        }),
    }));
    fixtures::ask(&mut world, 83, people::Ask::MoveToNewest { revision: 1 });
    assert_eq!(ends(&world, run.0).last(), Some(&(first.1, tasks::SessionEnd::Rules)));
    assert_eq!(live_row(&world, run.0).talk, talk_record);
    say(&mut world, run.0, 84);
    let fresh = assignment(&world, run.0, b"T\n\nWR", false);
    consume_and_park(&mut world, fresh);
    gap(&mut world, 1_801);
    say(&mut world, run.0, 85);
    assignment(&world, run.0, b"T\n\nWR", false);
    assert_eq!(ends(&world, run.0).last(), Some(&(fresh.1, tasks::SessionEnd::Left)));
    assert!(live_row(&world, run.0).talk.is_none());
    world.close();
}

fn assert_began(world: &World, run: (u64, u64), fresh: Option<tasks::Fresh>, end: Option<tasks::SessionEnd>) {
    let lines: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|row| match row {
            root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == run.0 => Some(line),
            _ => None,
        })
        .collect();
    let began = lines
        .iter()
        .find(|line| matches!(line.line, tasks::Line::RunBegan { attempt, .. } if attempt == run.1))
        .expect("claim's beginning");
    assert!(matches!(began.line, tasks::Line::RunBegan { fresh: cause, .. } if cause == fresh));
    if let Some(end) = end {
        assert!(
            lines.iter().any(|line| line.position + 1 == began.position
                && matches!(line.line, tasks::Line::SessionEnded { end: cause, .. } if cause == end)),
            "session end immediately precedes beginning"
        );
    }
}

#[test]
fn a_bound_session_records_why_its_next_run_began_fresh() {
    let mut setup = setup();
    let config = setup.configuration.as_mut().expect("session configuration");
    config.core.settings.models[0].window = 1;
    config.core.defaults.charters[0].policy.model.window = 1;
    config.core.defaults.charters[1].policy.model.window = 1;
    let (mut world, run) = start_with(true, setup);
    let _child = story::child(&mut world, run);
    story::turn(&mut world, run, None);
    story::park(&mut world, run, None);
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, LATER, false);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Bound)]);
    assert_began(&world, next, Some(tasks::Fresh::Session), Some(tasks::SessionEnd::Bound));
    world.close();
}

#[test]
fn a_moved_standing_window_records_moved_after_ending_its_session() {
    let (mut world, run, child) = opened();
    story::park(&mut world, run, None);
    let mut proof = world.domain.core().proofs.get(&run.0).expect("saved run proof").clone();
    proof.endpoint = Some(2);
    world.seed(Box::new([root::Record::Core(core::Record::Core(core::CoreRecord::RunProof(proof)))]));
    restart(&mut world, child);
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, LATER, false);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Fresh)]);
    assert_began(&world, next, Some(tasks::Fresh::Moved), Some(tasks::SessionEnd::Fresh));
    world.close();
}
