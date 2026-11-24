//! The shipped writer script on real git (temper/docs/design/client/journey-writer.md, 7).
//! The fake cargo starts the built writer and supplies the page printer's answer.
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
const TUI_ROOT: &str = "temper/docs/design/client/tui/journeys";

const CARGO: &str = r"#!/usr/bin/env -S python3 -I -B
import json, os, pathlib, sys
root = pathlib.Path(os.environ['WRITER_FAKE'])
package = sys.argv[sys.argv.index('-p') + 1]
if package == 'temper-client-journeys':
    binary = os.environ['WRITER_BINARY']
    os.execv(binary, [binary] + sys.argv[sys.argv.index('--') + 1:])
assert package == 'temper-client-walk', package
with (root / 'print.log').open('a') as log:
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

const TEMPER: &str = r"#!/usr/bin/env -S python3 -I -B
import json, os, pathlib, sys
root = pathlib.Path(os.environ['WRITER_FAKE'])
with (root / 'temper.log').open('a') as log:
    log.write(json.dumps({'args': sys.argv[1:], 'stdin': sys.stdin.read(), 'locale': os.environ.get('LC_ALL')}) + '\n')
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
        fs::create_dir_all(repos.person.join(TUI_ROOT).join("terminal")).expect("TUI directory");

        repos.git(&repos.person, &["add", "."]);
        repos.git(&repos.person, &["commit", "--quiet", "-m", "Main's journeys"]);
        repos.git(&repos.person, &["push", "--quiet", "origin", "main"]);
        repos.print(&[], "", 0);
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
            .env("WRITER_BINARY", env!("CARGO_BIN_EXE_temper-client-journeys"))
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
        fs::write(self.scratch.path().join("print.json"), config.to_string()).expect("engine's answer");
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
        if fs::read_to_string(self.lock().join("pid")).ok().as_deref() != Some(&format!("{}\n", std::process::id())) {
            assert!(!self.lock().exists(), "the run removes its own lock on every exit");
        }
        output
    }

    fn lock(&self) -> PathBuf {
        self.person.join(".git/journey-pages.lock")
    }

    fn occupy(&self, remote: &str, branch: &str) -> String {
        let other = self.git(&self.person, &["commit-tree", "HEAD^{tree}", "-m", "Another owner's commit"]);
        self.git(&self.person, &["push", "--quiet", remote, &format!("{other}:refs/heads/{branch}")]);
        other
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
        assert_eq!(fs::read_to_string(self.task_file()).expect("kept task number"), "task 42\n");
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
        repos.occupy("origin", "journey-pages");
        repos.print(&[(PAGE, Some("<p>Printed</p>\n"))], "printed welcome\nfresh other\n", 0);
        let mut arguments = vec!["--project", "P"];
        arguments.extend_from_slice(&budget);
        arguments.extend_from_slice(&["--prepared", "prepared inputs"]);
        let output = repos.run(&arguments, 0);
        let head = repos.git(&repos.person, &["rev-parse", "HEAD"]);
        assert_eq!(
            String::from_utf8_lossy(&output.stdout),
            format!("printed welcome\nfresh other\nwrote journey-pages {head}\n")
        );
        repos.assert_offered(&tip, "", &budget);
        assert_eq!(repos.git(&repos.origin, &["show", &format!("journey-pages:{PAGE}")]), "<p>Printed</p>");
        assert_eq!(repos.calls("temper").len(), 1, "opened once");
        assert_eq!(
            repos.calls("print"),
            vec![
                serde_json::json!({"args": ["run", "--locked", "-p", "temper-client-walk", "--", "pages"], "bundle": "none"})
            ]
        );
    }
}

#[test]
fn a_live_lock_is_left_alone_and_a_gone_process_is_replaced() {
    let repos = Repositories::new("journey-pages-lock");
    fs::create_dir(repos.lock()).expect("held lock");
    let holder = format!("{}\n", std::process::id());
    fs::write(repos.lock().join("pid"), &holder).expect("lock's process");
    let output = repos.run(&[], 2);
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("journey-pages.lock") && error.contains(holder.trim()), "{error}");
    assert_eq!(output.stdout, b"");
    assert_eq!(repos.calls("print"), Vec::<serde_json::Value>::new());
    assert_eq!(repos.calls("temper"), Vec::<serde_json::Value>::new());
    assert!(!repos.person.join(".git/FETCH_HEAD").exists());
    assert_eq!(fs::read_to_string(repos.lock().join("pid")).expect("lock left alone"), holder);
    // This child has been reaped, so kill -0 cannot reach it.
    let mut child = repos.command("true", &repos.person).spawn().expect("short-lived process");
    let gone = child.id();
    assert!(child.wait().expect("process reaped").success());
    fs::write(repos.lock().join("pid"), format!("{gone}\n")).expect("stale lock");
    repos.run(&[], 0);
    assert_eq!(repos.calls("print").len(), 1);
    assert!(!repos.lock().exists());
}
