//! The box as its parent's child (temper/docs/design/protocol/setup/box.md, 6).
#[expect(dead_code, reason = "the shared box fixture also supports ordinary pipe stories")]
#[expect(clippy::duplicate_mod, reason = "the front and listener stories share sign-in helpers")]
#[path = "../fake_backends/box_link.rs"]
mod box_link;
use crate::support::{self, patience::Patience};
use skein_io::kernel::{Complete, Done, Exit, Fd, Op, Pipe, Signal, Spawn, Submit, Target, Way};
use skein_lib::{Queue, Token};
use skein_shell::{Config, Kernel, Wait, keeper};
use std::{collections::BTreeMap, fs, os::unix::ffi::OsStrExt, path::Path};

const OPERATIONS: u32 = 12;
const OUTPUT_BYTES: usize = 262_144;
struct Pipes {
    kernel: Kernel,
    pidfd: Fd,
    input: Option<Fd>,
    outputs: [Fd; 4],
    reading: [bool; 4],
    ended: [bool; 4],
    bytes: [Vec<u8>; 4],
    complete: Queue<Complete>,
    submits: Queue<Submit>,
}
impl Pipes {
    fn start(root: &Path, arguments: &[&str], closed: Option<u32>) -> Self {
        let mut command = box_link::command(root, "");
        command.args(arguments);
        let mut environment: BTreeMap<_, _> = temper_test_env::environment()
            .into_vec()
            .into_iter()
            .map(|(name, value)| {
                (std::ffi::OsStr::from_bytes(&name).to_os_string(), std::ffi::OsStr::from_bytes(&value).to_os_string())
            })
            .collect();
        for (name, value) in command.get_envs() {
            if let Some(value) = value {
                environment.insert(name.to_owned(), value.to_owned());
            } else {
                environment.remove(name);
            }
        }
        let script = closed.map_or_else(|| "exec \"$@\"".to_owned(), |fd| format!("exec {fd}>&-; exec \"$@\""));
        let mut arguments = vec![
            b"-c".as_slice().into(),
            script.into_bytes().into(),
            b"box-front".as_slice().into(),
            command.get_program().as_bytes().into(),
        ];
        arguments.extend(command.get_args().map(|word| Box::from(word.as_bytes())));
        let root_fd = skein_shell::open_root(root).expect("scratch root");
        let mut spawn = Spawn {
            program: b"/bin/sh".as_slice().into(),
            args: arguments.into_boxed_slice(),
            env: environment
                .into_iter()
                .map(|(name, value)| {
                    let mut bytes = name.as_bytes().to_vec();
                    bytes.push(b'=');
                    bytes.extend_from_slice(value.as_bytes());
                    bytes.into_boxed_slice()
                })
                .collect(),
            root: root_fd,
            dir: b".".as_slice().into(),
            pipes: (0..5)
                .map(|child| Pipe { child, way: if child == 0 { Way::In } else { Way::Out }, parent: None })
                .collect(),
        };
        let started = keeper::start_binary(&mut spawn, false);
        skein_shell::close_startup_fd(root_fd);
        let (pidfd, terminal) = started.expect("start box on its parent's pipes");
        assert!(terminal.is_none());
        Self {
            kernel: Kernel::open(Config { operations: OPERATIONS }).expect("pipes kernel"),
            pidfd,
            input: spawn.pipes[0].parent,
            outputs: std::array::from_fn(|index| spawn.pipes[index + 1].parent.expect("output pipe")),
            reading: [false; 4],
            ended: [false; 4],
            bytes: std::array::from_fn(|_| Vec::new()),
            complete: Queue::with_capacity(OPERATIONS),
            submits: Queue::with_capacity(OPERATIONS),
        }
    }
    fn tick(&mut self) -> Option<Exit> {
        self.kernel.submit(&mut self.submits, Wait::No);
        self.kernel.reap(&mut self.complete);
        while let Some(done) = self.complete.pop() {
            let index = usize::try_from(done.op.raw() - 1).expect("read index");
            self.reading[index] = false;
            let Op::PipeRead { buf, .. } = done.kind else { panic!("pipe read") };
            let Done::Count(count) = done.result.expect("parent's pipe read") else { panic!("read count") };
            self.bytes[index].extend_from_slice(&buf[..usize::try_from(count).expect("count")]);
            assert!(self.bytes[index].len() <= OUTPUT_BYTES, "bounded box output");
            self.ended[index] = count == 0;
        }
        for (index, fd) in self.outputs.iter().enumerate() {
            if !self.reading[index] && !self.ended[index] {
                self.submits.push(Submit {
                    op: Token::new(u64::try_from(index + 1).expect("read token")),
                    kind: Op::PipeRead { fd: *fd, buf: vec![0; 8192].into() },
                });
                self.reading[index] = true;
            }
        }
        self.kernel.submit(&mut self.submits, Wait::No);
        keeper::poll_child(self.pidfd, false).expect("box exit observation")
    }
    fn wait(&mut self, what: &'static str, ready: impl Fn(&Self, Option<Exit>) -> bool) -> Option<Exit> {
        let mut patience = Patience::arm(what, support::WITHIN);
        patience.watch_process(keeper::child_id(self.pidfd).expect("box PID"));
        loop {
            let exit = self.tick();
            if ready(self, exit) {
                return exit;
            }
            assert!(exit.is_none() || !self.ended.iter().all(|ended| *ended), "box exited before {what}");
            if let Some(figures) = patience.failed() {
                panic!("waiting for {what}: {figures}");
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
    fn stop(&mut self) -> Exit {
        if let Some(input) = self.input.take() {
            keeper::close_keeper_fd(input);
        }
        self.wait("box exits", |pipes, exit| exit.is_some() && pipes.ended.iter().all(|ended| *ended))
            .expect("box exit")
    }
}
impl Drop for Pipes {
    fn drop(&mut self) {
        let _signalled = keeper::signal_kept_child(self.pidfd, Signal::Kill, Target::Group);
        for (index, reading) in self.reading.iter().enumerate() {
            if *reading {
                self.submits.push(Submit {
                    op: Token::new(u64::try_from(index + 10).expect("cancel token")),
                    kind: Op::Cancel { target: Token::new(u64::try_from(index + 1).expect("read token")) },
                });
            }
        }
        self.kernel.submit(&mut self.submits, Wait::No);
        while self.kernel.in_flight() != 0 {
            self.kernel.reap(&mut self.complete);
            self.complete.clear();
            if self.kernel.in_flight() != 0 {
                self.kernel.submit(&mut self.submits, Wait::Forever);
            }
        }
        let mut fds = self.outputs.to_vec();
        fds.extend(self.input.take());
        fds.push(self.pidfd);
        keeper::abandon_binary(Some(self.pidfd), &fds);
    }
}

#[test]
fn a_fresh_front_hands_a_local_sign_in_and_stops_at_its_input_end() {
    let scratch = support::Scratch::startup("box-front");
    let root = scratch.path();
    let mut pipes = Pipes::start(root, &["--front"], None);
    pipes.wait("local link's end", |pipes, _| pipes.ended[2]);
    let local = pipes.bytes[2].clone();
    assert!(!local.is_empty() && local.len() <= 4096 && !local.contains(&b'\n'), "fd 3 is the bare URL");
    assert!(box_link::signs_in(root, "front-person", &local), "the local link signs in");
    pipes.wait("printed link line", |pipes, _| pipes.bytes[3].contains(&b'\n'));
    let first = pipes.bytes[3].split_inclusive(|byte| *byte == b'\n').next().expect("first line");
    let printed =
        first.strip_prefix(b"link ").and_then(|url| url.strip_suffix(b"\n")).expect("link URL is first").to_vec();
    assert!(printed != local, "browser and local links differ");
    assert_eq!(pipes.stop(), Exit::Code(0));
    assert!(pipes.bytes[0].is_empty() && pipes.bytes[1].is_empty(), "nothing on the box's terminal");
    for entry in fs::read_dir(root.join("data/box/logs")).expect("box logs") {
        let entry = entry.expect("log entry");
        assert_ne!(entry.file_name(), "tui.log");
        let bytes = fs::read(entry.path()).expect("log bytes");
        for link in [&pipes.bytes[2], &printed] {
            assert!(skein_lib::bytes::find(&bytes, link).is_none(), "logs hold neither link");
        }
        assert!(skein_lib::bytes::find(&bytes, b"secret=").is_none(), "no fresh link in a log");
    }
    drop(pipes);
    scratch.remove();
}

#[test]
fn front_refuses_without_a_first_link_before_starting_steps() {
    let scratch = support::Scratch::startup("box-front-no-link");
    let initialized = box_link::command(scratch.path(), "init").arg("--box").output().expect("init box");
    assert!(initialized.status.success());
    let path = scratch.path().join("config/temper.json");
    let mut document: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("configuration")).expect("document");
    document["engine"]["clients"]["link"] = false.into();
    fs::write(&path, serde_json::to_vec(&document).expect("document bytes")).expect("disabled link");
    let mut pipes = Pipes::start(scratch.path(), &["--front"], None);
    assert_eq!(pipes.stop(), Exit::Code(1));
    assert_eq!(pipes.bytes[1], b"temper-box: --front: no first sign-in link drawn\n");
    assert!(pipes.bytes[0].is_empty() && pipes.bytes[2].is_empty() && pipes.bytes[3].is_empty());
    assert!(!scratch.path().join("data/box/logs").exists(), "no host or steps started");
    drop(pipes);
    scratch.remove();
}

#[test]
fn front_refuses_a_missing_parent_pipe_before_any_start() {
    for closed in [3, 4] {
        let scratch = support::Scratch::startup("box-front-missing");
        let mut pipes = Pipes::start(scratch.path(), &["--front"], Some(closed));
        assert_eq!(pipes.stop(), Exit::Code(2));
        assert!(pipes.bytes[0].is_empty() && pipes.bytes[2].is_empty() && pipes.bytes[3].is_empty());
        assert_eq!(pipes.bytes[1], b"temper-box --front: fd 3 and fd 4 are its parent's pipes\n");
        assert!(!scratch.path().join("config").exists(), "no initialization before descriptor refusal");
        drop(pipes);
        scratch.remove();
    }
}
