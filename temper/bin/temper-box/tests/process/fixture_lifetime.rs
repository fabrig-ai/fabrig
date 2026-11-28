//! Kill a test process without running its cleanup (temper/docs/design/testing/fixture-lifetime.md, 4).
use crate::{fake_backends, support};
use skein_io::kernel::Signal;
use skein_world::end_to_end::{Binary, Command, Containment, Expectation, Mode};
use std::{
    fs,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command as StdCommand, Stdio},
    thread,
};
use support::parent::Guard;

const PEER_DOCUMENTS: [&[u8]; 4] = [b"github.json", b"oauth.json", b"llm.json", b"forgejo.json"];
// Serve inserts the sign-in metadata before the closing head tag.
const PAGE: &[u8] = b"<!doctype html><html><head><title>fixture lifetime</title></head><body></body></html>";

pub(super) fn story() {
    if let Some(root) = temper_test_env::value(&temper_test_env::BOX_PARENT_TEST) {
        stand_in(Path::new(&root));
        return;
    }
    let scratch = support::Scratch::new("box-dies-with-test");
    let root = scratch.path();
    let mut command = StdCommand::new(std::env::current_exe().expect("test binary"));
    command
        .args(["--exact", "a_story_box_and_its_steps_die_when_the_test_process_is_killed", "--nocapture"])
        .env(temper_test_env::BOX_PARENT_TEST.name, root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(fs::File::create(root.join("stand-in.stderr")).expect("stand-in stderr"));
    let stand_in = support::fixture::spawn(&mut command, Signal::Kill).expect("stand-in starts");
    let pid = stand_in.id();
    let manifest = root.join("fixture-pids");
    let mut guard = wait_for(Guard::new(stand_in), root, "the stand-in publishes fixture pids", || manifest.exists());
    let pids = fs::read_to_string(manifest).expect("fixture pids");
    let [boxed, forge, llm]: [u32; 3] = pids
        .split_whitespace()
        .map(|pid| pid.parse().expect("fixture pid"))
        .collect::<Vec<_>>()
        .try_into()
        .expect("the box and two independent fakes");
    for child in [boxed, forge, llm] {
        guard.record(child, pid);
    }
    let data = box_data(root);
    let mut guard = wait_for(guard, root, "the independent fakes are ready and serve is listening", || {
        root.join("fakes-ready").exists() && support::boxed::serve_listening(&data)
    });
    for child in [boxed, forge, llm] {
        assert!(guard.executable(child).expect("observe fixture").is_some(), "fixture {child} is running");
    }
    guard.descendants(boxed);
    let long_steps = check_steps(&guard, boxed);

    // This story uses the fresh sandbox's stock configuration. Each running
    // long step can take two grace periods before the box closes its group.
    let bound = temper_box_machine::limits::stock().grace.saturating_mul(2 * long_steps);
    let mut deadline = skein_world::Deadline::arm("the box and fixtures end with the killed test", bound);
    deadline.watch_process(boxed);
    let child = guard.child.as_mut().expect("stand-in");
    child.kill().expect("SIGKILL the stand-in alone");
    child.wait().expect("reap the stand-in");
    while !guard.all_exited() {
        if deadline.passed() {
            match deadline.expired() {
                skein_world::Expired::Starved(starved) => {
                    drop(guard);
                    scratch.remove();
                    starved.pass();
                }
                skein_world::Expired::Failed(figures) => {
                    panic!("a fixture outlived the box's stop bound: {figures}; retained {:?}", guard.pids());
                }
            }
        }
        thread::sleep(std::time::Duration::from_millis(5));
    }
    scratch.remove();
}

fn wait_for(mut guard: Guard, root: &Path, condition: &'static str, ready: impl Fn() -> bool) -> Guard {
    let mut deadline = skein_world::Deadline::arm(condition, support::parent::WITHIN);
    deadline.watch_process(guard.child.as_ref().expect("stand-in").id());
    loop {
        if ready() {
            return guard;
        }
        let child = guard.child.as_mut().expect("stand-in");
        if let Some(exit) = child.try_wait().expect("stand-in status") {
            panic!("{condition}: the stand-in exited with {exit}\n{}", startup_evidence(root));
        }
        if deadline.passed() {
            match deadline.expired() {
                skein_world::Expired::Starved(starved) => {
                    drop(guard);
                    starved.pass();
                }
                skein_world::Expired::Failed(figures) => panic!("{figures}\n{}", startup_evidence(root)),
            }
        }
        thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn startup_evidence(root: &Path) -> String {
    let stderr = fs::read_to_string(root.join("stand-in.stderr")).expect("read stand-in stderr");
    let data = box_data(root);
    let log = support::boxed::box_log(&data);
    let serve = fs::read_to_string(data.join("box/logs/serve.log")).unwrap_or_else(|error| error.to_string());
    let fakes_ready = root.join("fakes-ready").exists();
    format!("independent fakes ready: {fakes_ready}\nstand-in stderr:\n{stderr}\nbox log:\n{log}\nserve log:\n{serve}")
}

fn box_data(root: &Path) -> PathBuf {
    root.join("data/box/fake-backends/data")
}

fn check_steps(guard: &Guard, boxed: u32) -> u64 {
    let mut peers = [false; PEER_DOCUMENTS.len()];
    let mut serve = false;
    let steps = guard.children(boxed);
    for pid in &steps {
        let executable = guard.executable(*pid).expect("observe step").expect("running box step");
        let words: Vec<_> = executable.arguments.split(|byte| *byte == 0).collect();
        if words.get(1..3) == Some(&[b"internal".as_slice(), b"serve".as_slice()]) {
            serve = true;
        }
        for (seen, name) in peers.iter_mut().zip(PEER_DOCUMENTS) {
            *seen |= words.windows(2).any(|pair| pair[0] == b"--document" && pair[1].ends_with(name));
        }
    }
    assert!(serve && peers.into_iter().all(|seen| seen), "the box owns its serve and all four fakes");
    u64::try_from(steps.len()).expect("the number of running steps")
}

fn stand_in(root: &Path) {
    let paths = ["config", "data", "cache"].map(|name| root.join(name));
    let bundle = root.join("bundle");
    fake_backends::bundle(&bundle, PAGE);
    let program = skein_scratch::paths::binary("temper-box");
    let command = support::fixture::dying_with_test(
        Command {
            program: program.clone(),
            arguments: fake_backends::flags(&paths[0], &paths[1], &paths[2], Some(&bundle))
                .into_iter()
                .map(Into::into)
                .collect(),
            directory: root.into(),
            environment: vec![("PATH".into(), "/usr/bin:/bin".into())],
        },
        Signal::Terminate,
    );
    // The stand-in never drives cleanup. Keep the binary and its pipes on this
    // thread until the parent kills the stand-in, without involving a keeper.
    let _box = Binary::start_contained(command, Mode::Pipes, 65_536, Expectation::Supervise, Containment::Group)
        .expect("start the fake-backends box");
    let [boxed]: [u32; 1] =
        Guard::observing_test().children(std::process::id()).try_into().expect("only the box has started");
    let forge = serde_json::json!({"listen":{"socket":"127.0.0.1:0"},"admin_token":root.join("forge-admin"),
        "owner":"fake","repositories":[{"name":"widget","landing":"main","ci":null}]});
    let llm = serde_json::json!({"listen":{"socket":"127.0.0.1:0"},"key":root.join("llm-key"),"tls":null,
        "models":[{"name":"qa","window":4096,"output":1024}],"cued":[],"asked":[],"otherwise":["fallback"]});
    let mut fakes = [("forge", forge), ("llm", llm)].map(|(kind, document)| {
        let path = root.join(format!("{kind}.json"));
        support::write_private(&path, &serde_json::to_vec(&document).expect("fake document"));
        let mut command = StdCommand::new(&program);
        command
            .args(["fake", kind, "--document"])
            .arg(path)
            .args(["--ready", "ready "])
            .env_clear()
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        support::fixture::spawn(&mut command, Signal::Kill).expect("start an independent fake")
    });
    fs::write(root.join("fixture-pids.tmp"), format!("{boxed} {} {}\n", fakes[0].id(), fakes[1].id()))
        .expect("write fixture pids");
    fs::rename(root.join("fixture-pids.tmp"), root.join("fixture-pids")).expect("publish fixture pids");
    for child in &mut fakes {
        let mut line = String::new();
        let pipe = child.stdout.as_mut().expect("ready pipe");
        BufReader::new(pipe).read_line(&mut line).expect("fake prints its ready line");
        assert!(line.starts_with("ready 127.0.0.1:"), "independent fake is ready");
    }
    fs::write(root.join("fakes-ready"), []).expect("publish fake readiness");
    loop {
        thread::park();
    }
}
