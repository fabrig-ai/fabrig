//! Line-wise recipe/drive IO on the process world's supervised binary.
use super::{Place, kept_project, no_link};
use crate::support::{boxed, patience::Patience};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Submit};
use skein_lib::{Queue, Time, Token, Wall};
use skein_shell::{Delegate, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Cleanup, Command, Expectation, Mode, Streams},
    real,
};

const READ: Token = Token::new(1);
const WRITE: Token = Token::new(2);
const CLOSE: Token = Token::new(3);
struct Lines {
    input: Option<Fd>,
    output: Option<Fd>,
    reading: bool,
    writing: bool,
    closing: u32,
    finish: bool,
    send: Option<Box<[u8]>>,
    pending: Vec<u8>,
    length: usize,
    received: Vec<String>,
    complete: Queue<Complete>,
    submit: Queue<Submit>,
}
impl Lines {
    fn new(input: Fd, output: Fd, walk: bool) -> Self {
        Self {
            input: Some(input),
            output: Some(output),
            reading: false,
            writing: false,
            closing: 0,
            finish: walk,
            send: None,
            pending: Vec::new(),
            length: 0,
            received: Vec::new(),
            complete: Queue::with_capacity(8),
            submit: Queue::with_capacity(8),
        }
    }

    fn line(&mut self, line: &str) {
        assert!(self.send.is_none() && !self.writing, "one drive command at a time");
        self.send = Some(format!("{line}\n").into_bytes().into_boxed_slice());
    }

    fn close(&mut self, fd: Fd) {
        self.closing += 1;
        self.submit.push(Submit { op: CLOSE, kind: Op::Close { fd } });
    }

    fn receive(&mut self, bytes: &[u8]) {
        self.length = self.length.checked_add(bytes.len()).expect("stdout length");
        assert!(self.length <= 1_048_576, "bounded drive stdout");
        self.pending.extend_from_slice(bytes);
        no_link(&self.pending);
        while let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line = self.pending.drain(..=end).collect::<Vec<_>>();
            self.received.push(String::from_utf8(line).expect("drive UTF-8").trim_end_matches(['\r', '\n']).into());
        }
    }
}
impl Host for Lines {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        while let Some(done) = self.complete.pop() {
            if done.op == READ {
                self.reading = false;
                let Op::PipeRead { buf, .. } = done.kind else { panic!("stdout read") };
                let Done::Count(count) = done.result.expect("drive stdout read") else { panic!("read count") };
                self.receive(&buf[..usize::try_from(count).expect("bounded stdout count")]);
                if count == 0 {
                    assert!(self.pending.is_empty(), "complete stdout lines");
                    let fd = self.output.take().expect("stdout pipe");
                    self.close(fd);
                    self.finish = true;
                }
            } else if done.op == WRITE {
                let Op::PipeWrite { fd, bytes, from } = done.kind else { panic!("drive command") };
                let Done::Count(count) = done.result.expect("drive command write") else { panic!("write count") };
                assert_ne!(count, 0, "drive command makes progress");
                let from = from.checked_add(count).expect("bounded command");
                if usize::try_from(from).expect("command offset") < bytes.len() {
                    self.submit.push(Submit { op: WRITE, kind: Op::PipeWrite { fd, bytes, from } });
                } else {
                    self.writing = false;
                }
            } else {
                assert_eq!(done.op, CLOSE);
                done.result.expect("drive pipe closed");
                self.closing -= 1;
            }
        }
        if !self.reading
            && let Some(fd) = self.output
        {
            self.reading = true;
            self.submit.push(Submit { op: READ, kind: Op::PipeRead { fd, buf: vec![0; 4096].into() } });
        }
        if !self.writing
            && let Some(fd) = self.input
        {
            if let Some(bytes) = self.send.take() {
                self.writing = true;
                self.submit.push(Submit { op: WRITE, kind: Op::PipeWrite { fd, bytes, from: 0 } });
            } else if self.finish {
                self.input = None;
                self.close(fd);
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.complete
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submit
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.complete.is_empty()
            || !self.submit.is_empty()
            || self.send.is_some()
            || self.finish && self.input.is_some() && !self.writing
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        None
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.input.is_none()
            && self.output.is_none()
            && self.closing == 0
            && self.complete.is_empty()
            && self.submit.is_empty()
    }
    fn operations(&self) -> u32 {
        8
    }
}

enum Root {
    Binary(Box<Binary>),
    Lines(Box<Lines>),
}
impl Delegate for Root {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Binary(x) => x.as_ref(),
            Self::Lines(x) => x.as_ref(),
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Binary(x) => x.as_mut(),
            Self::Lines(x) => x.as_mut(),
        }
    }
}

