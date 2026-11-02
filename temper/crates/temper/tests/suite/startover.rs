//! A cut setup starts over; a cut removal ends whole at startup.
//! temper/docs/design/domain/forge.md, 4.1; temper/docs/design/protocol/setup.md, 6.7 and 13.4.
use super::support::{
    self,
    boxed::{self, BOX_DEADLINE, CtlOutcome},
    ctl::{Call, Calls, Step},
    forge_land::{assert_cached_binary_unchanged, cache_forgejo_binaries, deployment_flags},
};
use skein_io::kernel::Exit;
use skein_world::end_to_end::Binary;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
enum Cut {
    Fetch,
    Prepare,
    Run,
    Started,
}

fn child(binary: &Binary, arguments: &[&[u8]]) -> Option<u32> {
    binary.tree_status().live_pids.iter().copied().find(|pid| {
        std::fs::read(format!("/proc/{pid}/cmdline")).is_ok_and(|bytes| {
            let words = bytes.split(|byte| *byte == 0).collect::<Vec<_>>();
            words.windows(arguments.len()).any(|words| words == arguments)
        })
    })
}

fn signal(pid: u32, name: &str) {
    assert!(
        std::process::Command::new("/usr/bin/kill")
            .args([name, &pid.to_string()])
            .status()
            .expect("fixture signal")
            .success(),
        "the fixture's supervised child accepts {name}"
    );
}

fn answered(outcome: &CtlOutcome<()>, expected: usize) {
    assert_eq!(outcome.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert_eq!(outcome.left, Some(vec![]), "the box drains its children");
    assert!(outcome.checked.is_some(), "{}", String::from_utf8_lossy(&outcome.stderr));
    assert_eq!(outcome.calls.len(), expected);
    for call in &outcome.calls {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&call.stderr));
    }
}

fn absent(data: &Path, number: u16) {
    for name in [format!("forge-{number}"), format!("box/groups/{number}"), format!("box/started/{number}")] {
        assert!(!data.join(&name).exists(), "removed {name}");
    }
}

