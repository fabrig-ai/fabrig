//! The writer's run (client/journey-writer.md, 4), with real git in scratch
//! and scripted cargo/temper neighbours. No print, service or network runs.
use skein_scratch::Scratch;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ROOT: &str = "temper/docs/design/client/web/journeys";
const PAGE: &str = "temper/docs/design/client/web/journeys/welcome/01.html";
const README: &str = "temper/docs/design/client/web/journeys/welcome/README.md";
const ARGS: &[&str] = &["--project", "P", "--as-allowed"];

const CARGO: &str = r"#!/usr/bin/env python3
import json, os, pathlib, sys
root = pathlib.Path(os.environ['WRITER_FAKE'])
with (root / 'cargo.log').open('a') as log:
    log.write(json.dumps({'args': sys.argv[1:], 'bundle': os.environ.get('TEMPER_INTERNAL_BUNDLE')}) + '\n')
config = json.loads((root / 'print.json').read_text())
for name, contents in config['writes']:
    path = pathlib.Path(name)
    if contents is None:
        path.unlink()
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(contents)
print(config['stdout'], end='')
sys.exit(config['exit'])
";

const TEMPER: &str = r"#!/usr/bin/env python3
import json, os, pathlib, sys
root = pathlib.Path(os.environ['WRITER_FAKE'])
with (root / 'temper.log').open('a') as log:
    log.write(json.dumps({'args': sys.argv[1:], 'stdin': sys.stdin.read(), 'locale': os.environ.get('LC_ALL')}) + '\n')
if sys.argv[1:4] == ['ctl', 'task', 'show']:
    print((root / 'task.json').read_text())
else:
    print('task 42')
";

struct Repositories {
    scratch: Scratch,
    person: PathBuf,
    origin: PathBuf,
}

