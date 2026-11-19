//! Questions answered from inside the Landlock domain (io/processes.md, 6).

use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::fd::IntoRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use skein_fake_machine::probe::{self, Answer, Ids, Question, Report};
use skein_io::kernel::{Complete, Done, Error, Exit, Fd, Op, Pipe, Reach, Rule, Spawn, Submit, Way};
use skein_lib::{Duration, Queue, Token};
use skein_shell::{Backend, Config, Kernel, Wait};

pub(super) fn run(args: impl Iterator<Item = String>) {
    let args: Vec<Box<[u8]>> = args.map(|arg| Box::from(arg.into_bytes())).collect();
    if args.as_slice() == [Box::from(b"--ids".as_slice())] {
        std::io::stdout().write_all(&ids()).expect("the set-id copy's ids");
        return;
    }
    let questions = probe::questions(&args).expect("the shared probe's arguments");
    let report = Report { answers: questions.iter().map(answer).collect() };
    std::io::stdout().write_all(&probe::encode(&report)).expect("the probe's report pipe");
}

fn answer(question: &Question) -> Answer {
    match question {
        Question::Read(path) => access(File::open(OsStr::from_bytes(path)).map(drop)),
        Question::Write(path) => access(OpenOptions::new().write(true).open(OsStr::from_bytes(path)).map(drop)),
        Question::List(path) => access(fs::read_dir(OsStr::from_bytes(path)).map(drop)),
        Question::Execute(path) => execute(path, false),
        Question::SetId(path) => execute(path, true),
        Question::NoNewPrivs => {
            let status = fs::read_to_string("/proc/self/status").expect("the probe's no_new_privs");
            let bit =
                status.lines().find_map(|line| line.strip_prefix("NoNewPrivs:")).expect("NoNewPrivs in proc status");
            Answer::NoNewPrivs(bit.trim() == "1")
        }
        Question::SignalParent => {
            let (signal, result) = skein_fork::signal_parent();
            eprintln!("signal_to_parent: sent signal {signal} (0 first; SIGURG if 0 bypasses scope)");
            match result {
                Ok(()) => Answer::Granted,
                Err(errno) => Answer::Refused(error(errno)),
            }
        }
        Question::Descriptors => descriptors(),
        Question::Nest { rules, questions } => nested(rules, questions),
    }
}

fn error(errno: i32) -> Error {
    match errno {
        libc::EACCES | libc::EPERM => Error::Permission,
        libc::ENOENT => Error::NotFound,
        other => Error::Other(other),
    }
}

fn access(result: std::io::Result<()>) -> Answer {
    match result {
        Ok(()) => Answer::Granted,
        Err(failed) => Answer::Refused(error(failed.raw_os_error().expect("a syscall errno"))),
    }
}

fn execute(path: &[u8], set_id: bool) -> Answer {
    let mut command = Command::new(OsStr::from_bytes(path));
    if set_id {
        command.args(["reach-probe", "--ids"]);
    }
    // Do not open /dev/null: it is deliberately not in the probe's reach.
    let output = match command.stdin(Stdio::inherit()).stderr(Stdio::inherit()).output() {
        Ok(output) => output,
        Err(failed) => return Answer::Refused(error(failed.raw_os_error().expect("exec errno"))),
    };
    assert!(output.status.success(), "the executable probe question exited: {:?}", output.status);
    if set_id { Answer::Ids(if output.stdout == ids() { Ids::Kept } else { Ids::Gained }) } else { Answer::Granted }
}

fn ids() -> Vec<u8> {
    let status = fs::read_to_string("/proc/self/status").expect("the probe's ids");
    let mut ids = Vec::new();
    for label in ["Uid:", "Gid:"] {
        let row = status.lines().find_map(|line| line.strip_prefix(label)).expect("the process ids");
        ids.extend_from_slice(row.trim().as_bytes());
        ids.push(b'\n');
    }
    ids
}

fn descriptors() -> Answer {
    let directory = PathBuf::from(format!("/proc/{}/fd", std::process::id()));
    let mut descriptors = Vec::new();
    for entry in fs::read_dir("/proc/self/fd").expect("the probe's descriptors") {
        let entry = entry.expect("a descriptor entry");
        // The directory iterator is opened by this observation, not inherited.
        if fs::read_link(entry.path()).expect("descriptor target") != directory {
            descriptors.push(
                entry.file_name().to_str().expect("a decimal descriptor").parse::<u32>().expect("descriptor number"),
            );
        }
    }
    descriptors.sort_unstable();
    Answer::Descriptors(descriptors.into_boxed_slice())
}

