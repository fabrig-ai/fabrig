//! The face's own io gate: it takes no entry of its up stage while the
//! kernel has fewer than two free submissions, ready work included, where
//! `skein_io::stage` resumes with the one `MAX_OUT_RESUME` asks. The order of
//! the stage's entries is `skein_io::stage`'s test, in tests/io.

use skein_io::{self as io, Event, Request, kernel};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall};

use super::{Face, Limits};

#[test]
fn the_face_resumes_ready_work_only_with_two_free_submissions() {
    // One query in a kernel queue of two leaves one free submission.
    const KERNEL_ROOM: u32 = 2;
    const OWNER: Token = Token::new(99);
    let limits = Limits {
        io: io::Limits {
            sockets: 3,
            refusals: 1,
            intake: 32_768,
            receive: 1024,
            output: 32_768,
            sends: 2,
            accepts: 1,
            backlog: 1,
            close_timeout: Duration::from_secs(1),
            names: 0,
            retry: Duration::from_millis(1),
        },
        connections: 2,
        queue: 64,
        plaintext: 32_768,
        ciphertext: 32_768,
        observations: 1,
        observation_bytes: 1,
    };
    let mut face = Face::new(([127, 0, 0, 1], 0).into(), limits).expect("face limits");
    face.submissions = Queue::with_capacity(KERNEL_ROOM);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.io };
    let ready = face.io.adopt_write_pipe(kernel::Fd::new(41), None).expect("pipe slot");
    io::down(&mut face.io, &env, Request::Usage { owner: OWNER }, &mut face.submissions);
    io::down(
        &mut face.io,
        &env,
        Request::Stream { stream: ready, down: Down::Demand { read: Read::Nothing, room: 1 } },
        &mut face.submissions,
    );
    face.io.reclaim();
    face.up(Time::ZERO, Wall::EPOCH);
    let held = face.events.is_empty() && face.io.is_ready();
    let _query = face.submissions.pop().expect("kernel takes the usage query");
    face.up(Time::ZERO, Wall::EPOCH);
    let events: Vec<_> = face.events.iter().collect();
    assert!(held, "ready work waits at one free submission");
    assert_eq!(events, [&Event::Stream { owner: ready, up: Up::Room }]);
}
