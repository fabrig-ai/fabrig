//! The terminal startup, drive and restoration path (runtime/shell.md, section 6.3).
use skein_io::kernel::{Complete, Fd, Submit};
use skein_io::{Event, Io, Limits, Request, Window};
use skein_lib::stream::{Down, Read, Up};
use skein_lib::{Env, Queue, Time, Token, Wall};
use skein_shell::{
    Clock, Config, Host, Input, Kept, Keys, Watch, close_startup_fd, drive, open_signals, open_terminal, raw_input,
    take_foreground,
};
use std::{
    collections::VecDeque,
    fs::File,
    io::{Read as _, Write as _},
    os::unix::process::CommandExt,
    process::{Command, ExitCode},
};

struct Terminal {
    io: Io,
    limits: Limits,
    input: Token,
    signals: Token,
    output: Token,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    events: Queue<Event>,
    requests: Queue<Request>,
    lines: VecDeque<Box<[u8]>>,
    ending: bool,
    writing: bool,
}
impl Terminal {
    fn line(&mut self, line: Vec<u8>) {
        self.lines.push_back(line.into_boxed_slice());
    }
    fn end(&mut self) {
        if !self.ending {
            self.ending = true;
            for entity in [self.input, self.signals] {
                self.requests.push(Request::Abort { entity });
            }
        }
    }
    fn event(&mut self, event: Event) {
        match event {
            Event::Stream { owner, up: Up::Bytes(bytes) } if owner == self.input => {
                for key in &bytes {
                    self.line(vec![b'k', b'e', b'y', b' ', *key, b'\n']);
                    if *key == b'q' {
                        self.end();
                    }
                }
                if !self.ending {
                    self.requests.push(Request::Stream {
                        stream: self.input,
                        down: Down::Demand { read: Read::Fill(1), room: 0 },
                    });
                }
            }
            Event::Stream { owner, up: Up::Room } if owner == self.output => {
                let bytes = self.lines.pop_front().expect("requested room for a line");
                self.requests.push(Request::Stream { stream: self.output, down: Down::Send(bytes) });
                self.writing = false;
            }
            Event::Resized => self.requests.push(Request::Window { owner: self.input, stream: self.input }),
            Event::Window { window: Window::Size(size), .. } => {
                self.line(format!("size {}x{}\n", size.rows, size.columns).into_bytes());
            }
            Event::Window { window: Window::Gone, .. }
            | Event::Shutdown { .. }
            | Event::Failed { .. }
            | Event::Stream { up: Up::End, .. } => self.end(),
            Event::Closed { .. }
            | Event::Stream { .. }
            | Event::Output { .. }
            | Event::Listening { .. }
            | Event::Accepted { .. }
            | Event::Connecting { .. }
            | Event::Connected { .. }
            | Event::Spawned { .. }
            | Event::Exited { .. }
            | Event::Usage { .. } => {}
        }
    }
}
impl Host for Terminal {
    fn iterate(&mut self, now: Time, wall: Wall) {
        let env = Env { now, wall, limits: self.limits };
        skein_io::stage::up(&mut self.io, &env, &mut self.completions, &mut self.events, &mut self.submissions, now);
        while let Some(event) = self.events.pop() {
            self.event(event);
        }
        if !self.writing {
            if let Some(line) = self.lines.front() {
                self.writing = true;
                self.requests.push(Request::Stream {
                    stream: self.output,
                    down: Down::Demand { read: Read::Nothing, room: u32::try_from(line.len()).expect("small line") },
                });
            } else if self.ending {
                self.requests.push(Request::Close { entity: self.output });
            }
        }
        skein_io::stage::down(&mut self.io, &env, &mut self.requests, &mut self.submissions);
        self.io.reclaim();
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        self.io.is_ready() || !self.completions.is_empty()
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.io.is_empty() && self.lines.is_empty() && self.requests.is_empty() && self.events.is_empty()
    }
    fn operations(&self) -> u32 {
        skein_io::operations(&self.limits).expect("small limits")
    }
}
#[expect(clippy::disallowed_methods, reason = "skein's round: reads the terminal fixture command line")]
fn main() -> ExitCode {
    skein_fail_stop::install();
    let option = std::env::args().nth(1);
    match option.as_deref() {
        Some("child") => return child(),
        Some("foreground") => {
            let status = Command::new(std::env::current_exe().expect("the fixture's executable"))
                .arg("child")
                .process_group(0)
                .status()
                .expect("a child in its own process group");
            if !status.success() {
                return ExitCode::FAILURE;
            }
            let terminal = open_terminal().expect("the parent's controlling terminal");
            take_foreground(terminal).expect("the parent takes its terminal back");
            close_startup_fd(terminal);
            eprintln!("back");
        }
        Some(_) | None => {}
    }
    let keys = if option.as_deref() == Some("kept-signal") { Keys::Signal } else { Keys::Pass };
    let Input::Terminal { kept, .. } = raw_input(Fd::new(0), keys).expect("terminal input") else {
        return ExitCode::FAILURE;
    };
    if matches!(option.as_deref(), Some("kept-pass" | "kept-signal")) {
        modes_cycle(kept);
        return ExitCode::SUCCESS;
    }
    let result = run();
    kept.restore().expect("startup modes restored after drive");
    result
}
fn child() -> ExitCode {
    let terminal = open_terminal().expect("the child's controlling terminal");
    take_foreground(terminal).expect("the child takes the terminal's foreground");
    // The descriptor's alias makes a blocking File without an unsafe ownership conversion.
    let mut input = File::open(format!("/dev/fd/{}", terminal.raw())).expect("the terminal as a blocking file");
    let Input::Terminal { kept, .. } = raw_input(terminal, Keys::Pass).expect("the child's terminal modes") else {
        close_startup_fd(terminal);
        return ExitCode::FAILURE;
    };
    eprintln!("foreground");
    let mut key = [0];
    let result = input.read_exact(&mut key);
    kept.restore().expect("the child restores its terminal modes");
    drop(input);
    close_startup_fd(terminal);
    result.expect("one key from the terminal");
    eprintln!("child key {}", char::from(key[0]));
    ExitCode::SUCCESS
}
fn run() -> ExitCode {
    let limits = Limits { sockets: 3, intake: 8, receive: 1, output: 256, sends: 8, ..Limits::TEST };
    let mut io = Io::new(&limits);
    let input = io.adopt_read_pipe(Fd::new(0)).expect("input");
    let output = io.adopt_write_pipe(Fd::new(2), None).expect("stderr");
    let signals = io.adopt_signals(open_signals(Watch::TerminationAndWindow).expect("signals")).expect("source");
    let mut host = Terminal {
        io,
        limits,
        input,
        output,
        signals,
        completions: Queue::with_capacity(32),
        submissions: Queue::with_capacity(32),
        events: Queue::with_capacity(32),
        requests: Queue::with_capacity(32),
        lines: VecDeque::from([Box::from(b"ready\n".as_slice())]),
        ending: false,
        writing: false,
    };
    host.requests.push(Request::Stream { stream: input, down: Down::Demand { read: Read::Fill(1), room: 0 } });
    let Ok(mut kernel) = skein_shell::Kernel::open(Config { operations: host.operations() }) else {
        return ExitCode::FAILURE;
    };
    let _exit = drive(&mut kernel, &Clock::new(), &mut host);
    ExitCode::SUCCESS
}

fn modes_cycle(kept: Kept) {
    let mut input = File::open("/dev/fd/0").expect("the terminal as a blocking file");
    eprintln!("raw");
    input.read_exact(&mut [0]).expect("the person observed raw modes");
    kept.set_kept().expect("kept modes without giving them up");
    eprintln!("kept");
    input.read_exact(&mut [0; 2]).expect("the person observed kept modes and sent a line");
    kept.set_raw().expect("startup raw modes again");
    eprintln!("raw again");
    input.read_exact(&mut [0]).expect("the person observed raw modes again");
    kept.restore().expect("kept modes at the end");
    eprintln!("restored");
    input.read_exact(&mut [0; 2]).expect("the person observed restored modes and sent a line");
    std::io::stdout().write_all(b"done").expect("a final marker before terminal EOF");
}
