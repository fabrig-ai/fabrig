//! A reach, observed inside its tree (io/processes.md, 6; kernel.md, 6.6;
//! testing/README.md, 4). Each scenario submits one probe spawn directly as a
//! kernel record, below io's ABI admission check.

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::vec;
use alloc::vec::Vec;

use skein_fake_machine::Landlock;
use skein_fake_machine::probe::{self, Answer, Ids, Question, Report};
use skein_io::kernel::{Done, Error, Exit, Fd, Op, Pipe, Reach, Rights, Rule, Unenforced, Way};

use crate::processes::{command, spawned};
use crate::run::{Run, unexpected, usize_of};
use crate::{Backend, Check, Item};

/// All answers from one probe, checked in the order the questions were asked.
#[derive(Debug)]
pub struct Observed {
    report: Report,
    expected: Report,
    rule: &'static str,
    exit: Result<Done, Error>,
}

impl Check for Observed {
    fn check(&self) {
        assert_eq!(self.report, self.expected, "{}", self.rule);
        assert_eq!(self.exit, Ok(Done::Exit(Exit::Code(0))), "the probe completed its report (testing/README.md, 4)");
    }
}

/// The spawn's refusal, without a child or parent pipe descriptor.
#[derive(Debug)]
pub struct Refused {
    result: Result<Done, Error>,
    expected: Unenforced,
    abi_above: bool,
    pipes_empty: bool,
}

impl Check for Refused {
    fn check(&self) {
        if self.abi_above {
            let Err(Error::Reach(Unenforced::Abi { found, needs })) = self.result else {
                unexpected("a missing ABI refuses the spawn (io/processes.md, 6)", &self.result)
            };
            assert_eq!(needs, u32::MAX, "the record's ABI is preserved below io's entrance (kernel.md, 6.6)");
            assert!(found < needs, "the kernel names its lower ABI (kernel.md, 6.6)");
        } else {
            assert_eq!(self.result, Err(Error::Reach(self.expected)), "a reach fails whole (kernel.md, 6.6)");
        }
        assert!(
            self.pipes_empty,
            "a refused reach creates no parent descriptors and runs nothing (io/processes.md, 6)"
        );
    }
}

/// Reading, writing, listing and executing beneath the grant; no escape by
/// parent traversal or by a link (io/processes.md, 6).
pub fn granted_and_refused<B: Backend>(backend: &mut B, executable: &[u8]) -> Observed {
    Probe::new(backend, executable).observe(
        &[
            Question::Read(Box::from(b"file".as_slice())),
            Question::Read(Box::from(b"inner/file".as_slice())),
            Question::Write(Box::from(b"file".as_slice())),
            Question::Write(Box::from(b"inner/file".as_slice())),
            Question::List(Box::from(b".".as_slice())),
            Question::List(Box::from(b"inner".as_slice())),
            Question::Execute(Box::from(b"./run".as_slice())),
            Question::Read(Box::from(b"../canary".as_slice())),
            Question::List(Box::from(b"..".as_slice())),
            Question::Read(Box::from(b"out".as_slice())),
        ],
        vec![
            Answer::Granted,
            Answer::Granted,
            Answer::Granted,
            Answer::Granted,
            Answer::Granted,
            Answer::Granted,
            Answer::Granted,
            Answer::Refused(Error::Permission),
            Answer::Refused(Error::Permission),
            Answer::Refused(Error::Permission),
        ],
        "only granted paths are reachable, links included (io/processes.md, 6)",
    )
}

/// `no_new_privs` and execution of a set-id copy keep the probe's ids.
pub fn no_privilege_gained<B: Backend>(backend: &mut B, executable: &[u8]) -> Observed {
    let mut probe = Probe::new(backend, executable);
    let mut questions = vec![Question::NoNewPrivs];
    let mut answers = vec![Answer::NoNewPrivs(true)];
    match probe.run.set_id_copy(probe.process, probe.root, executable, b"granted/set-id") {
        Ok(()) => {
            questions.push(Question::SetId(Box::from(b"./set-id".as_slice())));
            answers.push(Answer::Ids(Ids::Kept));
        }
        Err(reason) => diagnostics(&format!("no_privilege_gained::set_id_copy skipped: {reason}")),
    }
    probe.observe(&questions, answers, "no tree gains privilege by exec (io/processes.md, 6)")
}

/// A signal to the service outside the tree is refused.
pub fn signal_to_parent<B: Backend>(backend: &mut B, executable: &[u8]) -> Observed {
    Probe::new(backend, executable).observe(
        &[Question::SignalParent],
        vec![Answer::Refused(Error::Permission)],
        "signals are scoped to the tree, excluding its parent (io/processes.md, 6)",
    )
}

