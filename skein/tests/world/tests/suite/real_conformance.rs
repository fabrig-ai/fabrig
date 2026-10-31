//! Parent-side conformance and the shared real ring's ownership contracts.

use std::collections::BTreeMap;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use skein_io::kernel::{Complete, Done, Family, Fd, Op, Signal, Spawn, Submit};
use skein_lib::{Duration, Queue, Time, Token, Wall};
use skein_shell::{Clock, open_root};
use skein_world::{Dispatch, Host, HostedProgram, Inherited, Referee, real};
use skein_world_tests::hosted::{Act, Script};

#[cfg(target_os = "linux")]
const DESCRIPTORS: &str = "/proc/self/fd";
#[cfg(target_os = "macos")]
const DESCRIPTORS: &str = "/dev/fd";

#[derive(Debug)]
struct Quiet;

impl<P> Referee<P> for Quiet {
    fn observe(&mut self, _now: Time, _procs: &[P]) {}
    fn passed(&self) -> bool {
        true
    }
}

#[derive(Debug)]
struct RecordingParent {
    script: Script,
    descriptors: BTreeMap<Fd, u32>,
    records: Vec<String>,
}

impl RecordingParent {
    fn new(root: Fd) -> Self {
        Self {
            script: Script::parent(
                root,
                &[Act::Spawn, Act::Write(0, b"hello"), Act::Read(1), Act::Close(0), Act::Read(0), Act::Wait],
            ),
            descriptors: BTreeMap::from([(root, 0)]),
            records: Vec::new(),
        }
    }

    fn record(&mut self, complete: &Complete) {
        if let (Op::Spawn { spawn }, Ok(Done::Spawned { pidfd, .. })) = (&complete.kind, &complete.result) {
            self.descriptors.insert(*pidfd, 1);
            for (at, pipe) in spawn.pipes.iter().enumerate() {
                self.descriptors
                    .insert(pipe.parent.expect("successful parent end"), u32::try_from(at + 2).expect("small table"));
            }
        }
        // Descriptor numbers and sender identities differ between backends.
        // This transcript spawns one child, whose sender has logical identity 1.
        let mut result = complete.result.clone();
        if let Ok(Done::Spawned { sender, .. }) = &mut result {
            *sender = skein_io::kernel::Sender::new(1);
        }
        let mut normalized =
            format!("Complete {{ op: {:?}, kind: {:?}, result: {result:?} }}", complete.op, complete.kind);
        for (fd, logical) in &self.descriptors {
            normalized = normalized.replace(&format!("Fd({})", fd.raw()), &format!("Fd(logical{logical})"));
        }
        self.records.push(normalized);
    }
}

impl Host for RecordingParent {
    fn drain(&mut self) {
        self.script.drain();
    }

    fn iterate(&mut self, now: Time, wall: Wall) {
        let mut arrived = Vec::new();
        while let Some(complete) = self.script.completions().pop() {
            self.record(&complete);
            arrived.push(complete);
        }
        for complete in arrived {
            self.script.completions().push(complete);
        }
        self.script.iterate(now, wall);
        let count = self.script.submissions().len();
        for _ in 0..count {
            let mut record = self.script.submissions().pop().expect("counted record");
            if let Op::Spawn { spawn } = &mut record.kind {
                spawn.program = Box::from(skein_scratch::paths::binary("world_process_fixture").as_os_str().as_bytes());
                spawn.args = Box::new([Box::from(&b"echo"[..]), Box::from(&b"0"[..]), Box::from(&b"1"[..])]);
            }
            self.script.submissions().push(record);
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        self.script.completions()
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        self.script.submissions()
    }
    fn work_pending(&self, now: Time) -> bool {
        self.script.work_pending(now)
    }
    fn next_deadline(&self) -> Option<Time> {
        self.script.next_deadline()
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        self.script.next_policy_deadline()
    }

    fn is_empty(&self) -> bool {
        self.script.is_empty()
    }
    fn operations(&self) -> u32 {
        self.script.operations()
    }
}

#[derive(Debug)]
struct OpeningChild {
    completions: Queue<Complete>,
    submissions: Queue<Submit>,
    started: bool,
}

impl Host for OpeningChild {
    fn iterate(&mut self, _now: Time, _wall: Wall) {
        if !self.started {
            self.submissions.push(Submit { op: Token::new(1), kind: Op::Socket { family: Family::Ipv4 } });
            self.started = true;
        }
    }
    fn completions(&mut self) -> &mut Queue<Complete> {
        &mut self.completions
    }
    fn submissions(&mut self) -> &mut Queue<Submit> {
        &mut self.submissions
    }
    fn work_pending(&self, _now: Time) -> bool {
        !self.started
    }
    fn next_deadline(&self) -> Option<Time> {
        None
    }