fn wait_absent(binary: &Binary, data: &Path, number: u16) {
    let mut patience = support::patience::Patience::watching("the forge's removal", BOX_DEADLINE, binary);
    loop {
        let names = [format!("forge-{number}"), format!("box/groups/{number}"), format!("box/started/{number}")];
        if names.iter().all(|name| !data.join(name).exists()) {
            return;
        }
        if let Some(figures) = patience.failed() {
            panic!("forge {number} removal did not settle: {figures}");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn cached(outcome: &CtlOutcome<()>) {
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert!(!stderr.to_ascii_lowercase().contains("download"), "the cached release needs no download: {stderr}");
}

fn finished_setup(scratch: &Path, flags: Vec<String>) -> CtlOutcome<()> {
    boxed::run_ctl_with_cut(
        scratch,
        flags,
        vec![Step::Run(Call::new(&["forge", "setup"])), Step::Run(Call::new(&["forge", "list", "--json"]))],
        |_, _| false,
    )
}

#[test]
fn owned_setup_cuts_start_over_and_a_removal_cut_finishes_on_restart() {
    let scratch = support::Scratch::new("owned-setup-cuts");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    support::forge_land::initialize(&paths);
    support::own_listener_files(&paths[0]);
    let (binaries, sources) = cache_forgejo_binaries(&paths[2]);
    let flags = boxed::echo_steps(deployment_flags(&paths));
    for (index, cut) in [Cut::Fetch, Cut::Prepare, Cut::Run, Cut::Started].into_iter().enumerate() {
        start_over(&scratch, &paths, &flags, cut, u16::try_from(index + 1).expect("four cuts"));
    }
    let fresh = finished_setup(scratch.path(), flags.clone());
    answered(&fresh, 2);
    let list = support::ctl::json(&fresh.calls[1]);
    let rows = list["forges"].as_array().expect("new forge rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["number"], 5, "removed numbers are never reused");
    assert_eq!(rows[0]["first"]["state"], "made");
    cached(&fresh);
    removal_cut(&scratch, &paths, flags, 5);
    for (binary, source) in binaries.iter().zip(&sources) {
        assert_cached_binary_unchanged(binary, source);
    }
    scratch.remove();
}

fn start_over(scratch: &support::Scratch, paths: &[std::path::PathBuf; 3], flags: &[String], cut: Cut, number: u16) {
    let number_text = number.to_string();
    let mut engine_paused = false;
    let data = &paths[1];
    eprintln!("start-over {cut:?}: cut");
    let first = boxed::run_ctl_with_cut(
        scratch.path(),
        flags.to_vec(),
        vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["forge", "setup"]))],
        |binary, _| {
            let stage = match cut {
                Cut::Fetch => child(binary, &[b"forge", b"fetch"]),
                Cut::Prepare => child(binary, &[b"forge", b"prepare"]),
                Cut::Run | Cut::Started => child(binary, &[b"forge", b"run"]).or_else(|| child(binary, &[b"web"])),
            };
            if stage.is_none() {
                return false;
            }
            if matches!(cut, Cut::Started) {
                // Hold the engine before it can receive `started N` and adopt.
                if !engine_paused {
                    signal(child(binary, &[b"serve"]).expect("engine under this box"), "-STOP");
                    engine_paused = true;
                }
                return data.join(format!("box/started/{number}")).exists()
                    && binary.stderr().windows(b"Forgejo: started\n".len()).any(|line| line == b"Forgejo: started\n");
            }
            assert!(!data.join(format!("box/started/{number}")).exists(), "the cut precedes started");
            true
        },
    );
    assert!(first.checked.is_some(), "cut at {cut:?}: {}", String::from_utf8_lossy(&first.stderr));
    assert_eq!(first.exit, Some(Exit::Signal(9)));
    eprintln!("start-over {cut:?}: remove after restart");
    let restarted = boxed::run_ctl_with_cut(
        scratch.path(),
        flags.to_vec(),
        vec![
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::Run(Call::new(&["forge", "remove", &number_text])),
        ],
        |binary, calls| {
            if calls.idle() {
                wait_absent(binary, data, number);
            }
            false
        },
    );
    answered(&restarted, 2);
    let list = support::ctl::json(&restarted.calls[0]);
    let rows = list["forges"].as_array().expect("unfinished forge rows");
    assert_eq!(rows.len(), 1, "one unfinished owned forge: {list}");
    assert_eq!(rows[0]["number"], number);
    assert_eq!(rows[0]["mode"], "owned");
    assert_ne!(rows[0]["first"]["state"], "made", "the cut precedes adoption: {list}");
    assert_eq!(String::from_utf8_lossy(&restarted.calls[1].stdout).trim(), "removed");
    absent(data, number);
}

fn removal_cut(scratch: &support::Scratch, paths: &[std::path::PathBuf; 3], flags: Vec<String>, number: u16) {
    let number_text = number.to_string();
    let paused = std::rc::Rc::new(std::cell::Cell::new(false));
    let box_pid = std::rc::Rc::new(std::cell::Cell::new(None));
    let cut = boxed::run_ctl_with_cut(
        scratch.path(),
        flags.clone(),
        vec![
            Step::Run(Call::new(&["forge", "list", "--until", &format!("connected:{number}")])),
            Step::Then(Box::new({
                let paused = paused.clone();
                let box_pid = box_pid.clone();
                move |_| {
                    signal(box_pid.get().expect("the fixture box"), "-STOP");
                    paused.set(true);
                    Call::new(&["forge", "remove", &number_text])
                }
            })),
        ],
        |binary, calls: &Calls| {
            box_pid.set(child(binary, &[b"box"]));
            paused.get()
                && calls.ends().iter().any(|call| call.exit == Some(Exit::Code(0)) && call.stdout == b"removed\n")
        },
    );
    assert!(cut.checked.is_some(), "cut after removed answer: {}", String::from_utf8_lossy(&cut.stderr));
    assert_eq!(cut.exit, Some(Exit::Signal(9)));
    assert!(paths[1].join(format!("box/groups/{number}")).exists(), "paused box has not completed removal");
    let restarted = boxed::run_ctl_with_cut(
        scratch.path(),
        flags,
        vec![
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::Run(Call::new(&["forge", "list"])),
            Step::Run(Call::new(&["task", "list", "--project", "1", "--json"])),
        ],
        |binary, calls| {
            if calls.idle() {
                wait_absent(binary, &paths[1], number);
            }
            false
        },
    );
    answered(&restarted, 3);
    let list = support::ctl::json(&restarted.calls[0]);
    assert!(
        list["forges"].as_array().expect("forge rows").is_empty(),
        "no removed forge or secret-backed connection: {list}"
    );
    assert!(!String::from_utf8_lossy(&restarted.calls[1].stdout).lines().any(|line| line.starts_with("forge 1")));
    absent(&paths[1], number);
}
