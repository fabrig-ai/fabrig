//! Command capture over the loop's supplied turns (skein/docs/design/runtime/shell.md, 11).
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Op, Sender, Submit, Way};
use skein_lib::{Queue, Time, Wall};
use skein_shell::{Now, Turns, Wait};
use temper_forge_setup::{prepare, spec::Spec};
use temper_forge_setup_shell::command;

const ROOT: Fd = Fd::new(3);
const SIGNALS: Fd = Fd::new(4);
const PIPES: [Fd; 2] = [Fd::new(6), Fd::new(7)];
const CHILD: Fd = Fd::new(8);
const STREAM_BYTES: usize = 100_000;

// One spawn answer, two finite pipe scripts, and the records needed to settle the loop.
struct Script<'a> {
    program: &'a [u8],
    missing: bool,
    tick: u64,
    signal: Option<Submit>,
    ready: Queue<Complete>,
    remaining: [usize; 2],
    ended: [bool; 2],
    closed: [bool; 2],
}

impl<'a> Script<'a> {
    fn new(program: &'a [u8], missing: bool) -> Self {
        Self {
            program,
            missing,
            tick: 0,
            signal: None,
            ready: Queue::uncapped(),
            remaining: [STREAM_BYTES; 2],
            ended: [false; 2],
            closed: [false; 2],
        }
    }

    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "a command script refuses any submission outside its transcript"
    )]
    fn answer(&mut self, Submit { op, mut kind }: Submit) {
        let result = match &mut kind {
            Op::Spawn { spawn } => {
                assert_eq!(spawn.program.as_ref(), self.program);
                assert_eq!(spawn.root, ROOT);
                assert_eq!(spawn.pipes.len(), 2);
                for (i, pipe) in spawn.pipes.iter_mut().enumerate() {
                    assert_eq!(pipe.child, [1, 2][i]);
                    assert_eq!(pipe.way, Way::Out);
                    assert_eq!(pipe.parent, None);
                    if !self.missing {
                        pipe.parent = Some(PIPES[i]);
                    }
                }
                if self.missing {
                    Err(Error::NotFound)
                } else {
                    Ok(Done::Spawned { pidfd: CHILD, sender: Sender::new(1) })
                }
            }
            Op::PipeRead { fd, buf } => {
                let i = PIPES.iter().position(|pipe| pipe == fd).expect("a scripted pipe");
                assert!(!self.missing && !self.ended[i] && !buf.is_empty());
                let count = self.remaining[i].min(buf.len());
                self.remaining[i] -= count;
                self.ended[i] = count == 0;
                buf[..count].fill(0);
                Ok(Done::Count(u32::try_from(count).expect("one bounded read")))
            }
            Op::Wait { pidfd, reap: false } => {
                assert!(!self.missing);
                assert_eq!(*pidfd, CHILD);
                Ok(Done::Exit(Exit::Code(0)))
            }
            Op::Signal { pidfd, .. } => {
                assert_eq!(*pidfd, CHILD);
                assert_eq!(self.closed, [true; 2], "child settles after both read ends close");
                Ok(Done::Nothing)
            }
            Op::Wait { pidfd, reap: true } => {
                assert_eq!(*pidfd, CHILD);
                Ok(Done::Exit(Exit::Code(0)))
            }
            Op::ReadSignal { fd } => {
                assert_eq!(*fd, SIGNALS);
                assert!(self.signal.is_none());
                self.signal = Some(Submit { op, kind });
                return;
            }
            Op::Cancel { target } => {
                let read = self.signal.take().expect("only the pending signal read is cancelled");
                assert_eq!(*target, read.op);
                self.ready.push(Complete { op: read.op, kind: read.kind, result: Err(Error::Cancelled) });
                Ok(Done::Nothing)
            }
            Op::Close { fd } => {
                if let Some(i) = PIPES.iter().position(|pipe| pipe == fd) {
                    assert!(self.ended[i] && !self.closed[i], "each pipe is read to EOF and closed once");
                    self.closed[i] = true;
                } else {
                    assert!([CHILD, SIGNALS].contains(fd), "only the command's descriptors close");
                }
                Ok(Done::Nothing)
            }
            other => panic!("unexpected command submission: {other:?}"),
        };
        self.ready.push(Complete { op, kind, result });
    }
}

impl Turns for Script<'_> {
    fn reap(&mut self, out: &mut Queue<Complete>) {
        while let Some(complete) = self.ready.pop() {
            out.push(complete);
        }
    }
    fn now(&self) -> Now {
        Now { now: Time::from_nanos(self.tick), wall: Wall::from_nanos(0) }
    }
    fn until(&self, deadline: Option<Time>) -> Option<Time> {
        deadline
    }
    fn in_flight(&self) -> u32 {
        u32::from(self.signal.is_some()) + self.ready.len()
    }
    fn submit(&mut self, records: &mut Queue<Submit>, wait: Wait) {
        while let Some(record) = records.pop() {
            self.answer(record);
        }
        if matches!(wait, Wait::Forever) {
            assert!(!self.ready.is_empty(), "the script must wake a waiting loop");
        }
        self.tick += 1;
        assert!(self.tick <= 2 * u64::try_from(STREAM_BYTES).expect("fixture bound") + 32, "command must settle");
    }
}

fn spec(program: &[u8]) -> Spec {
    Spec {
        program: program.into(),
        arguments: Box::new([]),
        environment: Box::new([]),
        directory: b"/fixture".as_slice().into(),
        output: 17,
    }
}

#[test]
fn output_past_the_bound_is_cut_and_the_command_ends() {
    let spec = spec(b"/programs/capture");
    let mut script = Script::new(&spec.program, false);
    let answer = command::run_with(&mut script, &jig_protocol_config_fixtures::small().io, &spec, ROOT, SIGNALS)
        .expect("command completes");
    assert_eq!(
        (answer.exit, answer.stdout.expose(), answer.stderr.expose()),
        (prepare::Exit::Code(0), &[0; 17][..], &[0; 17][..])
    );
    assert_eq!(script.remaining, [0; 2]);
    assert_eq!(script.ended, [true; 2]);
    assert_eq!(script.closed, [true; 2]);
}

#[test]
fn a_program_that_is_not_there_is_reported_not_run() {
    let spec = spec(b"/no-such-temper-program");
    let mut script = Script::new(&spec.program, true);
    let answer = command::run_with(&mut script, &jig_protocol_config_fixtures::small().io, &spec, ROOT, SIGNALS);
    assert!(matches!(answer, Err(command::Error::Spawn(skein_io::Error::Other))));
}