/// An ABI no kernel has is submitted at record level, not through io.
pub fn missing_abi<B: Backend>(backend: &mut B, executable: &[u8]) -> Refused {
    let mut probe = Probe::new(backend, executable);
    probe.reach.abi = u32::MAX;
    probe.refuse(Unenforced::Abi { found: 0, needs: u32::MAX }, true)
}

/// A missing second rule is named by index, before any command runs.
pub fn missing_rule<B: Backend>(backend: &mut B, executable: &[u8]) -> Refused {
    let mut probe = Probe::new(backend, executable);
    let mut rules = probe.reach.rules.into_vec();
    rules.insert(1, Rule { root: probe.root, path: Box::from(b"missing".as_slice()), rights: Rights::READ });
    probe.reach.rules = rules.into_boxed_slice();
    probe.refuse(Unenforced::Rule { index: 1, errno: 2 }, false)
}

/// The child loses the outer file and writing, while its parent keeps reading.
pub fn child_narrows<B: Backend>(backend: &mut B, executable: &[u8]) -> Observed {
    let probe = Probe::new(backend, executable);
    let mut rules = vec![probe::Rule { path: Box::from(b"inner".as_slice()), rights: Rights::READ }];
    // Machine rules are beneath `/`. Only the real fixture receives them;
    // absolute nested paths are opened beneath its own root on `/`.
    for rule in &probe.machine {
        let mut path = vec![b'/'];
        path.extend_from_slice(&rule.path);
        rules.push(probe::Rule { path: path.into_boxed_slice(), rights: rule.rights });
    }
    probe.observe(
        &[
            Question::Nest {
                rules: rules.into_boxed_slice(),
                questions: Box::new([
                    Question::Read(Box::from(b"file".as_slice())),
                    Question::Write(Box::from(b"inner/file".as_slice())),
                    Question::Read(Box::from(b"inner/file".as_slice())),
                ]),
            },
            Question::Read(Box::from(b"file".as_slice())),
        ],
        vec![
            Answer::Nested(Report {
                answers: Box::new([
                    Answer::Refused(Error::Permission),
                    Answer::Refused(Error::Permission),
                    Answer::Granted,
                ]),
            }),
            Answer::Granted,
        ],
        "nested reaches intersect without narrowing the parent (io/processes.md, 6; testing/README.md, 4)",
    )
}

/// An unnamed, non-close-on-exec canary must not survive the helper.
pub fn descriptors<B: Backend>(backend: &mut B, executable: &[u8]) -> Observed {
    let mut probe = Probe::new(backend, executable);
    let held = probe.run.hold_inheritable(probe.process, probe.root, b"canary");
    probe.held = Some(held);
    probe.extra_pipe = true;
    probe.observe(
        &[Question::Descriptors],
        vec![Answer::Descriptors(Box::new([0, 1, 2, 63]))],
        "only named descriptors survive restriction, not the service's canary (kernel.md, 6.6; io/processes.md, 6)",
    )
}

/// Only the simulator can select a machine without Landlock.
pub fn landlock_absent<B: Backend>(backend: &mut B, executable: &[u8]) -> Refused {
    backend.set_landlock(Landlock::Absent);
    Probe::new(backend, executable).refuse(Unenforced::Absent, false)
}

/// Only the simulator can disable its machine's Landlock.
pub fn landlock_disabled<B: Backend>(backend: &mut B, executable: &[u8]) -> Refused {
    backend.set_landlock(Landlock::Disabled);
    Probe::new(backend, executable).refuse(Unenforced::Disabled, false)
}

/// Only the simulator can select exactly ABI 5 against the record's 6.
pub fn landlock_older_abi<B: Backend>(backend: &mut B, executable: &[u8]) -> Refused {
    backend.set_landlock(Landlock::Abi(5));
    Probe::new(backend, executable).refuse(Unenforced::Abi { found: 5, needs: 6 }, false)
}

/// macOS refuses every reached spawn without running the probe.
pub fn unsupported<B: Backend>(backend: &mut B, executable: &[u8]) -> Refused {
    Probe::new(backend, executable).refuse(Unenforced::Unsupported, false)
}

struct Probe<'a, B: Backend> {
    run: Run<'a, B>,
    process: B::Process,
    root: Fd,
    machine: Vec<Rule>,
    reach: Reach,
    executable: Box<[u8]>,
    held: Option<Fd>,
    extra_pipe: bool,
    mode: Option<&'static [u8]>,
}

