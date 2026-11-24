//! An owned setup cut at its run, then a removal cut recovered at startup.
//! temper/docs/design/domain/forge/adoption-connection.md, 1; temper/docs/design/protocol/setup/owned-repositories.md, 1, and temper/docs/design/protocol/setup/testing.md, 4.
// The readiness barrier reads the box's supervised processes through /proc.
#![cfg(target_os = "linux")]
use super::support::{
    self,
    boxed::{self, BOX_DEADLINE, CtlOutcome},
    ctl::{Call, Step, json},
    forge_land::{assert_cached_binary_unchanged, cache_forgejo_binaries, deployment_flags},
};
use skein_io::kernel::Exit;
use skein_world::end_to_end::Binary;
use std::{cell::Cell, path::Path, rc::Rc};

fn box_pid(binary: &Binary) -> Option<u32> {
    let program = skein_scratch::paths::binary("temper-box");
    binary.tree_status().live_pids.iter().copied().find(|pid| {
        std::fs::read(format!("/proc/{pid}/cmdline")).is_ok_and(|bytes| {
            let mut arguments = bytes.split(|byte| *byte == 0);
            // The engine and setup commands share the executable but put a verb before the flags.
            arguments.next() == Some(program.as_os_str().as_encoded_bytes()) && arguments.next() == Some(b"--config")
        })
    })
}

fn killed(outcome: &CtlOutcome<()>) {
    assert!(outcome.checked.is_some(), "cut observed: {}", String::from_utf8_lossy(&outcome.stderr));
    assert_eq!(outcome.exit, Some(Exit::Signal(9)));
    // run_ctl_with_cut checks that the keeper drains the whole tree before it returns.
}

const REMOVAL_PATHS: [&str; 3] = ["forge-1", "box/groups/1", "box/started/1"];

fn wait_removed(binary: &Binary, data: &Path) {
    let mut patience = support::patience::Patience::watching("the forge's removal", BOX_DEADLINE, binary);
    while REMOVAL_PATHS.iter().any(|name| data.join(name).try_exists().expect("removal path")) {
        if let Some(figures) = patience.failed() {
            panic!("forge 1 removal did not settle: {figures}");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

#[test]
fn an_owned_setup_killed_at_its_run_is_kept_and_a_removal_cut_finishes_on_restart() {
    let scratch = support::Scratch::new("owned-setup-run-cut");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    support::forge_land::initialize(&paths);
    support::own_listener_files(&paths[0]);
    let (binaries, sources) = cache_forgejo_binaries(&paths[2]);
    let flags = boxed::echo_steps(deployment_flags(&paths));

    let setup = boxed::run_ctl_with_cut(
        scratch.path(),
        flags.clone(),
        vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["forge", "setup"]))],
        |binary, _| boxed::find(&binary.tree_status().live_pids, &binaries[0], b"web").is_some(),
    );
    killed(&setup);

    let leader = Rc::new(Cell::new(None::<u32>));
    let removal = boxed::run_ctl_with_cut(
        scratch.path(),
        flags.clone(),
        vec![
            Step::Run(Call::new(&["forge", "list", "--json"])),
            Step::Then(Box::new({
                let leader = leader.clone();
                move |calls| {
                    let list = json(&calls[0]);
                    let [forge] = list["forges"].as_array().expect("unfinished forge rows").as_slice() else {
                        panic!("one unfinished owned forge: {list}");
                    };
                    assert_eq!(forge["number"], 1);
                    assert_eq!(forge["mode"], "owned");
                    assert_eq!(forge["first"]["state"], "making", "the cut precedes adoption: {list}");
                    // Hold only the box. Its engine can commit and answer, but the box cannot remove the group.
                    let pid = leader.get().expect("the fixture box");
                    assert!(
                        std::process::Command::new("/bin/kill")
                            .args(["-STOP", &pid.to_string()])
                            .status()
                            .expect("stop the box")
                            .success()
                    );
                    Call::new(&["forge", "remove", "1"])
                }
            })),
        ],
        |binary, calls| {
            if leader.get().is_none() {
                leader.set(box_pid(binary));
            }
            calls.ends().last().is_some_and(|call| call.exit == Some(Exit::Code(0)) && call.stdout == b"removed\n")
        },
    );
    killed(&removal);
    assert!(paths[1].join("box/groups/1").exists(), "the held box did not complete removal");

    let restarted = boxed::run_ctl_with_cut(
        scratch.path(),
        flags,
        vec![Step::Run(Call::new(&["forge", "list", "--json"]))],
        |binary, calls| {
            if calls.idle() {
                wait_removed(binary, &paths[1]);
            }
            false
        },
    );
    assert_eq!(restarted.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&restarted.stderr));
    assert_eq!(restarted.left, Some(vec![]), "the restarted box drains its children");
    assert!(restarted.checked.is_some());
    let [listed] = restarted.calls.as_slice() else { panic!("one forge list") };
    assert_eq!(listed.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&listed.stderr));
    assert_eq!(json(listed)["forges"], serde_json::json!([]));
    for name in REMOVAL_PATHS {
        assert!(!paths[1].join(name).try_exists().expect("removal path"), "removed {name}");
    }
    for (binary, source) in binaries.iter().zip(&sources) {
        assert_cached_binary_unchanged(binary, source);
    }
    scratch.remove();
}
