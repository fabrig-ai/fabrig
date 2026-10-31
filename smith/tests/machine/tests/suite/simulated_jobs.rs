//! Successive jobs on simulated time and pipes (smith/docs/design/protocol/agent.md, 2).

use super::jobs::{done, far, read, spawn};
use skein_lib::{Duration, Time, Token};
use smith_domain::tools;
use smith_machine_world::World;
use smith_protocol_machine::{FromDomain, ToDomain};

// The real jobs fixtures retain eight bytes at each end, or four for overflow.
const KEEP_BYTES: u32 = 8;
const OVERFLOW_KEEP_BYTES: u32 = 4;
const OWNER: Token = Token::new(10);
const READ_WAIT: Duration = Duration::from_secs(1);

fn run(seed: u64, requests: &[FromDomain], delays: &[Duration]) -> (Vec<ToDomain>, Time) {
    let mut first = World::new(seed, &[]);
    first.check_memory();
    let replies = first.simulated_jobs(requests, delays);
    let mut replay = World::new(seed, &[]);
    replay.check_memory();
    let repeated = replay.simulated_jobs(requests, delays);
    assert_eq!(replies, repeated, "seed {seed} replays every terminal");
    assert_eq!(first.trace(), replay.trace(), "seed {seed} replays the complete kernel trace");
    assert_eq!(first.now(), replay.now(), "seed {seed} replays settlement time");
    (replies, first.now())
}

fn job(end: tools::JobEnd) -> tools::Done {
    tools::Done::Job { head: Box::default(), tail: Box::default(), dropped: 0, end }
}
#[test]
fn a_yielded_command_is_read_to_its_end() {
    let root = World::ROOT;
    let (replies, _) = run(
        71,
        &[
            spawn(root, b"printf one; sleep 0.08; printf two", KEEP_BYTES, KEEP_BYTES, far()),
            read(KEEP_BYTES, KEEP_BYTES, READ_WAIT),
        ],
        &[],
    );
    assert_eq!(
        done(&replies[0]),
        tools::Done::Held { head: b"one".as_slice().into(), tail: Box::default(), dropped: 0 }
    );
    assert_eq!(
        done(&replies[1]),
        tools::Done::Job {
            head: b"two".as_slice().into(),
            tail: Box::default(),
            dropped: 0,
            end: tools::JobEnd::Ended(tools::Exit::Code { code: 0 }),
        }
    );
}
#[test]
fn held_output_is_partitioned_and_counted_only_once() {
    // Three alphabets contain 78 bytes: four head, four tail, 70 dropped.
    const DROPPED_BYTES: u64 = 70;
    let root = World::ROOT;
    let (replies, _) = run(
        72,
        &[
            spawn(
                root,
                b"printf abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz; sleep 0.08; printf end",
                OVERFLOW_KEEP_BYTES,
                OVERFLOW_KEEP_BYTES,
                far(),
            ),
            read(OVERFLOW_KEEP_BYTES, OVERFLOW_KEEP_BYTES, Duration::ZERO),
            read(OVERFLOW_KEEP_BYTES, OVERFLOW_KEEP_BYTES, READ_WAIT),
        ],
        &[],
    );
    assert_eq!(
        done(&replies[0]),
        tools::Done::Held { head: b"abcd".as_slice().into(), tail: b"wxyz".as_slice().into(), dropped: DROPPED_BYTES }
    );
    assert_eq!(done(&replies[1]), job(tools::JobEnd::Running));
    assert_eq!(
        done(&replies[2]),
        tools::Done::Job {
            head: b"end".as_slice().into(),
            tail: Box::default(),
            dropped: 0,
            end: tools::JobEnd::Ended(tools::Exit::Code { code: 0 }),
        }
    );
}
#[test]
fn a_stopped_job_settles_and_answers_stopped() {
    let root = World::ROOT;
    let (replies, _) = run(
        73,
        &[
            spawn(root, b"sleep 5", KEEP_BYTES, KEEP_BYTES, far()),
            FromDomain::Job { owner: OWNER, act: tools::JobAct::Stop { head: KEEP_BYTES, tail: KEEP_BYTES } },
        ],
        &[],
    );
    assert_eq!(done(&replies[1]), job(tools::JobEnd::Stopped));
}
#[test]
fn the_original_deadline_ends_a_held_job() {
    // The original fixture yields at 30 ms and expires at 60 ms.
    const DEADLINE: Time = Time::from_nanos(60_000_000);
    let root = World::ROOT;
    let (replies, now) = run(
        74,
        &[spawn(root, b"sleep 5", KEEP_BYTES, KEEP_BYTES, DEADLINE), read(KEEP_BYTES, KEEP_BYTES, READ_WAIT)],
        &[],
    );
    assert_eq!(done(&replies[1]), job(tools::JobEnd::Ended(tools::Exit::TimedOut)));
    assert!(now >= DEADLINE, "the run reaches its original deadline");
}

fn cancel(seed: u64, command: &[u8], delay: Duration) -> tools::Done {
    let root = World::new(seed, &[]).root();
    let (replies, _) = run(
        seed,
        &[spawn(root, command, KEEP_BYTES, KEEP_BYTES, far()), FromDomain::Cancel { owner: OWNER }],
        &[delay],
    );
    done(&replies[1])
}
#[test]
fn cancelling_a_running_held_job_answers_cancelled() {
    assert_eq!(cancel(76, b"sleep 5", Duration::ZERO), job(tools::JobEnd::Cancelled));
}
#[test]
fn cancelling_a_finished_job_preserves_its_successful_exit() {
    // The counterpart cancels 80 ms after the 30 ms yield of a 60 ms command.
    const CANCEL_DELAY: Duration = Duration::from_millis(80);
    assert_eq!(cancel(77, b"sleep 0.06", CANCEL_DELAY), job(tools::JobEnd::Ended(tools::Exit::Code { code: 0 })));
}
#[test]
fn a_hastened_read_finishes_before_its_virtual_wait() {
    // The real counterpart must settle before 500 ms of its one-second wait.
    const BEFORE_END: Time = Time::from_nanos(500_000_000);
    let root = World::ROOT;
    let (replies, now) = run(
        78,
        &[
            spawn(root, b"sleep 1", KEEP_BYTES, KEEP_BYTES, far()),
            read(KEEP_BYTES, KEEP_BYTES, READ_WAIT),
            FromDomain::Job { owner: OWNER, act: tools::JobAct::Hasten },
        ],
        &[],
    );
    assert_eq!(done(&replies[1]), job(tools::JobEnd::Running));
    assert!(now < BEFORE_END, "the read ends before its wait and command");
}
