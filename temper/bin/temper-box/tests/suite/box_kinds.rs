//! A real owned slot driven over the box pipe without serve (temper/docs/design/protocol/setup/box.md, 1).
use super::support;
use skein_io::kernel::{Exit, Fd, ServiceSignal};
use skein_lib::{Duration, Time};
use skein_shell::{Delegate, Host};
use skein_world::{
    Referee,
    real::{Controls, World},
};
use std::{fs, os::fd::IntoRawFd, path::PathBuf};
use temper_box_machine::host::{Service, Settings};
use temper_forge_setup::{Kind, Step};

#[derive(Debug)]
struct BoxProcess(Service);
impl Delegate for BoxProcess {
    fn host(&self) -> &dyn Host {
        &self.0
    }
    fn host_mut(&mut self) -> &mut dyn Host {
        &mut self.0
    }
}
struct Judge {
    controls: Controls,
    removed: PathBuf,
    stopping: bool,
    settled: bool,
    wake: Option<Time>,
}
impl Referee<BoxProcess> for Judge {
    fn act(&mut self, _now: Time, procs: &mut [BoxProcess]) {
        self.wake = None;
        if !self.stopping && self.removed.exists() {
            self.controls.signal(0, ServiceSignal::Terminate);
            self.stopping = true;
        }
        self.settled = self.stopping && procs.iter().all(Host::is_empty);
    }
    fn watch(&mut self, now: Time, procs: &[BoxProcess]) -> Result<(), String> {
        if (!self.stopping && self.removed.exists()) || (self.stopping && procs.iter().all(Host::is_empty)) {
            self.wake = Some(now);
        } else if !self.settled {
            self.wake = Some(now.saturating_add(Duration::from_millis(10)));
        }

        Ok(())
    }
    fn next_deadline(&self) -> Option<Time> {
        self.wake
    }
    fn passed(&self) -> bool {
        self.settled
    }
}

fn command(step: &Step) -> Step {
    let mut args: Vec<Box<[u8]>> = vec![
        b"-c".as_slice().into(),
        b"exec \"$@\"".as_slice().into(),
        b"slot".as_slice().into(),
        skein_scratch::paths::binary("temper-box").as_os_str().as_encoded_bytes().into(),
    ];
    args.extend_from_slice(step.arguments());
    let mut wrapped =
        Step::new(step.name().into(), args.into_boxed_slice(), step.kind()).listening(step.listeners().into());
    if let Some(ready) = step.ready() {
        wrapped = wrapped.with_ready(ready);
    }
    wrapped.set_started_notice(step.started_notice().into());
    wrapped
}

const DRIVER: &[u8] = br#"set -eu
printf 'start 17 owned\n' >&3
while IFS= read -r reply <&4; do
    case "$reply" in
        'started 17') break ;;
        'failed '*) exit 1 ;;
    esac
done
test -f box/groups/17
test -f box/started/17
test -d forge-17
printf 'remove 17\n' >&3
while IFS= read -r reply <&4; do
    case "$reply" in
        'removed 17') break ;;
        'failed '*) exit 1 ;;
    esac
done
test ! -e box/groups/17
test ! -e box/started/17
test ! -e forge-17
: > removed
exec /bin/sleep 30
"#;

