use skein_fake_machine::{Item, Machine};
use skein_io::kernel::{Exit, Fd, Op, ServiceSignal};
use skein_lib::{Time, Wall};
use skein_shell::Host;
use skein_sim::{Config, Handle, Pid, Sim};
use skein_ui_screen::Size;
use skein_ui_screen_terminal::{Depth, ENTER, LEAVE};
use temper_client_shell_tui::{
    limits,
    run::{Run, Settings},
};
use temper_client_tui::Scheme;

const USER: u32 = 1000;
const SEED: u64 = 7;
const SIZE: Size = Size { rows: 24, columns: 80 };
const ROOM: usize = 65_536;
const SIGN_IN: &[u8] = b"Sign in this terminal to 127.0.0.1:8367";

struct World {
    sim: Sim,
    files: Machine,
    pid: Pid,
    input: Fd,
    output: Fd,
    signals: Fd,
    run: Run,
    now: Time,
    bytes: Vec<u8>,
}
impl World {
    fn new(refuse: bool) -> Self {
        let mut files = Machine::for_user(USER);
        let root = files.lay(&[Item::directory(b"tui").mode(if refuse { 0o755 } else { 0o700 })]);
        let mut sim = Sim::new(SEED, Config::calm());
        let pid = sim.spawn_process();
        let root = sim.root(pid, Handle::new(root.raw()));
        let input = sim.open_inherited_read(pid);
        let output = sim.open_inherited_write(pid);
        let signals = sim.open_signal_source(pid);
        let settings = Settings {
            engine: skein_io::Addr::Ip("127.0.0.1:8367".parse().expect("loopback")),
            directory: b"tui".as_slice().into(),
            address: b"127.0.0.1:8367".as_slice().into(),
            scheme: Scheme::Dark,
            depth: Depth::Indexed,
            size: SIZE,
            limits: limits::tui(),
        };
        let run = Run::new(settings, root, USER, SEED, input, output, signals).expect("adopt descriptors");
        Self { sim, files, pid, input, output, signals, run, now: Time::ZERO, bytes: Vec::new() }
    }

    fn settle(&mut self) {
        for _ in 0..512 {
            self.sim.reap(self.pid, self.run.completions());
            self.run.iterate(self.now, Wall::EPOCH);
            for submit in self.run.submissions().iter() {
                if let Op::Socket { .. } | Op::Connect { .. } = &submit.kind {
                    panic!("a signed-out terminal never reaches an engine");
                }
            }
            self.sim.submit(self.pid, self.run.submissions());
            skein_fake_machine::serve(&mut self.files, &mut self.sim);
            self.bytes.extend(self.sim.peer_drain(self.pid, self.output, ROOM));
            if self.sim.ready(self.pid) > 0 || self.sim.deferred(self.pid) || self.run.work_pending(self.now) {
                continue;
            }
            let next = [self.sim.next_due(), self.run.next_mechanism_deadline(), self.run.next_policy_deadline()]
                .into_iter()
                .flatten()
                .min();
            if let Some(next) = next {
                assert!(next > self.now, "idle deadline is in the future");
                self.now = next;
                self.sim.advance_to(next);
            } else {
                return;
            }
        }
        panic!("the run did not settle: {}", self.sim.render_trace());
    }

    fn drained(&mut self, refused: bool) {
        assert!(self.bytes.starts_with(ENTER), "ENTER first");
        assert!(self.bytes.ends_with(LEAVE), "LEAVE last");
        assert_eq!(self.bytes.windows(LEAVE.len()).filter(|part| *part == LEAVE).count(), 1);
        assert_eq!(self.run.refused(), refused);
        assert_eq!(self.run.exit(), Some(Exit::Code(u8::from(refused))));
        assert!(self.run.is_empty());
        assert!(self.run.signed_in().is_none());
        self.sim.assert_no_open_fds(self.pid);
        self.sim.assert_quiescent(self.pid);
        assert_eq!(self.files.open_handles(), 0);
        let length = self.bytes.len();
        self.settle();
        assert_eq!(self.bytes.len(), length, "nothing follows LEAVE");
    }
}

#[derive(Debug)]
enum WayOut {
    End,
    ControlC,
    Termination,
    Refused,
}

#[test]
fn every_way_out_leaves_the_terminal_as_it_was_found() {
    let clock = skein_shell::Clock::new();
    for way in [WayOut::End, WayOut::ControlC, WayOut::Termination, WayOut::Refused] {
        let began = clock.now().now;
        let mut world = World::new(matches!(way, WayOut::Refused));
        world.settle();
        if matches!(way, WayOut::Refused) {
            assert!(!world.bytes.windows(SIGN_IN.len()).any(|part| part == SIGN_IN), "no Start on refusal");
            world.drained(true);
            println!("{way:?}: {} ns", clock.now().now.as_nanos().saturating_sub(began.as_nanos()));
            continue;
        }
        assert!(world.bytes.starts_with(ENTER), "{way:?}: ENTER");
        assert!(world.bytes.windows(SIGN_IN.len()).any(|part| part == SIGN_IN), "{way:?}: Start drawn");
        assert_eq!(world.run.exit(), None);
        match way {
            WayOut::End => world.sim.peer_close(world.pid, world.input),
            WayOut::ControlC => {
                assert_eq!(world.sim.peer_feed(world.pid, world.input, b"\x03\rignored"), 9);
            }
            WayOut::Termination => {
                world.sim.deliver_service_signal(world.pid, world.signals, ServiceSignal::Terminate, None);
            }
            WayOut::Refused => unreachable!("refusal checked above"),
        }
        world.settle();
        world.drained(false);
        println!("{way:?}: {} ns", clock.now().now.as_nanos().saturating_sub(began.as_nanos()));
    }
}
