//! Standing sessions through the real preparation, claim and store barrier (standing.md, 5).
use jig_core_world::{fixtures, fixtures::live_row, stories::per_wake as story, world::{Setup, World}};
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
    charter.parts = [(tasks::Reasons::FIRST, b"F".as_slice()), (tasks::Reasons::WORDS.union(tasks::Reasons::RESULT), b"WR")]
        .into_iter().map(|(on, instructions)| core::CharterPart { on, instructions: instructions.into() }).collect();
    setup
}

fn start(standing: bool) -> (World, (u64, u64)) {
    let mut world = World::new(setup());
    let caller = fixtures::run(&world);
    let mut member = fixtures::member(&world, tasks::Executor::Agent { charter: 2 }, b"keep watch");
    member.flags.ended_by = if standing { tasks::EndedBy::Standing } else { tasks::EndedBy::Requester };
    member.authority.budget.spend = 60;
    let task = fixtures::delegate(&mut world, caller, 1, member);
    let run = (task, live_row(&world, task).attempt);
    story::park(&mut world, caller, None);
    (world, run)
}

fn opened() -> (World, (u64, u64), (u64, u64)) {
    let (mut world, run) = start(true);
    assert_eq!(world.assigned.as_ref().expect("standing assignment").run.policy.instructions.as_ref(), FIRST);
    assert_eq!(live_row(&world, run.0).session.expect("claimed session").first, run.1);
    let child = story::child(&mut world, run);
    story::turn(&mut world, run, None);
    (world, run, child)
}

fn say(world: &mut World, task: u64, key: u8) {
    fixtures::ask(world, key, people::Ask::Say { project: 1, task, words: b"check now".as_slice().into() });
}

fn assignment(world: &World, task: u64, instructions: &[u8], resumed: bool) -> (u64, u64) {
    let assigned = world.assigned.as_ref().expect("task assigned");
    assert_eq!(assigned.task, task);
    assert_eq!(assigned.run.policy.instructions.as_ref(), instructions);
    assert_eq!(!assigned.transcript.is_empty(), resumed);
    assert_eq!(assigned.run.policy.resume, resumed);
    if !resumed { assert_eq!(assigned.conversation, assigned.attempt); }
    (task, assigned.attempt)
}

fn ends(world: &World, task: u64) -> Vec<(u64, tasks::SessionEnd)> {
    world.store.rows.values().filter_map(|row| match row {
        root::Record::Core(core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == task => match line.line {
            tasks::Line::SessionEnded { attempt, end } => Some((attempt, end)),
            _ => None,
        },
        _ => None,
    }).collect()
}

fn consume_and_park(world: &mut World, run: (u64, u64)) {
    let read = live_row(world, run.0).inbox.last().map(|word| word.number);
    story::turn(world, run, read);
    story::park(world, run, read);
}

#[test]
fn a_warm_wake_keeps_the_transcript_instructions_and_session() {
    let (mut world, run, _) = opened();
    let session = live_row(&world, run.0).session;
    story::park(&mut world, run, None);
    assert!(!live_row(&world, run.0).activation_open);
    world.advance(Duration::from_secs(269));
    say(&mut world, run.0, 80);
    assignment(&world, run.0, FIRST, true);
    assert_eq!(live_row(&world, run.0).session, session);
    assert!(ends(&world, run.0).is_empty());
    assert!(live_row(&world, run.0).activation_open);
    world.close();
}

#[test]
fn a_quiet_wake_commits_the_end_and_restarts_the_session_and_tries() {
    let (mut world, run, _) = opened();
    world.send(root::Event::Answer { channel: Token::new(7), task: run.0, attempt: run.1, cumulative: 0,
        end: tasks::End::Failed(tasks::Class::Transient), read: None, refused: Box::new([]) });
    world.retry_due();
    let retry = (run.0, live_row(&world, run.0).attempt);
    assert_eq!(live_row(&world, run.0).tries.transient, 1);
    consume_and_park(&mut world, retry);
    assert_eq!(live_row(&world, run.0).tries.transient, 1, "a standing park keeps the session's tries");
    world.advance(Duration::from_secs(271));
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, LATER, false);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Quiet)]);
    assert_eq!(live_row(&world, run.0).session, Some(tasks::Session {
        first: next.1, reasons: tasks::Reasons::WORDS, last: world.wall_time()
    }));
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
        reply_to: ReplyTo::new(Token::new(985)), task: run.0,
        subscription: tasks::Subscription { number: 985, kind: tasks::SubscriptionKind::Task {
            target: 1, held: true, result: true
        } }
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
    world.advance(Duration::from_secs(271));
    world.send(root::Event::Answer { channel: Token::new(7), task: run.0, attempt: run.1, cumulative: 0,
        end: tasks::End::Failed(tasks::Class::Transient), read: None, refused: Box::new([]) });
    world.retry_due();
    let retry = assignment(&world, run.0, FIRST, true);
    assert_eq!(live_row(&world, run.0).session.expect("same session").first, run.1);
    assert_eq!(live_row(&world, run.0).tries.transient, 1);
    assert!(ends(&world, run.0).is_empty());
    world.send(root::Event::Answer { channel: Token::new(7), task: retry.0, attempt: retry.1, cumulative: 0,
        end: tasks::End::Recheck, read: None, refused: Box::new([]) });
    assignment(&world, run.0, FIRST, true);
    assert_eq!(live_row(&world, run.0).session.expect("interrupted session").first, run.1);
    assert_eq!(live_row(&world, run.0).tries.transient, 1);
    assert!(ends(&world, run.0).is_empty());
    world.close();
}