    fn next_policy_deadline(&self) -> Option<Time> {
        None
    }

    fn is_empty(&self) -> bool {
        false
    }
    fn operations(&self) -> u32 {
        1
    }
}

#[derive(Debug)]
enum Proc {
    Parent(RecordingParent),
    Script(Script),
    Opening(OpeningChild),
}

impl Dispatch for Proc {
    fn host(&self) -> &dyn Host {
        match self {
            Self::Parent(parent) => parent,
            Self::Script(script) => script,
            Self::Opening(child) => child,
        }
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        match self {
            Self::Parent(parent) => parent,
            Self::Script(script) => script,
            Self::Opening(child) => child,
        }
    }
}

fn echo(spawn: &Spawn, inherited: &Inherited) -> Proc {
    assert_eq!(spawn.args.as_ref(), [Box::from(&b"echo"[..]), Box::from(&b"0"[..]), Box::from(&b"1"[..])]);
    Proc::Script(Script::child(inherited, &[Act::Read(0), Act::Write(1, b"hello"), Act::Read(0)]))
}

fn fixture(hosted: bool) -> RecordingParent {
    let mut world = real::World::new(Quiet);
    if hosted {
        world.host(HostedProgram {
            program: Box::from(skein_scratch::paths::binary("world_process_fixture").as_os_str().as_bytes()),
            make: echo,
            instances: 1,
            operations: 8,
        });
    }
    let root = open_root(Path::new(".")).expect("startup root");
    world.spawn_with_fds(vec![root], || Proc::Parent(RecordingParent::new(root)));
    let outcome = world.run(&Clock::new(), Duration::from_secs(2));
    let Proc::Parent(parent) = outcome.procs.into_iter().next().expect("parent") else {
        panic!("first process is the parent")
    };
    parent
}

#[test]
fn real_conformance_fixture_and_hosted_equivalent_return_the_same_parent_records() {
    let real = fixture(false);
    let hosted = fixture(true);
    assert_eq!(real.script.received, b"hello");
    assert_eq!(hosted.script.received, b"hello");
    assert_eq!(
        real.records, hosted.records,
        "kernel.md record return and io.md section 6: parent cannot distinguish hosting"
    );
}

fn opening(_spawn: &Spawn, _inherited: &Inherited) -> Proc {
    Proc::Opening(OpeningChild {
        completions: Queue::with_capacity(1),
        submissions: Queue::with_capacity(1),
        started: false,
    })
}

#[test]
fn kill_closes_a_descriptor_returned_before_the_child_reaps_it() {
    let before = std::fs::read_dir(DESCRIPTORS).expect("descriptor table").count();
    for _ in 0..16 {
        let mut world = real::World::new(Quiet);
        world.host(HostedProgram { program: Box::from(&b"hosted"[..]), make: opening, instances: 1, operations: 1 });
        let root = open_root(Path::new(".")).expect("startup root");
        world.spawn_with_fds(vec![root], || {
            Proc::Script(Script::parent(root, &[Act::Spawn, Act::Signal(Signal::Kill), Act::Read(1), Act::Wait]))
        });
        let outcome = world.run(&Clock::new(), Duration::from_secs(2));
        assert_eq!(outcome.killed.len(), 1);
        assert_eq!(
            std::fs::read_dir(DESCRIPTORS).expect("descriptor table").count(),
            before,
            "all inherited and completion-returned descriptors are closed after kill"
        );
    }
}
