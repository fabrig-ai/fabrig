//! The bridge in memory's world tests (playground.md, 8).

mod support;

use skein_lib::{Duration, Time, Wall};
use skein_playground::{memory::Memory, slices::Slices};
use skein_playground_world::{
    scripted,
    site::{Body, Route},
};
use skein_ui_web::{
    bridge::{Input, Speed, Stamp},
    inbox::Inbox,
    log::{self, End},
    platform::{Platform, worker::Heard},
    store::Record,
    transport::{Requests, StreamDuty},
};
use support::{SEEDS, Shell, ShellReport, answer, exchange, fixed, start};

#[test]
fn an_exchange_through_the_bridge_in_memory_is_answered() {
    for seed in SEEDS {
        let mut shell = Shell::new(seed, vec![fixed()]);
        shell.carry(Requests {
            writes: vec![Record { key: b"client".as_slice().into(), value: b"kept".as_slice().into() }],
            exchanges: vec![exchange(1)],
            ..Requests::default()
        });
        shell.until_message();
        assert_eq!(shell.fetches(), vec![answer(1)], "seed {seed}: the server's answer reached the shell");
        assert_eq!(
            shell.store.read_all(&shell.memory),
            vec![Record { key: b"client".as_slice().into(), value: b"kept".as_slice().into() }],
            "seed {seed}: the transport's write is in the page store"
        );
        shell.carry(Requests { removes: vec![b"client".as_slice().into()], ..Requests::default() });
        assert!(shell.store.read_all(&shell.memory).is_empty(), "seed {seed}: removed through the page store");
    }
}

#[test]
fn the_stream_is_read_on_through_the_platform() {
    // The server's six-byte body is delivered to the client in two three-byte pieces.
    const PIECE: u32 = 3;
    for seed in SEEDS {
        let route = Route::new(
            b"/stream",
            200,
            Body::Pieces { pieces: vec![(Duration::ZERO, b"abcdef".as_slice().into())], finish: false },
        );
        let mut shell = Shell::new(seed, vec![route]);
        let world = shell.memory.slices().world();
        let opened = Stamp { step: world.steps(), instant: world.now() };
        let open = StreamDuty::Open { path: b"/stream".as_slice().into(), headers: Vec::new(), bound: PIECE };
        shell.carry(Requests { stream: Some(open.clone()), ..Requests::default() });
        shell.until_message();
        assert_eq!(
            shell.fetches(),
            vec![
                skein_ui_web::fetch::Report::StreamOpened,
                skein_ui_web::fetch::Report::StreamPiece { bytes: b"abc".as_slice().into() },
            ],
            "seed {seed}: only the first piece is demanded"
        );
        let world = shell.memory.slices().world();
        let read_on = Stamp { step: world.steps(), instant: world.now() };
        Platform::read_on(&mut shell.memory);
        shell.until_message();
        assert_eq!(
            shell.fetches(),
            vec![skein_ui_web::fetch::Report::StreamPiece { bytes: b"def".as_slice().into() },],
            "seed {seed}: the trait's read-on asks for the next piece"
        );
        let world = shell.memory.slices().world();
        let closed = Stamp { step: world.steps(), instant: world.now() };
        shell.carry(Requests { stream: Some(StreamDuty::Close), ..Requests::default() });
        shell.until_message();
        assert_eq!(shell.fetches(), vec![skein_ui_web::fetch::Report::StreamEnded], "seed {seed}: closed");
        let read = log::read(&shell.memory.log()).unwrap_or_else(|why| panic!("seed {seed}: log refused: {why:?}"));
        let entries: Vec<_> = read.entries.into_iter().map(|entry| (entry.stamp, entry.input)).collect();
        assert_eq!(
            entries,
            vec![(opened, Input::Stream(open)), (read_on, Input::ReadOn), (closed, Input::Stream(StreamDuty::Close)),],
            "seed {seed}: every stream duty is logged at its entry stamp"
        );
    }
}

#[test]
fn an_abandoned_exchange_reports_nothing_more() {
    for seed in SEEDS {
        let mut shell = Shell::new(
            seed,
            vec![fixed(), Route::new(b"/waiting", 200, Body::Pieces { pieces: Vec::new(), finish: false })],
        );
        let waiting = skein_ui_web::fetch::Exchange { path: b"/waiting".as_slice().into(), ..exchange(1) };
        shell.carry(Requests { exchanges: vec![waiting], ..Requests::default() });
        shell.until_message();
        Platform::abandon(&mut shell.memory, 1);
        shell.until_message();
        shell.carry(Requests { exchanges: vec![exchange(1)], ..Requests::default() });
        shell.until_message();
        assert_eq!(shell.fetches(), vec![answer(1)], "seed {seed}: only the reused id's new exchange answers");
    }
}

