//! A first playground and its reset (temper/docs/design/draft/playground-reset.md, 6.1–6.3, 8).
use crate::support::{
    self, Ended,
    ctl::{self, Call, Step},
    forges,
};
use skein_io::kernel::Exit;
use std::path::Path;

#[test]
fn a_first_run_into_the_playground_and_out_by_its_reset_leaves_the_store_empty() {
    let clock = skein_shell::Clock::new();
    let began = clock.now().now;
    let scratch = support::Scratch::new("playground-reset");
    let (paths, flags) = crate::first_run_directories(&scratch);
    let initialized = std::process::Command::new(skein_scratch::paths::binary("temper"))
        .arg("init")
        .args(&flags)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .stdin(std::process::Stdio::null())
        .current_dir(scratch.path())
        .output()
        .expect("fresh temper init");
    assert!(initialized.status.success(), "{}", String::from_utf8_lossy(&initialized.stderr));
    support::own_listener_files(&paths[0]);
    let running_data = paths[1].clone();
    let reset_data = paths[1].clone();
    let seen = support::boxed::run_ctl_with_exits(
        scratch.path(),
        flags,
        scratch.path(),
        vec![
            Step::Run(Call::sign_in()),
            Step::Until(forges::playground_start(), b"running\n"),
            Step::Until(Call::new(&["forge", "list"]).json(), b"\"state\":\"connected\""),
            Step::Run(Call::new(&["account", "list"]).json()),
            Step::Run(Call::new(&["me"]).json()),
            Step::Then(Box::new(move |calls| {
                let forge = running_forge(&calls[2]);
                assert_running_accounts(&calls[3]);
                assert_playground_project(&calls[4], true);
                assert!(running_data.join(format!("box/groups/{forge}")).is_file(), "playground group settings");
                assert!(running_data.join(format!("box/started/{forge}")).is_file(), "playground group started");
                assert!(running_data.join(format!("forge-{forge}")).is_dir(), "playground forge directory");
                assert!(
                    !running_data.join(format!("forge-{forge}/admin_token")).exists(),
                    "the engine consumed the handover before connecting"
                );
                forges::playground_reset()
            })),
            Step::Run(Call::new(&["forge", "list"]).json()),
            Step::Run(Call::new(&["account", "list"]).json()),
            Step::Then(Box::new(move |calls| {
                assert_eq!(calls[5].stdout, b"reset\n", "reset finished before reading the store");
                assert_empty_forges(&calls[6]);
                assert_reset_accounts(&calls[7]);
                assert_removed(&reset_data, running_forge(&calls[2]));
                Call::new(&["me"]).json()
            })),
            Step::Run(Call::new(&["playground", "reset", "--key", "read-not-started"])),
        ],
        support::boxed::BOX_DEADLINE,
        || (),
        &[(9, 2)],
    );
    let stderr = String::from_utf8_lossy(&seen.stderr);
    assert_eq!(seen.exit, Some(Exit::Code(0)), "box stopped cleanly: {stderr}");
    assert_eq!(seen.left, Some(Vec::new()), "nothing outlived the box: {stderr}");
    assert_eq!(seen.checked, Some(()), "all calls completed before stopping: {stderr}");
    let [
        _signed,
        started,
        running,
        accounts,
        projects,
        reset,
        empty_forges,
        empty_accounts,
        empty_projects,
        not_started,
    ] = seen.calls.as_slice()
    else {
        panic!("sign-in, playground start, three running reads, reset, three empty reads, refused reset: {stderr}")
    };
    for call in &seen.calls[..9] {
        assert_eq!(call.exit, Some(Exit::Code(0)), "{}; box: {stderr}", String::from_utf8_lossy(&call.stderr));
        assert!(call.stderr.is_empty(), "{}", String::from_utf8_lossy(&call.stderr));
    }
    assert_eq!(started.stdout, b"running\n");
    let forge = running_forge(running);
    assert_running_accounts(accounts);
    assert_playground_project(projects, true);
    assert_eq!(reset.stdout, b"reset\n");
    assert_empty_forges(empty_forges);
    assert_reset_accounts(empty_accounts);
    assert_playground_project(empty_projects, false);
    assert_eq!(not_started.exit, Some(Exit::Code(2)));
    assert_eq!(not_started.stdout, b"refused: not offered\n");
    assert!(not_started.stderr.is_empty(), "{}", String::from_utf8_lossy(&not_started.stderr));
    assert_removed(&paths[1], forge);
    eprintln!(
        "reset: reset; forges empty; no scripted provider 3 account; no Playground project; \
         second keyed reset refused: not offered (not_started); group, started marker and forge directory absent; \
         box exit 0 with no remaining processes"
    );
    eprintln!("playground reset story: {} ms", clock.now().now.saturating_since(began).as_nanos() / 1_000_000);
    scratch.remove();
}

fn running_forge(call: &Ended) -> u16 {
    let document = ctl::json(call);
    let [forge] = document["forges"].as_array().expect("forge rows").as_slice() else {
        panic!("only the playground's forge: {document}")
    };
    assert_eq!(forge["state"], "connected", "{document}");
    assert_eq!(forge["mode"], "fake", "{document}");
    let number = u16::try_from(forge["number"].as_u64().expect("forge number")).expect("u16 forge");
    assert_ne!(number, 0);
    number
}

fn assert_empty_forges(call: &Ended) {
    let document = ctl::json(call);
    assert!(document["forges"].as_array().expect("forge rows").is_empty(), "{document}");
}

fn assert_running_accounts(call: &Ended) {
    let document = ctl::json(call);
    let [account] = document["accounts"]["deployment"].as_array().expect("deployment accounts").as_slice() else {
        panic!("only the scripted account: {document}")
    };
    assert_eq!(account["provider"], 3, "scripted provider: {document}");
    assert_eq!(account["provider_name"]["bytes"], "demo", "{document}");
}

fn assert_reset_accounts(call: &Ended) {
    let document = ctl::json(call);
    for holder in ["deployment", "party"] {
        let accounts = document["accounts"][holder].as_array().expect("account rows");
        assert!(!accounts.iter().any(|account| account["provider"] == 3), "no scripted account: {document}");
    }
}

fn assert_playground_project(call: &Ended, present: bool) {
    let document = ctl::json(call);
    let projects = document["projects"].as_array().expect("party projects");
    assert_eq!(projects.iter().any(|project| project["name"]["bytes"] == "Playground"), present, "{document}");
}

fn assert_removed(data: &Path, forge: u16) {
    for path in [format!("box/groups/{forge}"), format!("box/started/{forge}"), format!("forge-{forge}")] {
        assert!(!data.join(&path).exists(), "reset removed {path}");
    }
}
