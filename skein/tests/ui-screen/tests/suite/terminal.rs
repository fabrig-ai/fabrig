use skein_io::kernel::{Complete, Done, Fd, Op, Signalled, Submit};
use skein_lib::{Duration, List, Queue, Time, Token};
use skein_sim::{Config, Pid, Sim};
use skein_ui_screen::{Cell, Focus, Frame, Paint, Position, Rgb, Size};
use skein_ui_screen_terminal::{
    ASK_BACKGROUND, Background, Command, Depth, ENTER, Key, LEAVE, Settings, Terminal, command, draw, draw_bound,
};

const SIZE: Size = Size { rows: 1, columns: 1 };
const INTERVAL: Duration = Duration::from_secs(1);
const SECOND: Time = Time::from_nanos(1_000_000_000);
const THIRD: Time = Time::from_nanos(2_000_000_000);
// The terminal's output room (skein/docs/design/ui/screen.md, 8).
const OUTPUT: u32 = 65_536;
const READ: u32 = 4096;
// Alternating RGB paints make 4096 cells exceed the 64 KiB write room.
const LARGE: Size = Size { rows: 32, columns: 128 };
const SIGNAL: Token = Token::new(1_u64 << 61);
const SIGNAL_CLOSE: Token = Token::new((1_u64 << 61) | 1);
const WHOLE_A: &[u8] = b"\x1b[2J\x1b[1;1H\x1b[0mA\x1b[?25l";
const DIFF_C: &[u8] = b"\x1b[1;1H\x1b[0mC\x1b[?25l";

struct Pipes {
    sim: Sim,
    pid: Pid,
    input: Fd,
    output: Fd,
    signal: Fd,
    terminal: Terminal,
    now: Time,
    windows: usize,
    writes: Vec<usize>,
    reads: usize,
    quiet_leave: bool,
}
fn settings(most: Size) -> Settings {
    Settings { interval: INTERVAL, depth: Depth::True, most, paste: READ, background: None }
}
fn start(buffer: u32, most: Size) -> Pipes {
    start_with_window(buffer, most, true)
}
fn start_with_window(buffer: u32, most: Size, window: bool) -> Pipes {
    start_settings(buffer, settings(most), window, Time::ZERO)
}
fn start_settings(buffer: u32, settings: Settings, window: bool, first: Time) -> Pipes {
    let mut config = Config::calm();
    config.buffer = buffer;
    let mut sim = Sim::new(3, config);
    let pid = sim.spawn_process();
    let input = sim.open_inherited_read(pid);
    let output = sim.open_inherited_write(pid);
    let signal = sim.open_signal_source(pid);
    if window {
        sim.set_window(pid, input, skein_io::kernel::Size { rows: SIZE.rows, columns: SIZE.columns });
    }
    let mut terminal = Terminal::new(settings, SIZE).expect("settings fit");
    terminal.adopt(input, output).expect("terminal adopts once");
    let mut p = Pipes {
        sim,
        pid,
        input,
        output,
        signal,
        terminal,
        now: Time::ZERO,
        windows: 0,
        writes: Vec::new(),
        reads: 0,
        quiet_leave: false,
    };
    p.drive(first);
    p
}
impl Pipes {
    fn submit(&mut self) {
        for s in self.terminal.submissions().iter() {
            assert!(Terminal::owns(s.op), "terminal submissions carry its namespace");
            match &s.kind {
                Op::PipeRead { .. } => self.reads += 1,
                Op::Window { fd } => {
                    assert_eq!(*fd, self.input);
                    self.windows += 1;
                }
                Op::PipeWrite { bytes, .. } => {
                    if self.quiet_leave && bytes.as_ref() == LEAVE {
                        assert_eq!(self.sim.in_flight(self.pid), 0, "LEAVE follows the settled input read");
                    }
                    assert!(bytes.len() <= usize::try_from(OUTPUT).expect("room fits"));
                    self.writes.push(bytes.len());
                }
                Op::Cancel { target } => assert!(Terminal::owns(*target), "cancel target carries namespace"),
                _ => {}
            }
        }
        self.sim.submit(self.pid, self.terminal.submissions());
    }
    fn drive(&mut self, now: Time) {
        self.now = now;
        self.sim.advance_to(now);
        for _ in 0..20 {
            self.sim.reap(self.pid, self.terminal.completions());
            self.terminal.iterate(now);
            self.submit();
        }
    }
    fn feed(&mut self, bytes: &[u8]) {
        assert_eq!(self.sim.peer_feed(self.pid, self.input, bytes), bytes.len());
        self.drive(self.now);
    }
    fn drain(&mut self) -> Vec<u8> {
        self.sim.peer_drain(self.pid, self.output, usize::try_from(OUTPUT).expect("room fits"))
    }
    fn collect(&mut self) -> Vec<u8> {
        let mut bytes = Vec::new();
        // Enough turns for the one-byte-buffer close story, or a large frame's pieces.
        for _ in 0..256 {
            bytes.extend(self.drain());
            self.drive(self.now);
        }
        bytes
    }
    fn resize(&mut self, size: Size) {
        self.sim.set_window(self.pid, self.input, skein_io::kernel::Size { rows: size.rows, columns: size.columns });
        let mut submissions = Queue::with_capacity(1);
        submissions.push(Submit { op: SIGNAL, kind: Op::ReadSignal { fd: self.signal } });
        self.sim.submit(self.pid, &mut submissions);
        self.sim.deliver_window_changed(self.pid, self.signal);
        let mut completions = Queue::with_capacity(16);
        self.sim.reap(self.pid, &mut completions);
        for _ in 0..completions.len() {
            let complete = completions.pop().expect("counted completion");
            if complete.op == SIGNAL {
                assert_eq!(complete.result, Ok(Done::Signalled(Signalled::WindowChanged)));
                self.terminal.resized();
            } else {
                self.terminal.completions().push(complete);
            }
        }
    }
    fn finish(&mut self) {
        self.terminal.close();
        self.collect();
        assert!(self.terminal.closed());
        let mut submissions = Queue::with_capacity(1);
        submissions.push(Submit { op: SIGNAL_CLOSE, kind: Op::Close { fd: self.signal } });
        self.sim.submit(self.pid, &mut submissions);
        let mut completions: Queue<Complete> = Queue::with_capacity(1);
        self.sim.reap(self.pid, &mut completions);
        let complete = completions.pop().expect("signal source closed");
        assert_eq!(complete.op, SIGNAL_CLOSE);
        assert_eq!(complete.result, Ok(Done::Nothing));
        self.sim.assert_no_open_fds(self.pid);
    }
}
fn frame(most: Size, size: Size, glyph: char) -> Frame {
    let mut frame = Frame::new(most);
    frame.clear(size, Cell::BLANK);
    assert!(frame.put(Position { row: 0, column: 0 }, Cell { glyph, paint: Paint::NONE }));
    frame
}

