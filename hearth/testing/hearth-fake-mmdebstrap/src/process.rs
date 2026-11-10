//! Simulate as a process, with the child's files, permissions and pipes (hearth/docs/design/testing/tool-fakes.md, 1).
use crate::{
    apt,
    face::{self, Outcome, Printed},
};
use hearth_fake_machine::{
    Machine,
    hearth::Tool,
    linux::{At, Exit, Handle, How, Op, Pid, Refusal, Reply, Signal},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Faults {
    pub stall_after: Option<u64>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Progress {
    Working,
    Waiting,
    Exited,
}
#[derive(Debug)]
pub struct Mmdebstrap {
    child: Pid,
    printed: Printed,
    offsets: [usize; 2],
    faults: Faults,
    sent: u64,
    exited: bool,
}
impl Mmdebstrap {
    pub fn start(machine: &mut Machine, child: Pid, faults: Faults) -> Self {
        assert_eq!(machine.tool(child), Some(Tool::Mmdebstrap));
        let invocation = machine.linux().invocation(child);
        assert_eq!(invocation.program.as_ref(), b"/usr/bin/mmdebstrap");
        let simulate = face::check(&invocation.args, &invocation.env);
        let target = [simulate.scratch.as_ref(), b"/root"].concat();
        assert_eq!(
            open(machine, child, &target, How::Directory),
            Err(Refusal::NotFound),
            "a new scratch root for each run"
        );
        let text = read(machine, child, &[simulate.scratch.as_ref(), b"/sources.list"].concat());
        let sources = face::sources(&text);
        let scratch = open(machine, child, &simulate.scratch, How::Directory).expect("scratch directory");
        for path in [
            b"root".as_slice(),
            b"root/var",
            b"root/var/lib",
            b"root/var/lib/apt",
            b"root/var/lib/apt/lists",
            b"root/var/lib/apt/lists/partial",
        ] {
            machine
                .call(child, Op::MakeDirectory { dir: scratch, path: path.into(), mode: 0o755 })
                .expect("make apt directories as the child");
        }
        let lists = [target.as_slice(), b"/var/lib/apt/lists/"].concat();
        write(machine, child, &[lists.as_slice(), b"lock"].concat(), b"");
        let (files, outcome) = apt::pass(machine.reach(simulate.proxy), &simulate, &sources);
        if !matches!(outcome, Outcome::Resolved(_)) {
            machine
                .call(
                    child,
                    Op::MakeDirectory {
                        dir: scratch,
                        path: b"root/var/lib/apt/lists/auxfiles".as_slice().into(),
                        mode: 0o755,
                    },
                )
                .expect("apt's failed pass leaves auxfiles");
        }
        machine.call(child, Op::Close { handle: scratch }).expect("close scratch");
        for (name, bytes) in files {
            write(machine, child, &[lists.as_slice(), &name].concat(), &bytes);
        }
        Self {
            child,
            printed: face::print(&simulate, &sources, &outcome),
            offsets: [0, 0],
            faults,
            sent: 0,
            exited: false,
        }
    }
    fn exit(&mut self, machine: &mut Machine, exit: Exit) -> Progress {
        machine.call(self.child, Op::Exit { exit }).expect("child exits");
        self.exited = true;
        Progress::Exited
    }
    pub fn step(&mut self, machine: &mut Machine) -> Progress {
        if self.exited {
            return Progress::Exited;
        }
        if machine.linux().pending(self.child) == Some(Signal::Terminate) {
            return self.exit(machine, Exit::Signal(15));
        }
        for place in 0..2 {
            let bytes = if place == 0 { &self.printed.stdout } else { &self.printed.stderr };
            let offset = &mut self.offsets[place];
            if *offset == bytes.len() {
                continue;
            }
            let room = self
                .faults
                .stall_after
                .map_or(usize::MAX, |limit| usize::try_from(limit.saturating_sub(self.sent)).unwrap_or(usize::MAX));
            if room == 0 {
                return Progress::Waiting;
            }
            let end = bytes.len().min(offset.saturating_add(room));
            let pipe = machine.linux().descriptor(self.child, u32::try_from(place + 1).expect("stdout or stderr"));
            match machine.call(self.child, Op::Send { pipe, bytes: bytes[*offset..end].into() }) {
                Ok(Reply::Sent(count)) => {
                    *offset += usize::try_from(count).expect("pipe count fits");
                    self.sent += u64::from(count);
                    return if count == 0 { Progress::Waiting } else { Progress::Working };
                }
                Err(Refusal::BrokenPipe) => return self.exit(machine, Exit::Signal(13)),
                reply => panic!("pipe send answers bytes or EPIPE: {reply:?}"),
            }
        }
        self.exit(machine, Exit::Code(self.printed.exit))
    }
}
fn open(machine: &mut Machine, child: Pid, path: &[u8], how: How) -> Result<Handle, Refusal> {
    let Reply::Opened(handle) = machine.call(child, Op::Open { at: At::Root, path: path.into(), how })? else {
        panic!("open answers a handle");
    };
    Ok(handle)
}
fn read(machine: &mut Machine, child: Pid, path: &[u8]) -> Box<[u8]> {
    let file = open(machine, child, path, How::Read).expect("source list readable by child");
    let mut bytes = Vec::new();
    loop {
        let Reply::Read(part) = machine
            .call(child, Op::Read { file, at: u64::try_from(bytes.len()).expect("source length fits"), most: 4096 })
            .expect("read sources")
        else {
            panic!("read answers bytes");
        };
        if part.is_empty() {
            break;
        }
        bytes.extend_from_slice(&part);
    }
    machine.call(child, Op::Close { handle: file }).expect("close sources");
    bytes.into()
}
fn write(machine: &mut Machine, child: Pid, path: &[u8], bytes: &[u8]) {
    let file = open(machine, child, path, How::Create { mode: None }).expect("create apt list as child");
    machine.call(child, Op::Write { file, at: 0, bytes: bytes.into() }).expect("write apt list");
    machine.call(child, Op::Close { handle: file }).expect("close apt list");
}