impl Repositories {
    fn new(name: &str) -> Self {
        let scratch = Scratch::new(name);
        let person = scratch.path().join("person");
        let origin = scratch.path().join("origin.git");
        for directory in ["home", "hooks", "bin"] {
            fs::create_dir(scratch.path().join(directory)).expect("isolated directory");
        }
        fs::write(scratch.path().join("gitconfig"), "").expect("empty global configuration");
        for (name, script) in [("cargo", CARGO), ("temper", TEMPER)] {
            let path = scratch.path().join("bin").join(name);
            fs::write(&path, script).expect("fake neighbour");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).expect("executable fake");
        }
        let repos = Self { scratch, person, origin };
        repos.git(repos.scratch.path(), &["init", "--bare", "--quiet", "origin.git"]);
        repos.git(repos.scratch.path(), &["clone", "--quiet", "origin.git", "person"]);
        fs::create_dir_all(repos.person.join(ROOT).join("welcome")).expect("journey directory");
        repos.write(PAGE, "<p>Main's page</p>\n");
        repos.write(README, "Journey source\n");
        repos.git(&repos.person, &["add", "."]);
        repos.git(&repos.person, &["commit", "--quiet", "-m", "Main's journeys"]);
        repos.git(&repos.person, &["push", "--quiet", "origin", "main"]);
        repos.print(&[], "", 0);
        repos.phase("running");
        repos
    }

    #[expect(clippy::disallowed_methods, reason = "cli C4: PATH, through temper's test table once it exists")]
    fn command(&self, program: &str, directory: &Path) -> Command {
        let path = std::env::var_os("PATH").expect("git, sh and python3 on PATH");
        let mut paths = vec![self.scratch.path().join("bin")];
        paths.extend(std::env::split_paths(&path));
        let mut command = Command::new(program);
        command
            .current_dir(directory)
            .env_clear()
            .env("PATH", std::env::join_paths(paths).expect("fake neighbours first"))
            .env("WRITER_FAKE", self.scratch.path())
            .env("HOME", self.scratch.path().join("home"))
            .env("XDG_CONFIG_HOME", self.scratch.path().join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.scratch.path().join("gitconfig"))
            .env("GIT_AUTHOR_NAME", "Journey Writer")
            .env("GIT_AUTHOR_EMAIL", "writer@example.invalid")
            .env("GIT_COMMITTER_NAME", "Journey Keeper")
            .env("GIT_COMMITTER_EMAIL", "keeper@example.invalid")
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-01-02T00:00:00Z")
            .env("GIT_EDITOR", "true")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .env("TZ", "UTC")
            .env("TMPDIR", self.scratch.path());
        let settings = [
            ("init.defaultBranch", "main".to_owned()),
            ("commit.gpgSign", "false".to_owned()),
            ("core.autocrlf", "false".to_owned()),
            ("core.hooksPath", self.scratch.path().join("hooks").to_string_lossy().into_owned()),
        ];
        command.env("GIT_CONFIG_COUNT", settings.len().to_string());
        for (index, (key, value)) in settings.iter().enumerate() {
            command.env(format!("GIT_CONFIG_KEY_{index}"), key).env(format!("GIT_CONFIG_VALUE_{index}"), value);
        }
        command
    }

    fn git(&self, directory: &Path, arguments: &[&str]) -> String {
        let output = self.command("git", directory).args(arguments).output().expect("scratch git");
        assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).expect("git text").trim_end().to_owned()
    }

    fn write(&self, path: &str, contents: &str) {
        fs::write(self.person.join(path), contents).expect("scratch file");
    }

    fn print(&self, writes: &[(&str, Option<&str>)], stdout: &str, status: i32) {
        let config = serde_json::json!({"writes": writes, "stdout": stdout, "exit": status});
        fs::write(self.scratch.path().join("print.json"), config.to_string()).expect("print's answer");
    }

    fn phase(&self, phase: &str) {
        let document = serde_json::json!({"task": {"chip": {"task": 42, "phase": phase}}});
        fs::write(self.scratch.path().join("task.json"), document.to_string()).expect("task's document");
    }

    fn run(&self, arguments: &[&str], status: i32) -> Output {
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/journey-pages.sh");
        let output = self.command("sh", &self.person).arg(script).args(arguments).output().expect("writer's run");
        assert_eq!(
            output.status.code(),
            Some(status),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn calls(&self, neighbour: &str) -> Vec<serde_json::Value> {
        let path = self.scratch.path().join(format!("{neighbour}.log"));
        if !path.exists() {
            return Vec::new();
        }
        fs::read_to_string(path).expect("calls").lines().map(|line| serde_json::from_str(line).expect("call")).collect()
    }

    fn task_file(&self) -> PathBuf {
        self.person.join(".git/journey-pages")
    }

    fn assert_nothing_offered(&self, tip: &str) {
        assert_eq!(self.git(&self.person, &["rev-parse", "HEAD"]), tip, "nothing committed");
        assert_eq!(self.git(&self.origin, &["branch", "--list", "journey-pages"]), "", "nothing pushed");
        assert!(self.calls("temper").is_empty(), "nothing opened");
        assert!(!self.task_file().exists());
    }

    fn assert_offered(&self, tip: &str, report: &str, budget: &[&str]) {
        let head = self.git(&self.origin, &["rev-parse", "journey-pages"]);
        assert_eq!(self.git(&self.person, &["rev-parse", "HEAD"]), head);
        assert_eq!(self.git(&self.origin, &["rev-parse", "journey-pages^"]), tip);
        let short = tip.get(..12).expect("twelve hash digits");
        let subject = format!("Journey pages printed at {short}");
        let body = if report.is_empty() { format!("Printed: {tip}") } else { format!("{report}\n\nPrinted: {tip}") };
        assert_eq!(self.git(&self.origin, &["log", "-1", "--format=%B", &head]), format!("{subject}\n\n{body}"));
        assert_eq!(self.git(&self.origin, &["diff-tree", "--no-commit-id", "--name-only", "-r", &head]), PAGE);
        assert_eq!(
            self.git(&self.origin, &["log", "-1", "--format=%an <%ae>", &head]),
            "Journey Writer <writer@example.invalid>"
        );
        assert_eq!(fs::read_to_string(self.task_file()).expect("kept task number"), "42\n");
        let calls = self.calls("temper");
        let call = calls.last().expect("change opened");
        let mut expected = vec![
            "ctl",
            "change",
            "open",
            "--project",
            "P",
            "--branch",
            "journey-pages",
            "--at",
            &head,
            "--title",
            &subject,
            "--body",
            &body,
        ];
        expected.extend_from_slice(budget);
        let key = format!("journey-pages-{tip}");
        expected.extend_from_slice(&["--key", &key]);
        assert_eq!(call["args"], serde_json::json!(expected));
        assert_eq!(call["stdin"], "");
        assert_eq!(call["locale"], "C");
    }
}

#[test]
fn unchanged_pages_start_no_change() {
    let repos = Repositories::new("journey-pages-unchanged");
    let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
    let output = repos.run(ARGS, 0);
    assert_eq!(String::from_utf8_lossy(&output.stdout), format!("main's pages are {tip}'s print\n"));
    repos.assert_nothing_offered(&tip);
    assert_eq!(
        repos.calls("cargo"),
        vec![
            serde_json::json!({"args": ["run", "--locked", "-p", "temper-client-journeys", "--", "pages"], "bundle": "none"})
        ]
    );
}

#[test]
fn changed_pages_offer_the_fetched_tip_with_the_given_budget() {
    for budget in [vec!["--as-allowed"], vec!["--budget", "25"]] {
        let repos = Repositories::new("journey-pages-offered");
        let old = repos.git(&repos.person, &["rev-parse", "origin/main"]);
        repos.write("another", "A newer main\n");
        repos.git(&repos.person, &["add", "another"]);
        repos.git(&repos.person, &["commit", "--quiet", "-m", "Advance main"]);
        repos.git(&repos.person, &["push", "--quiet", repos.origin.to_str().expect("origin path"), "main"]);
        let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
        repos.git(&repos.person, &["checkout", "--quiet", "--detach", &old]);
        assert_eq!(repos.git(&repos.person, &["rev-parse", "origin/main"]), old, "stale until fetched");
        repos.print(&[(PAGE, Some("<p>Printed</p>\n"))], "walked again welcome\nreprinted welcome\n", 0);
        let mut arguments = vec!["--project", "P"];
        arguments.extend_from_slice(&budget);
        arguments.extend_from_slice(&["--prepared", "prepared inputs"]);
        repos.run(&arguments, 0);
        repos.assert_offered(&tip, "walked again welcome", &budget);
        assert_eq!(repos.git(&repos.origin, &["show", &format!("journey-pages:{PAGE}")]), "<p>Printed</p>");
        assert_eq!(repos.calls("temper").len(), 1, "opened once");
        assert_eq!(
            repos.calls("cargo"),
            vec![
                serde_json::json!({"args": ["run", "--locked", "-p", "temper-client-journeys", "--", "pages", "--prepared", "prepared inputs"], "bundle": "none"})
            ]
        );
    }
}

#[test]
fn a_live_change_waits_and_every_ended_phase_releases_it() {
    let repos = Repositories::new("journey-pages-one-at-a-time");
    let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
    repos.print(&[(PAGE, Some("<p>Printed</p>\n"))], "walked again welcome\n", 0);
    repos.run(ARGS, 0);
    let head = repos.git(&repos.origin, &["rev-parse", "journey-pages"]);
    // A fetch would fail: waiting must precede fetching, printing and pushing.
    repos.git(&repos.person, &["remote", "set-url", "origin", "../absent.git"]);
    let output = repos.run(ARGS, 0);
    assert_eq!(String::from_utf8_lossy(&output.stdout), "waiting on change 42\n");
    assert_eq!(repos.calls("cargo").len(), 1);
    assert_eq!(repos.calls("temper").len(), 2);
    assert_eq!(
        repos.calls("temper").last().expect("task read")["args"],
        serde_json::json!(["ctl", "task", "show", "42", "--json"])
    );
    assert_eq!(repos.git(&repos.origin, &["rev-parse", "journey-pages"]), head);
    repos.git(&repos.person, &["remote", "set-url", "origin", "../origin.git"]);
    for phase in ["ended", "done", "failed", "cancelled"] {
        let before = repos.calls("cargo").len();
        repos.phase(phase);
        // Each replacement is a different child of main: only a forced push
        // can replace the script's previous offer after its task has ended.
        repos.print(&[(PAGE, Some(phase))], "walked again welcome\n", 0);
        repos.run(ARGS, 0);
        assert_eq!(repos.calls("cargo").len(), before + 1, "{phase} must release the writer");
        repos.assert_offered(&tip, "walked again welcome", &["--as-allowed"]);
    }
    // An ended task is forgotten even when the next print changes no byte.
    repos.phase("ended");
    repos.print(&[], "", 0);
    repos.run(ARGS, 0);
    assert!(!repos.task_file().exists());
    for call in repos.calls("temper") {
        assert_eq!(call["stdin"], "");
    }
}

#[test]
fn an_unreadable_task_document_starts_nothing() {
    let repos = Repositories::new("journey-pages-unreadable-task");
    let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
    fs::write(repos.task_file(), "42\n").expect("kept task");
    for document in ["not json", "{}", r#"{"task":{"chip":{"phase":"unfamiliar"}}}"#] {
        fs::write(repos.scratch.path().join("task.json"), document).expect("unreadable document");
        let output = repos.run(ARGS, 1);
        assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read the task document"));
        assert_eq!(repos.calls("cargo"), Vec::<serde_json::Value>::new());
        assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD"]), tip);
        assert_eq!(repos.git(&repos.origin, &["branch", "--list", "journey-pages"]), "");
        assert_eq!(fs::read_to_string(repos.task_file()).expect("task kept"), "42\n");
    }
}

#[test]
fn paths_outside_the_written_set_are_never_offered() {
    for path in [
        README,
        "outside",
        "temper/docs/design/client/web/journeys/welcome/1.html",
        "temper/docs/design/client/web/journeys/welcome/nested/01.html",
    ] {
        let repos = Repositories::new("journey-pages-outside");
        let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
        repos.print(&[(PAGE, Some("<p>Printed</p>\n")), (path, Some("Not the writer's\n"))], "", 0);
        let output = repos.run(ARGS, 1);
        assert!(String::from_utf8_lossy(&output.stderr).contains(path));
        repos.assert_nothing_offered(&tip);
        assert_eq!(repos.git(&repos.person, &["diff", "--cached", "--name-only"]), "");
    }
}

#[test]
fn written_additions_and_deletions_are_committed_together() {
    let repos = Repositories::new("journey-pages-written-paths");
    let page = "temper/docs/design/client/tui/journeys/a space/100.txt";
    let css = "temper/docs/design/client/web/journeys/skein-ui.css";
    repos.print(&[(PAGE, None), (page, Some("Terminal page\n")), (css, Some("css\n"))], "", 0);
    repos.run(ARGS, 0);
    let changes = repos.git(&repos.origin, &["diff-tree", "--no-commit-id", "--name-status", "-r", "journey-pages"]);
    assert_eq!(changes, format!("A\t{page}\nA\t{css}\nD\t{PAGE}"));
}

#[test]
fn a_stopped_walk_offers_other_pages_but_an_incomplete_print_does_not() {
    let repos = Repositories::new("journey-pages-stopped");
    let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
    let report = "walked again welcome\nstopped blocked: line 7: missing control";
    repos.print(&[(PAGE, Some("<p>Other pages printed</p>\n"))], &format!("{report}\n"), 1);
    repos.run(ARGS, 1);
    repos.assert_offered(&tip, report, &["--as-allowed"]);
    for status in [2, 3] {
        let repos = Repositories::new("journey-pages-incomplete");
        let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
        repos.print(&[(PAGE, Some("<p>Partly printed</p>\n"))], "stopped blocked: unreadable root\n", status);
        repos.run(ARGS, 1);
        repos.assert_nothing_offered(&tip);
    }
}

#[test]
fn unsuitable_checkouts_are_refused_before_starting() {
    let repos = Repositories::new("journey-pages-refused-checkout");
    let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
    repos.write(PAGE, "Unsaved page\n");
    repos.run(ARGS, 2);
    repos.git(&repos.person, &["add", PAGE]);
    repos.run(ARGS, 2);
    repos.git(&repos.person, &["reset", "--hard", &tip]);
    for state in ["rebase-merge", "rebase-apply", "MERGE_HEAD", "CHERRY_PICK_HEAD", "sequencer"] {
        let path = repos.person.join(".git").join(state);
        fs::write(&path, "").expect("in-progress sentinel");
        repos.run(ARGS, 2);
        fs::remove_file(path).expect("remove sentinel");
    }
    assert_eq!(repos.calls("cargo"), Vec::<serde_json::Value>::new());
    repos.assert_nothing_offered(&tip);
    assert!(!repos.person.join(".git/FETCH_HEAD").exists(), "nothing fetched");
}

#[test]
fn refused_arguments_start_nothing() {
    let repos = Repositories::new("journey-pages-refused-arguments");
    let tip = repos.git(&repos.origin, &["rev-parse", "main"]);
    for arguments in [
        vec![],
        vec!["--unknown"],
        vec!["--project", "P", "--budget", "25", "--as-allowed"],
        vec!["--project", "P"],
        vec!["--as-allowed"],
        vec!["--project", "P", "--budget"],
        vec!["--project", "P", "--as-allowed", "--prepared"],
    ] {
        repos.run(&arguments, 2);
    }
    assert_eq!(repos.calls("cargo"), Vec::<serde_json::Value>::new());
    repos.assert_nothing_offered(&tip);
    assert!(!repos.person.join(".git/FETCH_HEAD").exists());
}