#[test]
fn enter_is_written_before_the_first_frame() {
    let mut p = start(OUTPUT, SIZE);
    p.terminal.draw(&frame(SIZE, SIZE, 'A'));
    p.drive(Time::ZERO);
    assert_eq!(p.drain(), [ENTER, WHOLE_A].concat());
    assert!(!Terminal::owns(SIGNAL));
    p.finish();
}

#[test]
fn background_is_asked_only_with_a_wait_and_before_the_first_frame() {
    assert_eq!(ASK_BACKGROUND, b"\x1b]11;?\x1b\\");
    for wait in [None, Some(INTERVAL), Some(Duration::from_nanos(0))] {
        let settings = Settings { background: wait, ..settings(SIZE) };
        if wait.is_none() {
            assert_eq!(Terminal::new(settings, SIZE).expect("valid").background(), Background::Unanswered);
        }
        let mut p = start_settings(OUTPUT, settings, true, SECOND);
        let state = if wait == Some(INTERVAL) { Background::Waiting } else { Background::Unanswered };
        assert_eq!(p.terminal.background(), state, "{wait:?}");
        p.terminal.draw(&frame(SIZE, SIZE, 'A'));
        p.drive(SECOND);
        let query = if wait.is_some() { ASK_BACKGROUND } else { b"" };
        assert_eq!(p.drain(), [ENTER, query, WHOLE_A].concat(), "{wait:?}");
        p.finish();
    }
}

