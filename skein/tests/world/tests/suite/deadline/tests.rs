use super::{Deadline, Duration, Expired};
use skein_world::testing::deadline::{Watch, figure, has_boot_origin, has_cgroup, thread_baseline, watches};

#[test]
fn a_binary_watch_chooses_the_keepers_cgroup_else_owned_threads() {
    use skein_world::end_to_end::{Binary, Command, Mode};
    let command = Command {
        program: "/bin/sh".into(),
        arguments: vec!["-c".into(), "read answer".into()],
        environment: vec![],
        directory: "/tmp".into(),
    };
    let binary = Binary::start(command, Mode::Pipes, 1024).expect("start a waiting owned binary");
    let has_cgroup = has_cgroup(&binary);
    let mut deadline = Deadline::arm("the accounting observation", Duration::from_millis(20));
    deadline.watch(&binary);
    let (is_cgroup, source) = match watches(&deadline).first().expect("a watch adds a source") {
        Watch::Cgroup { start } => {
            (true, format!("cgroup pressure baseline={}", figure(start.map(Duration::from_nanos))))
        }
        Watch::Threads(threads) => (false, format!("cgroup absent; {threads} readable owned threads")),
    };
    assert_eq!(is_cgroup, has_cgroup);
    deadline.watch_process(std::process::id());
    deadline.watch(&binary);
    assert_eq!(watches(&deadline).count(), 3, "binary and process watches accumulate");
    std::thread::sleep(core::time::Duration::from_nanos(deadline.remaining().as_nanos()));
    let expired = deadline.expired();
    drop(binary);
    match expired {
        Expired::Failed(figures) => {
            let baseline = match thread_baseline(&deadline) {
                Some(sched) => {
                    format!("CPU {} ns, runnable wait {} ns", sched.cpu, sched.waited.expect("Linux uses schedstat"))
                }
                None => String::from("UNAVAILABLE"),
            };
            eprintln!("deadline accounting observation: {source}; own baseline={baseline}; {figures}");
        }
        Expired::Starved(starved) => starved.pass(),
    }
}

#[test]
fn watching_our_process_and_an_idle_child_keeps_both_sources() {
    let mut child = std::process::Command::new("/bin/sleep").arg("10").spawn().expect("spawn idle child");
    // Let the child enter its idle wait before taking the scheduler baseline.
    let startup = Deadline::arm("the idle child's startup", Duration::from_millis(20));
    std::thread::sleep(core::time::Duration::from_nanos(startup.remaining().as_nanos()));
    let mut deadline = Deadline::arm("our process and idle child", Duration::from_millis(20));
    assert!(has_boot_origin(&deadline), "Linux exposes the shared boot-clock origin");
    deadline.watch_process(std::process::id());
    deadline.watch_process(child.id());
    std::thread::sleep(core::time::Duration::from_nanos(deadline.remaining().as_nanos()));
    assert_eq!(watches(&deadline).count(), 2, "each watch keeps a source");
    let sources = watches(&deadline).waits();
    let expired = deadline.expired();
    child.kill().expect("end idle child");
    child.wait().expect("reap idle child");
    let figures = match expired {
        Expired::Failed(figures) => figures,
        Expired::Starved(starved) => starved.pass(),
    };
    for source in &sources {
        assert!(source.is_some(), "each owned process has readable accounting: {sources:?}; {:?}", watches(&deadline));
        let waited = source.expect("checked readable accounting");
        assert!(waited.as_nanos() < figures.patience.as_nanos().div_ceil(2));
    }
    let waited = figures.tree_waited.expect("both owned processes have readable accounting");
    assert!(waited.as_nanos() < figures.patience.as_nanos().div_ceil(2));
    let sources = sources.iter().copied().map(figure).collect::<Vec<_>>().join(", ");
    eprintln!("deadline accounting: own process and child: sources=[{sources}]; {figures}");
}