fn restart(world: &mut World, child: (u64, u64)) {
    world.restart_with(setup().configuration.expect("same configuration"));
    world.send(root::Event::Core(core::Event::Fleet(jig_domain_fleet::Event::Hello {
        worker: 0, channel: Token::new(7), hello: jig_domain_link::Hello {
            jobs: Box::new([]), bounds: jig_test_profile::fixtures::bounds(), stop_bound: Duration::from_millis(100),
            slots: 2, workstreams: Box::new([]), hosting: Box::new([jig_domain_link::Hosting {
                run: Token::new(child.0), attempt: Token::new(child.1), phase: jig_domain_link::Phase::Active
            }])
        }
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
    world.advance(Duration::from_secs(1));
    say(&mut world, run.0, 80);
    let next = assignment(&world, run.0, FIRST, true);
    consume_and_park(&mut world, next);
    assert_eq!(live_row(&world, run.0).session.expect("same session").first, run.1);
    assert!(ends(&world, run.0).is_empty());
    world.close();
}

#[test]
fn an_instruction_amendment_ends_rules_only_when_it_commits() {
    let (mut world, run, _) = opened();
    story::park(&mut world, run, None);
    let mut amendment = jig_test_profile::fixtures::person_unchanged(b"new instructions");
    amendment.instructions = Some(b"own".as_slice().into());
    world.store.fault(jig_core_world::Fault::Hold { commits: 1 });
    fixtures::ask(&mut world, 80, people::Ask::Amend { project: 1, task: run.0,
        revision: 0, release: None, amendment });
    assert!(ends(&world, run.0).is_empty(), "no end is published ahead of the commit");
    world.release_commits();
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Rules)]);
    assignment(&world, run.0, b"own", false);
    world.close();
}

#[test]
fn moving_to_new_settings_ends_rules_and_the_next_run_is_fresh() {
    let (mut world, run, _) = opened();
    story::park(&mut world, run, None);
    let charter = setup().configuration.unwrap().core.defaults.charters[1].clone();
    world.send(root::Event::Core(core::Event::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(986)), sign_in: world.sign_in.unwrap(), key: [81; 16], digest: [81; 32],
        change: Box::new(core::settings::Change { against: 1,
            items: Box::new([core::settings::Item { setting: core::settings::Setting::Charter(2),
                edit: core::settings::Edit::Set(core::settings::Value::Charter(Box::new(charter))) }]),
            running: core::settings::Running::Stays
        })
    }));
    assert_eq!(world.domain.settings().newest(), 2);
    fixtures::ask(&mut world, 82, people::Ask::MoveToNewest { revision: 1 });
    assert_eq!(live_row(&world, run.0).settings.task, 2);
    assert_eq!(ends(&world, run.0), [(run.1, tasks::SessionEnd::Rules)]);
    assert!(live_row(&world, run.0).session.is_none());
    say(&mut world, run.0, 83);
    assignment(&world, run.0, LATER, false);
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