#[test]
fn background_answer_and_keys_are_read_together() {
    for (answer, expected) in [
        (b"\x1b]11;rgb:12/34/56\x07".as_slice(), Background::Answered(Rgb { red: 0x12, green: 0x34, blue: 0x56 })),
        (b"\x1b]11;unreadable\x1b\\", Background::Unanswered),
    ] {
        let mut p = start_settings(OUTPUT, Settings { background: Some(INTERVAL), ..settings(SIZE) }, true, SECOND);
        assert_eq!(p.terminal.background(), Background::Waiting);
        // Input already completed at the deadline wins over expiry.
        assert_eq!(p.sim.peer_feed(p.pid, p.input, &[b"a", answer, b"z\x1b"].concat()), answer.len() + 3);
        p.drive(THIRD);
        assert_eq!(p.terminal.background(), expected);
        assert_eq!(p.terminal.next_deadline(), None);
        assert_eq!(p.terminal.keys().pop(), Some(Key::Char('a')));
        assert_eq!(p.terminal.keys().pop(), Some(Key::Char('z')));
        assert_eq!(p.terminal.keys().pop(), Some(Key::Escape), "the answer ends the wait");
        assert_eq!(p.terminal.keys().pop(), None, "answers never reach keys");
        p.finish();
    }
}

#[test]
fn background_wait_starts_at_first_iteration_and_wakes_when_due() {
    let mut p = start_settings(OUTPUT, Settings { background: Some(INTERVAL), ..settings(SIZE) }, true, SECOND);
    assert_eq!(p.terminal.background(), Background::Waiting);
    assert_eq!(p.terminal.next_deadline(), Some(THIRD));
    assert!(!p.terminal.work_pending(SECOND));
    assert!(p.terminal.work_pending(THIRD));
    p.drive(THIRD);
    assert_eq!(p.terminal.background(), Background::Unanswered);
    assert_eq!(p.terminal.next_deadline(), None);
    assert!(!p.terminal.work_pending(THIRD));
    p.feed(b"\x1b]11;rgb:ff/aa/00\x1b\\q");
    assert_eq!(p.terminal.background(), Background::Unanswered, "a late answer stays unanswered");
    assert_eq!(p.terminal.keys().pop(), Some(Key::Char('q')));
    assert_eq!(p.terminal.keys().pop(), None, "a late answer is never a key");
    p.finish();
}

#[test]
fn escape_reaches_the_host_after_an_unanswered_background_wait() {
    for (before, held) in [(b"".as_slice(), None), (b"\x1b".as_slice(), Some(Key::Escape))] {
        let mut p = start_settings(OUTPUT, Settings { background: Some(INTERVAL), ..settings(SIZE) }, true, SECOND);
        p.feed(before);
        assert_eq!(p.terminal.keys().pop(), None, "a lone ESC waits for the background");
        p.drive(THIRD);
        assert_eq!(p.terminal.keys().pop(), held, "the wait's end releases any held Escape");
        p.feed(b"\x1b");
        assert_eq!(p.terminal.keys().pop(), Some(Key::Escape));
        assert_eq!(p.terminal.keys().pop(), None);
        p.finish();
    }
}

#[test]
fn input_end_while_waiting_leaves_background_unanswered() {
    let mut p = start_settings(OUTPUT, Settings { background: Some(INTERVAL), ..settings(SIZE) }, true, SECOND);
    assert_eq!(p.terminal.background(), Background::Waiting);
    p.feed(b"\x1b");
    p.sim.peer_close(p.pid, p.input);
    p.drive(SECOND);
    assert!(p.terminal.ended());
    assert_eq!(p.terminal.keys().pop(), Some(Key::Escape), "input ending releases the held Escape");
    assert_eq!(p.terminal.background(), Background::Unanswered);
    assert_eq!(p.terminal.next_deadline(), None);
    p.finish();
}

#[test]
fn split_keys_and_paste_wait_for_the_host() {
    let mut p = start(OUTPUT, SIZE);
    p.feed(b"a\xc3");
    p.feed(b"\xa9\x1b[");
    assert_eq!(p.terminal.keys().len(), 1, "untaken keys prevent decoding the next read");
    assert_eq!(p.terminal.keys().pop(), Some(Key::Char('a')));
    assert!(p.terminal.work_pending(p.now), "taking the last key wakes the reader");
    p.drive(p.now);
    assert_eq!(p.terminal.keys().pop(), Some(Key::Char('é')));
    p.feed(b"A\x1b[200~pa");
    assert_eq!(p.terminal.keys().pop(), Some(Key::Up));
    p.feed(b"ste\x1b[20");
    p.feed(b"1~z");
    assert_eq!(p.terminal.keys().pop(), Some(Key::Paste(b"paste".as_slice().into())));
    p.feed(b"b");
    assert_eq!(p.terminal.keys().pop(), Some(Key::Char('z')));
    p.drive(p.now);
    assert_eq!(p.terminal.keys().pop(), Some(Key::Char('b')));
    p.finish();
}