impl<'a, B: Backend> Probe<'a, B> {
    fn new(backend: &'a mut B, executable: &[u8]) -> Self {
        let mode = backend.probe_mode();
        let mut run = Run::new(backend);
        let process = run.process();
        let root = run.root(
            process,
            &[
                Item::directory(b"granted"),
                Item::file(b"granted/file", b"file"),
                Item::directory(b"granted/inner"),
                Item::file(b"granted/inner/file", b"inner file"),
                Item::link(b"granted/out", b"../canary"),
                Item::file(b"granted/run", b"#!/usr/bin/true\n").mode(0o755),
                Item::file(b"granted/set-id", b"").mode(0o755),
                Item::file(b"canary", b"not granted"),
            ],
        );
        let machine = run.reach_rules(process, executable);
        let mut rules = vec![Rule { root, path: Box::from(b"granted".as_slice()), rights: Rights::FULL }];
        rules.extend(machine.clone());
        Self {
            run,
            process,
            root,
            machine,
            reach: Reach { abi: Reach::ABI, rules: rules.into_boxed_slice() },
            executable: Box::from(executable),
            held: None,
            extra_pipe: false,
            mode,
        }
    }

    fn spawn(&self, questions: &[Question]) -> skein_io::kernel::Spawn {
        // Name all standard descriptors explicitly so that the simulator and
        // real process observe the same table. The sparse extra exercises gaps
        // below the helper's own descriptors, not only closefrom at spawn.
        let mut pipes = vec![
            Pipe { child: 0, way: Way::In, parent: None },
            Pipe { child: 1, way: Way::Out, parent: None },
            Pipe { child: 2, way: Way::Out, parent: None },
        ];
        if self.extra_pipe {
            pipes.push(Pipe { child: 63, way: Way::Out, parent: None });
        }
        let mut spawn = command(&self.executable, self.root, &[], &pipes);
        spawn.args = self.mode.into_iter().map(Box::from).chain(probe::arguments(questions)).collect();
        spawn.dir = Box::from(b"granted".as_slice());
        spawn.reach = Some(Box::new(self.reach.clone()));
        spawn
    }

    fn observe(mut self, questions: &[Question], expected: Vec<Answer>, rule: &'static str) -> Observed {
        let spawn = self.spawn(questions);
        let (pidfd, pipes) = spawned(self.run.call(self.process, Op::Spawn { spawn: Box::new(spawn) }));
        let [input, output, errors, extra @ ..] = pipes.as_slice() else { unexpected("the probe's pipes", &pipes) };
        self.run.close(self.process, *input);
        let bytes = pipe_all(&mut self.run, self.process, *output);
        let diagnostics = pipe_all(&mut self.run, self.process, *errors);
        if !diagnostics.is_empty() {
            crate::reach::diagnostics(&format!("reach_probe: {}", String::from_utf8_lossy(&diagnostics)));
        }
        self.run.close(self.process, *output);
        self.run.close(self.process, *errors);
        for fd in extra {
            assert!(pipe_all(&mut self.run, self.process, *fd).is_empty(), "the extra named pipe carries no report");
            self.run.close(self.process, *fd);
        }
        let exit = self.run.call(self.process, Op::Wait { pidfd, reap: false }).result;
        assert_eq!(
            self.run.call(self.process, Op::Wait { pidfd, reap: true }).result,
            exit,
            "an observed exit can be reaped (kernel.md, 6.2)"
        );
        self.run.close(self.process, pidfd);
        self.finish();
        Observed {
            report: probe::decode(&bytes).expect("the probe's report (testing/README.md, 4)"),
            expected: Report { answers: expected.into_boxed_slice() },
            rule,
            exit,
        }
    }

    fn refuse(mut self, expected: Unenforced, abi_above: bool) -> Refused {
        let spawn = self.spawn(&[Question::Read(Box::from(b"file".as_slice()))]);
        let complete = self.run.call(self.process, Op::Spawn { spawn: Box::new(spawn) });
        let Op::Spawn { spawn } = complete.kind else { unexpected("the spawn record returned", &complete.kind) };
        let pipes_empty = spawn.pipes.iter().all(|pipe| pipe.parent.is_none());
        self.finish();
        Refused { result: complete.result, expected, abi_above, pipes_empty }
    }

    fn finish(mut self) {
        if let Some(fd) = self.held {
            self.run.close(self.process, fd);
        }
        for root in self.machine.iter().map(|rule| rule.root).collect::<BTreeSet<_>>() {
            self.run.close(self.process, root);
        }
        self.run.close(self.process, self.root);
        self.run.finish();
    }
}

#[expect(
    clippy::print_stderr,
    reason = "the conformance report names skipped set-id parts and the signal the probe tried"
)]
fn diagnostics(text: &str) {
    eprintln!("{text}");
}

fn pipe_all<B: Backend>(run: &mut Run<'_, B>, process: B::Process, fd: Fd) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let complete = run.call(process, Op::PipeRead { fd, buf: vec![0; 4096].into_boxed_slice() });
        let (buf, n) = match (complete.kind, complete.result) {
            (Op::PipeRead { buf, .. }, Ok(Done::Count(n))) => (buf, n),
            (_, other) => unexpected("the probe's pipe read", &other),
        };
        if n == 0 {
            return bytes;
        }
        bytes.extend_from_slice(buf.get(..usize_of(n)).expect("a valid pipe read count"));
    }
}