#[test]
#[expect(clippy::too_many_lines, reason = "the owned-slot story keeps its start and removal evidence together")]
fn a_box_starts_an_owned_slot_on_start_kind_and_removes_it() {
    let scratch = support::Scratch::new("box-owned-kind");
    let root = scratch.path();
    let cache = root.join("cache");
    fs::create_dir(&cache).expect("cache");
    let [listen] = support::free_addresses::<1>(&[]);
    let (_, document) = support::configuration::documents(root, "/fixture/temper");
    let mut document: serde_json::Value = serde_json::from_slice(&document).expect("fixture document");
    document["application"]["machine"] = serde_json::json!({"owned":{
        "listen":{"socket":listen.to_string()}, "remote":format!("http://{listen}"),
        "person":{"login":"ada","email":"ada@example.invalid"},
        "forgejo":"fetched", "runner":null
    }});
    let bytes = serde_json::to_vec(&document).expect("document JSON");
    fs::write(root.join("temper.json"), &bytes).expect("document");
    let mut directories = support::configuration::directories(root);
    directories.cache = cache.as_os_str().as_encoded_bytes().into();
    let (mut steps, limits) =
        temper_box_shell::box_command::configuration_steps(&directories, &bytes, Box::new([])).expect("kind steps");
    let (cached, _) = support::forge_land::cache_forgejo_binaries(&cache);
    assert_eq!(steps.always.len(), 1, "serve is the always step");
    assert_eq!(steps.asked.len(), 1, "owned slot");
    assert!(steps.asked[0].steps.iter().any(|(_, step)| !step.started_notice().is_empty()), "owned notice");
    // The shell is the pipe's owner in serve's place; every slot command execs the shipped binary.
    steps.always = Box::new([Step::new(
        b"serve".as_slice().into(),
        Box::new([b"-c".as_slice().into(), DRIVER.into()]),
        Kind::Long,
    )]);
    let mut listeners = Vec::new();
    for group in &mut steps.asked {
        for (_, step) in &mut group.steps {
            for address in step.listeners() {
                let (fd, _) = skein_shell::open_listener(*address, 16).expect("slot listener");
                listeners.push((fd, *address));
            }
            *step = command(step);
        }
        group.remover = group.remover.as_ref().map(command);
    }
    let environment = temper_test_env::environment()
        .into_vec()
        .into_iter()
        .map(|(key, value)| {
            let mut entry = key.into_vec();
            entry.push(b'=');
            entry.extend_from_slice(&value);
            entry.into_boxed_slice()
        })
        .collect();
    let stdout = Fd::new(fs::File::create(root.join("stdout")).expect("stdout").into_raw_fd());
    let stderr = Fd::new(fs::File::create(root.join("stderr")).expect("stderr").into_raw_fd());
    let directory = skein_shell::open_root(root).expect("root");
    let mut world = World::new_controlled(|controls| Judge {
        controls,
        removed: root.join("removed"),
        stopping: false,
        settled: false,
        wake: None,
    });
    world.spawn_signals(|signals| {
        BoxProcess(
            Service::new(
                &limits,
                steps,
                Settings {
                    word: b"temper".as_slice().into(),
                    link_form: temper_box_machine::terminal::LinkForm::Bare,
                    front: None,
                    first_link: None,
                    after_block: Box::new([]),
                    log_lines: Box::new([]),
                    parent: None,
                    program: b"/bin/sh".as_slice().into(),
                    environment,
                    serve: b"serve".as_slice().into(),
                    listeners: listeners.into_boxed_slice(),
                    root: directory,
                    echo: temper_box_machine::host::Echo::Steps,
                    data: root.as_os_str().as_encoded_bytes().into(),
                    signals,
                    stdout,
                    stderr,
                },
            )
            .expect("real box"),
        )
    });
    let outcome = world.run(&skein_shell::Clock::new(), support::boxed::BOX_DEADLINE);
    let stderr = fs::read_to_string(root.join("stderr")).expect("box diagnostics");
    assert!(outcome.killed.is_empty(), "{stderr}");
    assert!(root.join("removed").exists(), "start and remove assertions completed: {stderr}");
    assert_eq!(outcome.procs[0].exit(), Some(Exit::Code(0)), "{stderr}");
    for path in ["box/groups/17", "box/started/17", "forge-17"] {
        assert!(!root.join(path).exists(), "{path}");
    }
    assert!(cached[0].is_file(), "Forgejo's cached binary stays");
    let log = support::boxed::box_log(root);
    for words in ["Forge 17 (Forgejo): starting", "Forge 17 (Forgejo): started", "Forge 17 (Forgejo): stopped"] {
        assert_eq!(log.lines().filter(|line| line.ends_with(words)).count(), 1, "{log}");
    }
    assert!(!stderr.contains("Forgejo: api ") && !stderr.contains("Forgejo: person "), "{stderr}");
    let log = fs::read_to_string(root.join("box/logs/forgejo-17.log")).expect("Forgejo step log");
    for words in [format!("Forgejo: api http://{listen}/api/v1"), "Forgejo: person ada".to_owned()] {
        let lines: Vec<_> = log.lines().filter(|line| line.ends_with(&words)).collect();
        assert_eq!(lines.len(), 1, "hands are logged once: {log}");
        assert_eq!(lines[0].strip_suffix(&words).expect("hands suffix").len(), 21, "UTC prefix");
    }
    scratch.remove();
}
