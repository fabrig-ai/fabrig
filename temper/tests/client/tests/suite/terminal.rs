//! Print and read the joint world's terminal pages (client/line.md, 5).
use jig_client_kit::harness::{ClientId, Until, World};
use skein_io::kernel::Fd;
use skein_ui_text::{Command, Print};
use skein_world::domain::Verdict;
use temper_client_shell_terminal::{
    limits,
    terminal::{Client, Loop, Notice},
};
use temper_client_world::{app::Temper, fixtures, front::Joint};

type TerminalWorld = World<Temper, Joint>;

fn print(world: &TerminalWorld, client: ClientId, limits: &temper_client_domain::Limits) -> Print {
    let words = fixtures::phrasebooks(limits);
    let bounds = fixtures::view_bounds(limits, &words);
    let mut print = Print::new(&bounds.tree).expect("terminal print bounds");
    skein_ui_text::render(world.tree(client), b"temper", None, &mut print);
    print
}

pub fn printed(world: &TerminalWorld, client: ClientId, limits: &temper_client_domain::Limits) -> String {
    let print = print(world, client, limits);
    String::from_utf8(print.text.as_slice().to_vec()).expect("printed UTF-8")
}

/// Lines without their Enter, dispatched as the terminal composition dispatches them.
pub fn line(world: &mut TerminalWorld, client: ClientId, bytes: &[u8]) -> String {
    let limits = fixtures::terminal::reading_limits();
    let print = print(world, client, &limits);
    let command = skein_ui_text::command(
        bytes,
        Some(print.numbers.as_slice()),
        world.tree(client),
        temper_client_shell_terminal::limits::terminal().line,
    );
    match command {
        Command::Act { node, first, submit } => {
            world.act(client, node, first);
            if submit {
                world.act(client, node, skein_ui::Act::Submit);
            }
        }
        command => panic!(
            "expected a terminal act for {bytes:?}, got {command:?}: {}",
            String::from_utf8_lossy(print.text.as_slice())
        ),
    }
    let outcome = world.run(Until::Done);
    assert_eq!(outcome.verdict, Verdict::Passed, "{outcome:?}");
    printed(world, client, &limits)
}

pub struct Page {
    pub commands: u32,
    pub text: String,
}

/// The shipped line loop at the joint world's client boundary. Only the two
/// pipes meet the simulator; the client's transport still meets `Joint`.
pub struct LineTerminal {
    line: Loop,
    sim: skein_sim::Sim,
    pid: skein_sim::Pid,
    stdin: Fd,
    stdout: Fd,
    commands: u32,
    client_closed: bool,
    said_closed: bool,
    pub pages: Vec<Page>,
}

impl LineTerminal {
    pub fn new(seed: u64) -> Self {
        let mut sim = skein_sim::Sim::new(seed, skein_sim::Config::calm());
        let pid = sim.spawn_process();
        let stdin = sim.open_inherited_read(pid);
        let stdout = sim.open_inherited_write(pid);
        let (view, _) = limits::view(&limits::native()).expect("terminal view bounds");
        // Sample the redraws without waiting for the print interval.
        let mut line = Loop::new(
            &limits::Terminal { interval: skein_lib::Duration::ZERO, ..limits::terminal() },
            &view.tree,
            Box::from(*b"line-private"),
            None,
        );
        line.terminal.adopt(stdin, stdout).expect("terminal pipes");
        Self { line, sim, pid, stdin, stdout, commands: 0, client_closed: false, said_closed: false, pages: Vec::new() }
    }

    pub fn type_line(&mut self, command: &str) {
        assert!(self.line.exit().is_none());
        let stdin = format!("{command}\n");
        assert_eq!(self.sim.peer_feed(self.pid, self.stdin, stdin.as_bytes()), stdin.len());
        self.commands += 1;
    }

    pub fn step(&mut self, world: &mut TerminalWorld, client: ClientId) -> Option<&str> {
        let outcome = world.run(Until::Steps(1));
        assert!(matches!(outcome.verdict, Verdict::Passed | Verdict::Open { .. }), "{outcome:?}");
        self.sim.advance_to(outcome.now);
        self.sim.reap(self.pid, self.line.terminal.completions());
        self.line.iterate(
            outcome.now,
            &mut JointClient { world, client, built: outcome.callbacks, closed: &mut self.client_closed },
        );
        self.sim.submit(self.pid, self.line.terminal.submissions());
        // In a calm simulator, a short chat page is one complete pipe write.
        let bytes = self.sim.peer_drain(self.pid, self.stdout, 65_536);
        if bytes == b"temper: closed\n" {
            self.said_closed = true;
        } else if !bytes.is_empty() {
            let text = String::from_utf8(bytes).expect("printed UTF-8");
            assert!(text.starts_with("== ") && text.ends_with('\n'), "expected a whole printed page: {text}");
            self.pages.push(Page { commands: self.commands, text });
            return self.pages.last().map(|page| page.text.as_str());
        }
        None
    }

    pub fn drained(&self) -> bool {
        let Some(exit) = self.line.exit() else { return false };
        assert_eq!(exit, (0, None), "the terminal exits successfully after its drain");
        assert!(self.client_closed && self.said_closed, "q closes the client and prints closed");
        assert!(self.sim.peer_eof(self.pid, self.stdout), "the shell's output drained to EOF");
        true
    }
}

struct JointClient<'a> {
    world: &'a mut TerminalWorld,
    client: ClientId,
    built: u64,
    closed: &'a mut bool,
}
impl Client for JointClient<'_> {
    fn tree(&self) -> &skein_ui::Tree {
        self.world.tree(self.client)
    }
    fn built(&self) -> u64 {
        self.built
    }
    fn act(&mut self, node: skein_ui::NodeId, act: skein_ui::Act) -> bool {
        self.world.act(self.client, node, act);
        true
    }
    fn sign_in(&mut self, _: &[u8]) -> Result<(), jig_client_protocol_http::Unlinked> {
        unreachable!("the joint world binds the client before the terminal opens")
    }
    fn notice(&mut self) -> Option<Notice> {
        None
    }
    fn close(&mut self) {
        *self.closed = true;
    }
}
