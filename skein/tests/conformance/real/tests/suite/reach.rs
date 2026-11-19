//! One reached spawn per proof, on each Linux backend (runtime/kernel.md, 6.6).
#![cfg(target_os = "linux")]

use std::fs::File;
use std::os::fd::AsRawFd;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use skein_conformance::Backend as _;
use skein_conformance_real::Real;
use skein_io::kernel::{
    Complete, Done, Error, Exit, Fd, Op, Pipe, Reach, Rights, Rule, Spawn, Submit, Unenforced, Way,
};
use skein_lib::{Duration, Queue, Token};
use skein_shell::{Backend, Config, Kernel};

fn fixture(name: &str) -> Box<[u8]> {
    Box::from(skein_scratch::paths::binary(name).as_os_str().as_bytes())
}

fn ready(backend: Backend) -> (Real, usize, File) {
    let mut real = Real::new(backend);
    let process = real.open();
    real.set_helper(process, fixture("process_fixture"));
    (real, process, File::open("/").expect("the host root"))
}

fn command(root: &File, rules: Box<[Rule]>) -> Spawn {
    Spawn {
        program: fixture("process_fixture"),
        args: Box::new([Box::from(b"exit".as_slice()), Box::from(b"7".as_slice())]),
        env: Box::new([]),
        root: Fd::new(root.as_raw_fd()),
        dir: Box::new([]),
        pipes: Box::new([Pipe { child: 1, way: Way::Out, parent: None }]),
        reach: Some(Box::new(Reach { abi: Reach::ABI, rules })),
    }
}

fn call(real: &mut Real, process: usize, kind: Op) -> Complete {
    let mut submissions = Queue::with_capacity(1);
    submissions.push(Submit { op: Token::new(1), kind });
    real.submit(process, &mut submissions);
    let until = real.now().saturating_add(Duration::from_secs(3));
    real.wait_begins("the reach child settling", Duration::from_secs(3));
    let mut completions = Queue::with_capacity(1);
    loop {
        real.enter(process);
        real.reap(process, &mut completions);
        if let Some(complete) = completions.pop() {
            assert!(complete.is_valid(), "the reach backend keeps its record contract: {complete:?}");
            return complete;
        }
        if real.now() >= until {
            real.wait_unanswered();
            panic!("the reach child did not settle: {:?}", real.wait_figures());
        }
        real.pass(process, Duration::from_millis(5));
    }
}

fn children() -> Vec<u32> {
    // Rust's harness runs the test on a thread other than the thread-group
    // leader. Linux lists children on the task that spawned them.
    let mut children = Vec::new();
    for task in std::fs::read_dir("/proc/self/task").expect("our tasks") {
        let path = task.expect("task entry").path().join("children");
        let text = std::fs::read_to_string(path).expect("our task's child list");
        children.extend(text.split_whitespace().map(|pid| pid.parse::<u32>().expect("child PID")));
    }
    children.sort_unstable();
    children
}

fn startup(backend: Backend) {
    let mut kernel = Kernel::open_on(Config { operations: 2 }, backend).expect("the forced backend");
    kernel.set_helper(fixture("process_fixture"));
    if let Err(reason) = kernel.can_restrict() {
        panic!("{reason}");
    }
    assert!(children().is_empty(), "the startup helper was reaped");
}

fn allowed(backend: Backend) {
    let (mut real, process, root) = ready(backend);
    let executable = skein_scratch::paths::binary("process_fixture");
    // Resolve host library links before making rules: absolute symlinks cannot
    // be followed by RESOLVE_BENEATH. Grant no other part of the host's tree.
    let mut paths = vec![executable.parent().expect("fixture directory").to_path_buf()];
    for library in ["/lib", "/lib64", "/usr/lib"] {
        if Path::new(library).exists() {
            paths.push(std::fs::canonicalize(library).expect("library directory"));
        }
    }
    paths.sort();
    paths.dedup();
    let rules = paths
        .into_iter()
        .map(|path| Rule {
            root: Fd::new(root.as_raw_fd()),
            path: Box::from(path.strip_prefix("/").expect("absolute grant").as_os_str().as_bytes()),
            rights: Rights::READ_EXECUTE,
        })
        .collect();
    let complete = call(&mut real, process, Op::Spawn { spawn: Box::new(command(&root, rules)) });
    let Ok(Done::Spawned { pidfd, .. }) = complete.result else { panic!("reached spawn failed: {complete:?}") };
    assert_eq!(call(&mut real, process, Op::Wait { pidfd, reap: true }).result, Ok(Done::Exit(Exit::Code(7))));
    let Op::Spawn { spawn } = complete.kind else { panic!("spawn record") };
    for fd in std::iter::once(pidfd).chain(spawn.pipes.iter().filter_map(|pipe| pipe.parent)) {
        assert_eq!(call(&mut real, process, Op::Close { fd }).result, Ok(Done::Nothing));
    }
    real.assert_settled(process);
}

fn refused(backend: Backend, rules: Box<[Rule]>, helper: &str, expected: Error) {
    let (mut real, process, root) = ready(backend);
    real.set_helper(process, fixture(helper));
    let before = children();
    let complete = call(&mut real, process, Op::Spawn { spawn: Box::new(command(&root, rules)) });
    assert_eq!(complete.result, Err(expected), "reached spawn refusal");
    let Op::Spawn { spawn } = complete.kind else { panic!("spawn record") };
    assert!(spawn.pipes.iter().all(|pipe| pipe.parent.is_none()), "failed spawn leaves no parent pipe");
    assert_eq!(children(), before, "no unreaped helper remains");
    real.assert_settled(process);
}

fn missing_rule(backend: Backend) {
    let scratch = skein_scratch::Scratch::new("reach-rule");
    let root = File::open(scratch.path()).expect("rule root");
    refused(
        backend,
        Box::new([
            Rule { root: Fd::new(root.as_raw_fd()), path: Box::new([]), rights: Rights::READ },
            Rule { root: Fd::new(root.as_raw_fd()), path: Box::from(b"missing".as_slice()), rights: Rights::READ },
        ]),
        "process_fixture",
        Error::Reach(Unenforced::Rule { index: 1, errno: libc::ENOENT }),
    );
}

#[test]
fn startup_restricts_on_ring() {
    startup(Backend::Ring);
}
#[test]
fn startup_restricts_on_readiness() {
    startup(Backend::Readiness);
}
#[test]
fn granted_command_exits_seven_on_ring() {
    allowed(Backend::Ring);
}
#[test]
fn granted_command_exits_seven_on_readiness() {
    allowed(Backend::Readiness);
}
#[test]
fn empty_reach_refuses_exec_and_reaps_on_ring() {
    refused(Backend::Ring, Box::new([]), "process_fixture", Error::Permission);
}
#[test]
fn empty_reach_refuses_exec_and_reaps_on_readiness() {
    refused(Backend::Readiness, Box::new([]), "process_fixture", Error::Permission);
}
#[test]
fn missing_rule_names_its_index_on_ring() {
    missing_rule(Backend::Ring);
}
#[test]
fn missing_rule_names_its_index_on_readiness() {
    missing_rule(Backend::Readiness);
}
#[test]
fn absent_dispatch_is_refused_and_reaped_on_ring() {
    refused(Backend::Ring, Box::new([]), "inherit_fixture", Error::Reach(Unenforced::Helper));
}
#[test]
fn absent_dispatch_is_refused_and_reaped_on_readiness() {
    refused(Backend::Readiness, Box::new([]), "inherit_fixture", Error::Reach(Unenforced::Helper));
}
