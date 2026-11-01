//! Terminal-marked whole-tree cuts (temper/docs/design/draft/playground-reset.md, 8).
use super::{Ended, Process, boxed, ctl, patience::Patience};
use skein_io::kernel::{Exit, Signal};
use skein_lib::{Duration, Time};
use skein_shell::{Clock, Host};
use skein_world::{
    Referee,
    end_to_end::{Binary, Command, Expectation, Mode},
    real,
};
use std::path::Path;

pub struct Cut {
    pub line: String,
    pub calls: Vec<Ended>,
    pub stderr: Vec<u8>,
}

struct Judge<'a> {
    marker: &'a dyn Fn(&str) -> bool,
    scanned: usize,
    line: &'a std::cell::Cell<Option<String>>,
    cut: bool,
    deadline: Patience,
    settled: bool,
}

impl Referee<Process> for Judge<'_> {
    fn act(&mut self, now: Time, procs: &mut [Process]) {
        let [Process::Binary(binary), Process::Calls(calls)] = procs else { panic!("box and ctl") };
        if !self.cut {
            // Binary drains stderr on this same ring. Inspect complete lines in
            // their order and kill on the first matching line, before more calls.
            let stderr = binary.stderr();
            while let Some(end) = stderr[self.scanned..].iter().position(|byte| *byte == b'\n') {
                let end = self.scanned + end + 1;
                let line = std::str::from_utf8(&stderr[self.scanned..end - 1]).expect("terminal UTF-8");
                self.scanned = end;
                if (self.marker)(line) {
                    self.line.set(Some(line.into()));
                    self.cut = true;
                    binary.signal_tree(Signal::Kill);
                    calls.release();
                    calls.cancel();
                    return;
                }
            }
            calls.observe_box_stderr(binary.stderr(), now);
            for (index, call) in calls.ends().iter().enumerate() {
                if calls.observed_at()[index].is_some() {
                    assert_eq!(
                        call.exit,
                        Some(Exit::Code(0)),
                        "ctl before cut: {}",
                        String::from_utf8_lossy(&call.stderr)
                    );
                }
            }
            assert!(binary.exit_status().is_none(), "box ended before cut: {}", String::from_utf8_lossy(stderr));
        }
    }

    fn observe(&mut self, _now: Time, procs: &[Process]) {
        self.settled = self.cut && procs.iter().all(Host::is_empty);
        if !self.settled
            && let Some(figures) = self.deadline.failed()
        {
            let Process::Binary(binary) = &procs[0] else { panic!("box") };
            panic!("terminal cut did not settle: {}; {figures}", String::from_utf8_lossy(binary.stderr()));
        }
    }

    fn next_deadline(&self) -> Option<Time> {
        if self.settled { None } else { self.deadline.until() }
    }

    fn passed(&self) -> bool {
        self.settled
    }
}

/// Start the shipped box with echoed steps and drain its entire tree after SIGKILL.
#[must_use]
pub fn at_line(
    directory: &Path,
    flags: Vec<String>,
    client: &Path,
    steps: Vec<ctl::Step>,
    marker: impl Fn(&str) -> bool,
) -> Cut {
    let clock = Clock::new();
    let mut binary = Binary::start_with_tree(
        boxed::dying_with_test(Command {
            program: skein_scratch::paths::binary("temper"),
            arguments: std::iter::once("box".into())
                .chain(boxed::echo_steps(flags).into_iter().map(Into::into))
                .collect(),
            directory: directory.into(),
            environment: Vec::new(),
        }),
        Mode::Pipes,
        262_144,
        Expectation::Supervise,
        false,
    )
    .expect("box starts for terminal cut");
    let streams = binary.take_streams().expect("box streams");
    let mut fds = streams.descriptors();
    let root = skein_shell::open_root(client).expect("client root");
    fds.push(root);
    let line = std::cell::Cell::new(None);
    let mut world = real::World::new(Judge {
        marker: &marker,
        scanned: 0,
        line: &line,
        cut: false,
        deadline: Patience::watching(
            "the box's terminal cut and drain",
            Duration::from_nanos(boxed::BOX_DEADLINE.as_nanos() - Duration::from_secs(10).as_nanos()),
            &binary,
        ),
        settled: false,
    });
    world.spawn_with_fds(binary.descriptors(), || Process::Binary(Box::new(binary)));
    world.spawn_with_fds(fds, || {
        Process::Calls(Box::new(ctl::Calls::new(root, steps, Some(&streams), None).with_box_stderr()))
    });
    let ended = world.run(&clock, boxed::BOX_DEADLINE);
    assert!(ended.procs.iter().all(Host::is_empty), "cut box and ctl drained");
    let [Process::Binary(binary), Process::Calls(calls)] = ended.procs.as_slice() else { panic!("box and ctl") };
    assert!(binary.counts().is_some(), "the killed process tree settled before restart");
    assert!(!binary.stderr_overflowed(), "terminal cut retained every line");
    Cut {
        line: line.take().expect("terminal cut reached"),
        calls: calls
            .ends()
            .iter()
            .map(|call| Ended { exit: call.exit, stdout: call.stdout.clone(), stderr: call.stderr.clone() })
            .collect(),
        stderr: binary.stderr().to_vec(),
    }
}