#[test]
fn frames_within_the_interval_keep_only_the_newest() {
    let mut p = start(OUTPUT, SIZE);
    p.drain();
    p.terminal.draw(&frame(SIZE, SIZE, 'A'));
    p.drive(Time::ZERO);
    assert_eq!(p.drain(), WHOLE_A);
    p.terminal.draw(&frame(SIZE, SIZE, 'B'));
    p.drive(Time::from_nanos(100));
    p.terminal.draw(&frame(SIZE, SIZE, 'C'));
    p.drive(Time::from_nanos(200));
    assert_eq!(p.drain().len(), 0);
    assert_eq!(p.terminal.next_deadline(), Some(SECOND));
    assert!(!p.terminal.work_pending(p.now));
    assert!(p.terminal.work_pending(SECOND));
    p.drive(SECOND);
    assert_eq!(p.drain(), DIFF_C);
    p.drive(THIRD);
    assert_eq!(p.drain().len(), 0);
    assert_eq!(p.terminal.next_deadline(), None);
    p.finish();
}

#[test]
fn a_blocked_frame_finishes_before_the_newest_waiting_frame() {
    let mut p = start(1, SIZE);
    assert_eq!(p.collect(), ENTER);
    p.terminal.draw(&frame(SIZE, SIZE, 'A'));
    p.drive(Time::ZERO);
    p.terminal.draw(&frame(SIZE, SIZE, 'B'));
    p.drive(SECOND);
    p.terminal.draw(&frame(SIZE, SIZE, 'C'));
    p.drive(THIRD);
    assert_eq!(p.terminal.next_deadline(), None, "blocked output has no drawing deadline");
    assert_eq!(p.collect(), [WHOLE_A, DIFF_C].concat());
    p.finish();
}

fn large_frame() -> Frame {
    let mut large = frame(LARGE, LARGE, 'A');
    for row in 0..LARGE.rows {
        for column in 0..LARGE.columns {
            let paint = Paint {
                fore: Some(Rgb { red: u8::try_from(column % 2).expect("parity fits"), green: 0, blue: 0 }),
                ..Paint::NONE
            };
            assert!(large.put(Position { row, column }, Cell { glyph: 'x', paint }));
        }
    }
    large
}

#[test]
fn a_large_frame_is_written_in_pieces_before_the_next() {
    let large = large_frame();
    let next = frame(LARGE, SIZE, 'C');
    let mut expected = List::with_capacity(draw_bound(LARGE).expect("bound fits"));
    draw(None, &large, Depth::True, &mut expected);
    assert!(expected.len() > OUTPUT, "fixture spans writes");
    draw(Some(&large), &next, Depth::True, &mut expected);
    let mut p = start(OUTPUT, LARGE);
    p.drain();
    p.writes.clear();
    p.terminal.draw(&large);
    p.drive(Time::ZERO);
    p.terminal.draw(&next);
    p.drive(SECOND);
    assert_eq!(p.collect(), expected.as_slice());
    assert!(p.writes.iter().filter(|len| **len == usize::try_from(OUTPUT).expect("room fits")).count() >= 1);
    p.finish();
}

#[test]
fn resizing_queries_once_then_again_and_redraws_whole() {
    const WIDER: Size = Size { rows: 1, columns: 2 };
    let mut p = start(OUTPUT, WIDER);
    p.drain();
    let a = frame(WIDER, SIZE, 'A');
    p.terminal.draw(&a);
    p.drive(Time::ZERO);
    p.drain();
    p.resize(WIDER);
    p.terminal.iterate(p.now);
    p.terminal.resized();
    p.terminal.iterate(p.now);
    p.submit();
    p.drive(p.now);
    assert_eq!(p.windows, 2, "a resize during the request is asked again after its answer");
    assert_eq!(p.terminal.size(), WIDER);
    p.terminal.draw(&a);
    p.drive(SECOND);
    assert_eq!(p.drain(), WHOLE_A, "even the unchanged frame is drawn whole after resize");
    // A pipe without a simulated window answers Gone and keeps the last size.
    let mut gone = start_with_window(OUTPUT, SIZE, false);
    gone.terminal.resized();
    gone.drive(Time::ZERO);
    assert_eq!(gone.terminal.size(), SIZE);
    gone.finish();
    p.finish();
}