struct Judge<'a> {
    place: &'a Place,
    old: &'a [super::Identity],
    drive: bool,
    cursor: usize,
    answer: String,
    read_project: bool,
    patience: Patience,
    settled: bool,
}
impl Referee<Root> for Judge<'_> {
    fn act(&mut self, _now: Time, roots: &mut [Root]) {
        let [Root::Binary(binary), Root::Lines(lines)] = roots else { panic!("driver and its pipes") };
        no_link(binary.stderr());
        if !self.drive {
            return;
        }
        // Consume only complete lines; ctl's answers are not assumed to fit one pipe read.
        while let Some(line) = lines.received.get(self.cursor).cloned() {
            self.cursor += 1;
            if line == "ready" {
                self.place.processes(self.old);
                self.place.listeners();
                lines.line("ctl project show Hello --json");
            } else if let Some(text) = line.strip_prefix("| ") {
                self.answer.push_str(text);
                self.answer.push('\n');
            } else if line == "ok" {
                kept_project(&serde_json::from_str(&self.answer).expect("ctl project JSON"));
                self.read_project = true;
                lines.line("end");
                lines.finish = true;
            } else if line.starts_with("failed:") {
                panic!("drive project read: {line}");
            }
        }
    }
    fn watch(&mut self, _now: Time, roots: &[Root]) -> Result<(), String> {
        self.settled = roots.iter().all(Host::is_empty);
        if !self.settled
            && let Some(figures) = self.patience.failed()
        {
            return Err(format!("deployment walk/read did not settle: {figures}"));
        }
        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        if self.settled { None } else { self.patience.until() }
    }
    fn passed(&self) -> bool {
        self.settled && (!self.drive || self.read_project)
    }
}

pub(super) fn run(place: &Place, arguments: Vec<String>, drive: bool, old: &[super::Identity]) -> Vec<String> {
    let mut binary = Binary::start_with_tree(
        Command {
            program: place.bin.join("temper-client-journeys"),
            arguments: arguments.into_iter().map(Into::into).collect(),
            directory: place.root.path().into(),
            environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        },
        Mode::Pipes,
        262_144,
        // Supervision retains the leader-exit evidence checked after settlement.
        Expectation::Supervise,
        false,
    )
    .expect("copied journeys starts");
    let streams = binary.take_streams().expect("journeys pipes");
    let fds = streams.descriptors();
    let Streams::Pipes { input, output } = streams else { panic!("piped journeys") };
    let mut world = real::World::new(Judge {
        place,
        old,
        drive,
        cursor: 0,
        answer: String::new(),
        read_project: false,
        patience: Patience::watching("the deployment at its place", boxed::BOX_DEADLINE, &binary),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Root::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || Root::Lines(Box::new(Lines::new(input, output, !drive))));
    let ended = world.run(&skein_shell::Clock::new(), boxed::BOX_DEADLINE);
    let [Root::Binary(binary), Root::Lines(lines)] = ended.procs.as_slice() else {
        panic!("journeys and pipes settled")
    };
    no_link(binary.stderr());
    assert!(!binary.stderr_overflowed(), "bounded deployment stderr");
    // Status 3 is a failure here, not permission to recreate the copy. Its named line is in this diagnostic.
    assert_eq!(
        binary.exit_status(),
        Some(Exit::Code(0)),
        "deployment exit; lines: {}; stderr: {}",
        lines.received.join("\n"),
        String::from_utf8_lossy(binary.stderr())
    );
    assert!(ended.procs.iter().all(Host::is_empty), "all IO settled");
    assert_eq!(binary.tree_status().live_at_leader_exit, Some(Vec::new()), "nothing outlives walk/end");
    assert_eq!(binary.tree_status().cleanup(), Cleanup::None, "no test cleanup needed");
    assert!(binary.tree_status().live_pids.is_empty(), "no deployment process left");
    place.logs();
    lines.received.clone()
}
