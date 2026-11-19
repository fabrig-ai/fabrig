//! The box pipe on the world's clock (temper/docs/design/client/journey-walks.md, 4.4).
use skein_io::kernel::{Complete, Done, Fd, Op, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::Host;
use std::{cell::RefCell, collections::BTreeSet, rc::Rc};
use temper_forge_setup::{
    machine::BoxKind,
    place::{File, Place},
    setup_file,
};
use temper_playground::{machine, programs, service::read_handover};

pub(crate) const PROGRAM: &[u8] = b"/world/box-peer";
pub(crate) const LINE_BYTES: u32 = 256;

/// A line relative to the owned start. Its forge number is supplied at the pipe.
pub struct Line {
    pub after: Duration,
    verb: &'static str,
    rest: &'static str,
}
impl Line {
    #[must_use]
    pub fn bytes(&self, forge: u16) -> Box<[u8]> {
        format!("{} {forge}{}\n", self.verb, self.rest).into_bytes().into()
    }
}

/// The owned Forgejo's script, including its terminal handover.
pub const OWNED: &[Line] = &[
    Line { after: Duration::ZERO, verb: "step", rest: " fetch" },
    Line { after: Duration::from_secs(2), verb: "fetched", rest: " forgejo 25000000 100000000" },
    Line { after: Duration::from_secs(4), verb: "fetched", rest: " forgejo 100000000 100000000" },
    Line { after: Duration::from_secs(6), verb: "fetched", rest: " runner 10000000 20000000" },
    Line { after: Duration::from_secs(8), verb: "fetched", rest: " runner 20000000 20000000" },
    Line { after: Duration::from_secs(10), verb: "step", rest: " prepare" },
    Line { after: Duration::from_secs(12), verb: "step", rest: " run" },
    Line { after: Duration::from_secs(14), verb: "started", rest: "" },
];

struct Running {
    forge: u16,
    began: Time,
    next: usize,
}
#[derive(Default)]
struct Answers {
    running: Option<Running>,
    ended: BTreeSet<u16>,
    input: Vec<u8>,
    overflow: bool,
}
impl Answers {
    fn ask(&mut self, line: &[u8], owned: bool, now: Time) -> Option<Box<[u8]>> {
        let line = std::str::from_utf8(line).ok()?;
        let (verb, number) = line.split_once(' ')?;
        let forge = number.parse::<u16>().ok()?;
        match verb {
            "start" => {
                if let Some(running) = &self.running {
                    return (running.forge != forge).then(|| format!("failed {forge} busy\n").into_bytes().into());
                }
                if self.ended.contains(&forge) {
                    return Some(format!("started {forge}\n").into_bytes().into());
                }
                if owned {
                    self.running = Some(Running { forge, began: now, next: 0 });
                }
                None
            }
            "stop" | "remove" => {
                if self.running.as_ref().is_some_and(|running| running.forge == forge) {
                    self.running = None;
                }
                self.ended.remove(&forge);
                let answer = if verb == "stop" { "stopped" } else { "removed" };
                Some(format!("{answer} {forge}\n").into_bytes().into())
            }
            _ => None,
        }
    }
    fn byte(&mut self, byte: u8) -> Option<Vec<u8>> {
        if byte == b'\n' {
            let line = (!self.overflow).then(|| std::mem::take(&mut self.input));
            self.input.clear();
            self.overflow = false;
            line
        } else {
            // The bound includes the terminating line feed.
            if self.input.len() < usize::try_from(LINE_BYTES - 1).expect("line bound") {
                self.input.push(byte);
            } else {
                self.overflow = true;
            }
            None
        }
    }
    fn due(&mut self, now: Time) -> Option<(u16, &'static Line, bool)> {
        if self.deadline().is_none_or(|at| at > now) {
            return None;
        }
        let running = self.running.as_mut().expect("due script");
        let forge = running.forge;
        let line = OWNED.get(running.next).expect("script line");
        running.next += 1;
        let last = running.next == OWNED.len();
        if last {
            self.ended.insert(forge);
            self.running = None;
        }
        Some((forge, line, last))
    }
    fn deadline(&self) -> Option<Time> {
        let running = self.running.as_ref()?;
        Some(running.began.saturating_add(OWNED.get(running.next)?.after))
    }
}

/// A hosted pipe peer; no box programs, ctl, or extra network run here.
pub struct BoxPeer {
    requests: Option<Fd>,
    answers: Option<Fd>,
    disk: Rc<RefCell<skein_fake_machine::Machine>>,
    seed: u64,
    script: Answers,
    output: std::collections::VecDeque<Box<[u8]>>,
    writing: bool,
    pending: usize,
    serial: u64,
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
}
impl BoxPeer {
    pub(crate) fn program(
        _: &skein_io::kernel::Spawn,
        inherited: &skein_sim_world::Inherited,
    ) -> programs::Process<crate::Clients> {
        let pipe = |child| inherited.pipes.iter().find(|(number, _)| *number == child).expect("box pipe").1;
        let mut peer = Self {
            requests: Some(pipe(3)),
            answers: Some(pipe(4)),
            disk: programs::parameters().disk,
            seed: programs::parameters().seed,
            script: Answers::default(),
            output: std::collections::VecDeque::new(),
            writing: false,
            pending: 0,
            serial: 0,
            completions: Queue::with_capacity(16),
            submissions: Queue::with_capacity(16),
        };
        peer.submit(Op::Close { fd: inherited.signal });
        peer.read(pipe(3));
        programs::Process::Other(crate::Native::Box(Box::new(peer)))
    }
    fn submit(&mut self, kind: Op) {
        self.serial += 1;
        self.pending += 1;
        self.submissions.push(Submit { op: Token::new(self.serial), kind });
    }
    fn read(&mut self, fd: Fd) {
        self.submit(Op::PipeRead { fd, buf: vec![0; usize::try_from(LINE_BYTES).expect("line bytes")].into() });
    }
    fn setup(&self, forge: u16) -> Option<temper_forge_setup::setup::Entry> {
        let directories =
            jig_protocol_config::Directories::given(b"/config", b"/setup", b"/cache").expect("world directories");
        let path = directories
            .join(jig_protocol_config::Directory::Data, &setup_file::path(forge, BoxKind::Owned))
            .expect("setup path");
        let read = temper_service::HandoverRead { forge, run: Token::new(0), path, max: 65536 };
        let bytes = read_handover(&mut self.disk.borrow_mut(), &read).ok()??;
        setup_file::decode(bytes.expose(), &directories).ok()?.entry(forge, &directories)
    }
    /// When the currently running start was read, for checks on the world's time.
    #[must_use]
    pub fn began(&self) -> Option<Time> {
        self.script.running.as_ref().map(|running| running.began)
    }
    fn due(&mut self, now: Time) {
        let Some((forge, line, last)) = self.script.due(now) else { return };
        if last {
            let entry = self.setup(forge).expect("owned group's setup");
            let path = Place { directory: entry.directory, user: Box::new([]) }.path(File::AdminToken);
            machine::lay_handover(
                &mut self.disk.borrow_mut(),
                &path,
                &temper_playground::stories::admin_token(self.seed),
            );
        }
        self.output.push_back(line.bytes(forge));
    }
}
impl Host for BoxPeer {
    fn iterate(&mut self, now: Time, _: Wall) {
        while let Some(complete) = self.completions.pop() {
            self.pending -= 1;
            match (complete.kind, complete.result.expect("box peer operation")) {
                (Op::PipeRead { fd, buf }, Done::Count(count)) => {
                    if count == 0 {
                        self.requests = None;
                        self.script.running = None;
                        self.submit(Op::Close { fd });
                    } else {
                        for byte in buf.iter().take(usize::try_from(count).expect("read size")) {
                            if let Some(line) = self.script.byte(*byte) {
                                let owned = std::str::from_utf8(&line)
                                    .ok()
                                    .and_then(|s| s.strip_prefix("start "))
                                    .and_then(|s| s.parse().ok())
                                    .is_some_and(|forge| self.setup(forge).is_some());
                                if let Some(answer) = self.script.ask(&line, owned, now) {
                                    self.output.push_back(answer);
                                }
                            }
                        }
                        self.read(fd);
                    }
                }
                (Op::PipeWrite { fd, bytes, from }, Done::Count(count)) => {
                    let from = from + count;
                    if usize::try_from(from).expect("write offset") < bytes.len() {
                        self.submit(Op::PipeWrite { fd, bytes, from });
                    } else {
                        self.writing = false;
                    }
                }
                (Op::Close { .. }, Done::Nothing) => {}
                (op, done) => panic!("box completion {op:?}: {done:?}"),
            }
        }
        self.due(now);
        if !self.writing {
            if let Some(bytes) = self.output.pop_front() {
                self.writing = true;
                self.submit(Op::PipeWrite { fd: self.answers.expect("answers open"), bytes, from: 0 });
            } else if self.requests.is_none()
                && let Some(fd) = self.answers.take()
            {
                self.submit(Op::Close { fd });
            }
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, now: Time) -> bool {
        !self.completions.is_empty()
            || self.script.deadline().is_some_and(|at| at <= now)
            || (!self.writing && !self.output.is_empty())
    }
    fn next_mechanism_deadline(&self) -> Option<Time> {
        self.script.deadline()
    }
    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }
    fn is_empty(&self) -> bool {
        self.requests.is_none() && self.answers.is_none() && self.pending == 0
    }
    fn operations(&self) -> u32 {
        16
    }
}

#[cfg(test)]
#[path = "box_peer/tests.rs"]
mod tests;