#[derive(Debug)]
enum Exit {
    Close,
    End,
    Frame,
    ControlC,
}
#[test]
fn every_close_path_finishes_the_frame_and_leaves_last() {
    let large = large_frame();
    let mut large_bytes = List::with_capacity(draw_bound(LARGE).expect("bound fits"));
    draw(None, &large, Depth::True, &mut large_bytes);
    for (exit, prefix) in [
        (Exit::Close, ENTER),
        (Exit::End, ENTER),
        (Exit::Frame, [ENTER, large_bytes.as_slice()].concat().as_slice()),
        (Exit::ControlC, ENTER),
    ] {
        // Half a write's room holds the first piece in flight with another piece waiting.
        let mut p = start(OUTPUT / 2, LARGE);
        let mut bytes = Vec::new();
        match exit {
            Exit::Close => {}
            Exit::End => {
                p.sim.peer_close(p.pid, p.input);
                p.drive(p.now);
                assert!(p.terminal.ended());
                assert!(!p.terminal.closed(), "input end leaves close to the host");
            }
            Exit::Frame => {
                bytes.extend(p.collect());
                p.terminal.draw(&large);
                p.drive(p.now);
            }
            Exit::ControlC => {
                p.feed(b"\x03");
                let key = p.terminal.keys().pop().expect("Ctrl+C read");
                let limits = skein_ui::Limits { nodes: 1, depth: 1, ..skein_ui::Limits::EMPTY };
                let mut builder = skein_ui::Builder::new(&limits);
                builder.open(skein_ui::Role::Page);
                builder.close();
                let tree = builder.finish();
                assert_eq!(command(&key, &tree, &mut Focus::default(), &limits), Command::Close);
            }
        }
        p.terminal.close();
        p.terminal.draw(&frame(SIZE, SIZE, 'B'));
        bytes.extend(p.collect());
        assert_eq!(bytes, [prefix, LEAVE].concat(), "{exit:?}");
        assert!(p.terminal.closed(), "{exit:?}");
        p.terminal.draw(&frame(SIZE, SIZE, 'C'));
        p.terminal.resized();
        p.terminal.close();
        p.drive(THIRD);
        assert!(p.drain().is_empty(), "nothing follows LEAVE: {exit:?}");
        assert!(!p.terminal.work_pending(THIRD));
        assert_eq!(p.terminal.next_deadline(), None);
        p.finish();
    }
}

#[test]
fn operations_and_construction_refuse_invalid_settings() {
    let valid = settings(SIZE);
    let overflow = Size { rows: u16::MAX, columns: u16::MAX };
    for (settings, size, count, accepted) in [
        (valid, SIZE, Some(10), true),
        (Settings { background: Some(Duration::from_nanos(0)), ..valid }, SIZE, Some(10), true),
        (Settings { most: overflow, ..valid }, SIZE, None, false),
        (Settings { paste: 0, ..valid }, SIZE, None, false),
        (Settings { interval: Duration::from_nanos(0), ..valid }, SIZE, None, false),
        (valid, Size { rows: 0, columns: 1 }, Some(10), false),
        (valid, Size { rows: 1, columns: 0 }, Some(10), false),
    ] {
        assert_eq!(Terminal::operations(&settings), count, "{settings:?}");
        assert_eq!(Terminal::new(settings, size).is_some(), accepted, "{settings:?} {size:?}");
    }
}