#[test]
fn the_clock_is_the_worlds() {
    const LATER: Time = Time::from_nanos(100);
    const DEADLINE: Time = Time::from_nanos(500);
    for seed in SEEDS {
        let mut shell = Shell::new(seed, Vec::new());
        shell.until_message();
        shell.memory.pace(Speed(500), Time::ZERO);
        shell.until_message();
        assert_eq!(shell.memory.clocks(Time::ZERO), (Time::ZERO, start(seed).start), "seed {seed}");
        assert_eq!(shell.memory.timer(DEADLINE, Time::ZERO), Some(Time::from_nanos(1000)), "seed {seed}: half speed");
        let projected = shell.memory.clocks(LATER);
        assert_eq!(projected, (Time::from_nanos(50), Wall::from_nanos(1_000_050)), "seed {seed}: idle projection");
        shell.memory.slice(LATER);
        let world = shell.memory.slices().world();
        assert_eq!(projected, (world.now(), world.wall()), "seed {seed}: the projection is the world's time");
        shell.memory.pace(Speed::PAUSED, LATER);
        shell.memory.slice(LATER);
        assert_eq!(shell.memory.clocks(DEADLINE), projected, "seed {seed}: a paused client waits with the world");
        assert_eq!(shell.memory.timer(DEADLINE, LATER), None, "seed {seed}: no timer while paused");
    }
}

#[test]
fn the_log_kept_natively_reads_back() {
    for seed in SEEDS {
        let mut shell = Shell::new(seed, vec![fixed()]);
        let world = shell.memory.slices().world();
        let first = Stamp { step: world.steps(), instant: world.now() };
        shell.carry(Requests { exchanges: vec![exchange(1)], ..Requests::default() });
        shell.until_message();
        let world = shell.memory.slices().world();
        let second = Stamp { step: world.steps(), instant: world.now() };
        Platform::abandon(&mut shell.memory, 1);
        shell.until_message();
        let read = log::read(&shell.memory.log()).unwrap_or_else(|why| panic!("seed {seed}: log refused: {why:?}"));
        assert_eq!(read.build, None, "seed {seed}: a native log has no build");
        assert_eq!((read.seed, read.start), (seed, start(seed).start), "seed {seed}: visit identity");
        let entries: Vec<_> = read.entries.iter().map(|entry| (entry.stamp, entry.input.clone())).collect();
        assert_eq!(
            entries,
            vec![(first, Input::Exchange((&exchange(1)).into())), (second, Input::Abandon { exchange: 1 })],
            "seed {seed}: every duty in order, at its world's stamp"
        );
        let mut said = log::Saying::new();
        assert_eq!(
            read.entries.first().map(|entry| entry.said),
            Some(said.said()),
            "seed {seed}: nothing before input"
        );
        said.report(&answer(1));
        assert_eq!(
            read.entries.get(1).map(|entry| entry.said),
            Some(said.said()),
            "seed {seed}: answer folded before abandon"
        );
        let world = shell.memory.slices().world();
        assert_eq!(
            read.end,
            End::At { step: world.steps(), instant: world.now(), said: log::Saying::new().said() },
            "seed {seed}: the last clock closes the log"
        );
    }
}

#[test]
fn a_navigation_is_not_followed() {
    for seed in SEEDS {
        let mut shell = Shell::new(seed, vec![fixed()]);
        assert_eq!(shell.memory.shown(), None, "seed {seed}: no navigation yet");
        for (target, expected) in [
            (b"/first".as_slice(), Some(b"/first".as_slice())),
            (b"https://application.example/".as_slice(), Some(b"/first".as_slice())),
            (b"//elsewhere/".as_slice(), Some(b"/first".as_slice())),
            (b"/\\elsewhere".as_slice(), Some(b"/first".as_slice())),
            (b"/last?query=yes".as_slice(), Some(b"/last?query=yes".as_slice())),
        ] {
            shell.carry(Requests { navigate: Some(target.into()), ..Requests::default() });
            assert_eq!(shell.memory.shown(), expected, "seed {seed}: target {target:?}");
        }
        shell.carry(Requests { exchanges: vec![exchange(1)], ..Requests::default() });
        shell.until_message();
        assert_eq!(shell.fetches(), vec![answer(1)], "seed {seed}: the same world still answers");
    }
}

#[test]
fn the_worlds_lines_and_stop_reach_the_shell_in_order() {
    for seed in SEEDS {
        let (world, reach) = scripted::world(seed, &[], true);
        let inbox = Inbox::new();
        let mut memory = Memory::new(
            Slices::new(world, reach, support::BUDGET, Time::ZERO),
            start(seed),
            support::LOG_BOUND,
            inbox.clone(),
            ShellReport::Fetch,
            ShellReport::World,
        );
        Platform::abandon(&mut memory, 1);
        memory.slice(Time::ZERO);
        let reports: Vec<_> = std::iter::from_fn(|| inbox.next()).collect();
        let printed = |stream, line: &[u8]| {
            ShellReport::World(Heard::Printed(skein_ui_web::bridge::Printed {
                process: b"script".as_slice().into(),
                stream,
                line: line.into(),
            }))
        };
        assert_eq!(
            reports,
            vec![
                printed(skein_ui_web::bridge::Stream::Output, b"one"),
                printed(skein_ui_web::bridge::Stream::Output, b"two"),
                printed(skein_ui_web::bridge::Stream::Error, b"error"),
                ShellReport::World(Heard::Stopped),
                ShellReport::World(Heard::Clock),
            ],
            "seed {seed}: stamps stay internal, clock last"
        );
    }
}