fn call(kernel: &mut Kernel, next: &mut u64, kind: Op) -> Complete {
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit { op: Token::new(*next), kind });
    *next = next.checked_add(1).expect("bounded nested operations");
    let mut completions = Queue::with_capacity(1);
    let until = skein_shell::Clock::new().now().now.saturating_add(Duration::from_secs(20));
    loop {
        kernel.submit(&mut submissions, Wait::Until(until));
        kernel.reap(&mut completions);
        if let Some(complete) = completions.pop() {
            assert!(complete.is_valid(), "the nested kernel's completion contract");
            return complete;
        }
        assert!(skein_shell::Clock::new().now().now < until, "the nested probe operation completes");
    }
}

fn close(kernel: &mut Kernel, next: &mut u64, fd: Fd) {
    assert_eq!(call(kernel, next, Op::Close { fd }).result, Ok(Done::Nothing), "nested descriptor closed");
}

fn read_pipe(kernel: &mut Kernel, next: &mut u64, fd: Fd) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let complete = call(kernel, next, Op::PipeRead { fd, buf: vec![0; 4096].into_boxed_slice() });
        let (buf, n) = match (complete.kind, complete.result) {
            (Op::PipeRead { buf, .. }, Ok(Done::Count(n))) => (buf, n),
            (_, other) => panic!("nested pipe read: {other:?}"),
        };
        if n == 0 {
            return bytes;
        }
        bytes.extend_from_slice(&buf[..usize::try_from(n).expect("a pipe read fits usize")]);
    }
}

fn nested(rules: &[probe::Rule], questions: &[Question]) -> Answer {
    let executable = std::env::current_exe().expect("the nested probe executable");
    let mut kernel = Kernel::open_on(Config { operations: 4 }, Backend::Readiness).expect("the nested probe's kernel");
    kernel.set_helper(Box::from(executable.as_os_str().as_bytes()));
    let root = skein_shell::open_root(Path::new(".")).expect("the probe's working directory");
    // Opening a directory to read would ask to list `/`, which the outer
    // reach deliberately refuses. O_PATH holds only the rule resolution root.
    let machine = File::options().read(true).custom_flags(libc::O_PATH).open("/").expect("the nested machine root");
    let machine = Fd::new(machine.into_raw_fd());
    let rules = rules
        .iter()
        .map(|rule| {
            let (root, path) = if let Some(relative) = rule.path.strip_prefix(b"/") {
                (machine, relative)
            } else {
                (root, rule.path.as_ref())
            };
            Rule { root, path: Box::from(path), rights: rule.rights }
        })
        .collect();
    let spawn = Spawn {
        program: Box::from(executable.as_os_str().as_bytes()),
        args: std::iter::once(Box::from(b"reach-probe".as_slice())).chain(probe::arguments(questions)).collect(),
        env: Box::new([]),
        root,
        dir: Box::new([]),
        pipes: Box::new([
            Pipe { child: 0, way: Way::In, parent: None },
            Pipe { child: 1, way: Way::Out, parent: None },
            Pipe { child: 2, way: Way::Out, parent: None },
        ]),
        reach: Some(Box::new(Reach { abi: Reach::ABI, rules })),
    };
    let mut next = 1;
    let complete = call(&mut kernel, &mut next, Op::Spawn { spawn: Box::new(spawn) });
    let result = if let Ok(Done::Spawned { pidfd, .. }) = complete.result {
        let Op::Spawn { spawn } = complete.kind else { panic!("the nested spawn record") };
        let pipes: Vec<_> = spawn.pipes.iter().map(|pipe| pipe.parent.expect("the nested parent pipe")).collect();
        let [input, output, errors] = pipes.as_slice() else { panic!("the nested standard descriptors") };
        close(&mut kernel, &mut next, *input);
        let bytes = read_pipe(&mut kernel, &mut next, *output);
        let diagnostics = read_pipe(&mut kernel, &mut next, *errors);
        std::io::stderr().write_all(&diagnostics).expect("nested diagnostics relayed");
        assert_eq!(
            call(&mut kernel, &mut next, Op::Wait { pidfd, reap: true }).result,
            Ok(Done::Exit(Exit::Code(0))),
            "the nested probe exits"
        );
        for fd in [*output, *errors, pidfd] {
            close(&mut kernel, &mut next, fd);
        }
        Answer::Nested(probe::decode(&bytes).expect("the nested report"))
    } else {
        Answer::Refused(complete.result.expect_err("a failed nested spawn"))
    };
    close(&mut kernel, &mut next, machine);
    close(&mut kernel, &mut next, root);
    result
}