#[test]
fn lending_finishes_the_frame_withdraws_read_and_resumes_whole() {
    let large = large_frame();
    let mut expected = List::with_capacity(draw_bound(LARGE).expect("bound fits"));
    draw(None, &large, Depth::True, &mut expected);
    let mut p =
        start_settings(OUTPUT / 2, Settings { background: Some(INTERVAL), ..settings(LARGE) }, true, Time::ZERO);
    assert_eq!(p.collect(), [ENTER, ASK_BACKGROUND].concat());
    p.feed(b"\x1b]11;rgb:12/34/56\x07");
    let background = p.terminal.background();
    p.terminal.draw(&large);
    p.drive(Time::ZERO);
    assert!(p.sim.in_flight(p.pid) > 0, "the frame is still being written");
    p.quiet_leave = true;
    p.terminal.lend();
    assert!(!p.terminal.lent(), "the frame and LEAVE have not finished");
    p.terminal.draw(&frame(LARGE, SIZE, 'B'));
    assert_eq!(p.collect(), [expected.as_slice(), LEAVE].concat());
    assert!(p.terminal.lent());
    assert_eq!(p.sim.in_flight(p.pid), 0, "no input read is outstanding");
    p.quiet_leave = false;
    let reads = p.reads;
    let writes = p.writes.len();
    p.terminal.lend();
    p.terminal.resized();
    p.terminal.draw(&frame(LARGE, SIZE, 'B'));
    let mut newest = large_frame();
    assert!(newest.put(Position { row: 0, column: 0 }, Cell { glyph: 'C', paint: Paint::NONE }));
    p.terminal.draw(&newest);
    p.drive(SECOND);
    assert_eq!((p.reads, p.writes.len()), (reads, writes), "lent neither reads nor writes");
    assert!(!p.terminal.work_pending(SECOND));
    assert_eq!(p.terminal.next_deadline(), None);
    p.sim.set_window(p.pid, p.input, skein_io::kernel::Size { rows: 12, columns: 34 });
    p.terminal.resume();
    assert!(!p.terminal.lent());
    p.drive(SECOND);
    expected.clear();
    draw(None, &newest, Depth::True, &mut expected);
    assert!(p.collect() == [ENTER, expected.as_slice()].concat(), "no background query, newest frame whole");
    assert_eq!(p.windows, 1);
    assert_eq!(p.terminal.size(), Size { rows: 12, columns: 34 });
    assert_eq!(p.terminal.background(), background);
    assert!(p.reads > reads, "reading restarts");
    p.feed(b"q");
    assert_eq!(p.terminal.keys().pop(), Some(Key::Char('q')));
    p.finish();
}

#[test]
fn a_read_that_wins_lending_is_decoded_and_its_keys_wait() {
    for already_completed in [false, true] {
        let mut p = start(OUTPUT, SIZE);
        p.drain();
        if already_completed {
            assert_eq!(p.sim.peer_feed(p.pid, p.input, b"a"), 1);
        }
        p.terminal.lend();
        if !already_completed {
            // The cancellation has been asked for, but not submitted yet.
            p.terminal.iterate(Time::ZERO);
            assert_eq!(p.sim.peer_feed(p.pid, p.input, b"a"), 1);
        }
        p.drive(Time::ZERO);
        assert_eq!(p.collect(), LEAVE);
        assert!(p.terminal.lent());
        assert_eq!(p.sim.in_flight(p.pid), 0);
        p.terminal.resume();
        p.drive(SECOND);
        assert_eq!(p.terminal.keys().pop(), Some(Key::Char('a')));
        assert_eq!(p.terminal.keys().pop(), None);
        p.finish();
    }
}

#[test]
fn lending_and_resuming_are_inert_outside_their_live_phases() {
    let mut p = start(OUTPUT, SIZE);
    p.drain();
    p.terminal.resume();
    p.drive(SECOND);
    assert!(p.drain().is_empty(), "resume while open is inert");
    assert_eq!(p.windows, 0);
    p.terminal.lend();
    p.terminal.resume();
    assert_eq!(p.collect(), LEAVE, "resume while lending is inert");
    assert!(p.terminal.lent());
    p.terminal.close();
    assert!(p.terminal.closed(), "closing while lent is immediate");
    p.terminal.resume();
    p.terminal.lend();
    assert!(!p.terminal.lent());
    assert!(p.collect().is_empty(), "close while lent adds no output");
    p.finish();

    let mut closing = start(1, SIZE);
    closing.terminal.close();
    closing.terminal.lend();
    closing.terminal.resume();
    assert_eq!(closing.collect(), [ENTER, LEAVE].concat());
    assert!(closing.terminal.closed());
    assert!(!closing.terminal.lent());
    closing.terminal.lend();
    closing.terminal.resume();
    assert_eq!(closing.collect(), Vec::<u8>::new());
    closing.finish();

    // A window that answers Gone cannot supply the whole-frame flag for resume.
    let mut gone = start_with_window(OUTPUT, SIZE, false);
    let a = frame(SIZE, SIZE, 'A');
    gone.terminal.draw(&a);
    gone.drive(Time::ZERO);
    gone.drain();
    for _ in 0..2 {
        gone.terminal.lend();
        assert_eq!(gone.collect(), LEAVE);
        assert!(gone.terminal.lent());
        gone.terminal.draw(&a);
        gone.terminal.resume();
        gone.drive(SECOND);
        assert_eq!(gone.drain(), [ENTER, WHOLE_A].concat(), "resume draws even an unchanged frame whole");
    }
    gone.finish();
}
