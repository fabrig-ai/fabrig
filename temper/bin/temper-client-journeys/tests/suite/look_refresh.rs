//! The look's requests use real git, exclusively in each story's scratch.
//! Run the worktree's script by absolute path, from the person's clone.
use skein_scratch::Scratch;
use std::{
    fs,
    os::unix::fs::PermissionsExt as _,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const JOURNEYS: &str = "docs/design/client/web/journeys";
const PAGE: &str = "docs/design/client/web/journeys/welcome-playground/01.html";
const OTHER_PAGE: &str = "docs/design/client/web/journeys/welcome-playground/02.html";
const OVERLAY: &str = "docs/design/client/web/journeys/overlay.css";

struct Repositories {
    scratch: Scratch,
    person: PathBuf,
    origin: PathBuf,
    branch: &'static str,
}

impl Repositories {
    fn new(name: &str) -> Self {
        Self::on(name, "look-requests")
    }

    fn on(name: &str, branch: &'static str) -> Self {
        let scratch = Scratch::new(name);
        let person = scratch.path().join("person");
        let origin = scratch.path().join("origin.git");
        fs::create_dir(scratch.path().join("home")).expect("isolated home");
        fs::create_dir(scratch.path().join("hooks")).expect("empty hooks");
        fs::write(scratch.path().join("gitconfig"), "").expect("empty global configuration");
        let bin = scratch.path().join("bin");
        fs::create_dir(&bin).expect("scratch programs");
        let cargo = bin.join("cargo");
        fs::write(
            &cargo,
            br#"#!/bin/sh
[ "$#" -ge 9 ] && [ "$1" = run ] && [ "$2" = --locked ] && [ "$3" = --quiet ] &&
    [ "$4" = --manifest-path ] && [ "$5" = "$LOOK_REFRESH_MANIFEST" ] &&
    [ "$6" = -p ] && [ "$7" = temper-client-journeys ] && [ "$8" = -- ] || exit 99
shift 8
exec "$LOOK_REFRESH_BINARY" "$@"
"#,
        )
        .expect("cargo stand-in");
        fs::set_permissions(cargo, fs::Permissions::from_mode(0o700)).expect("executable cargo stand-in");
        let repos = Self { scratch, person, origin, branch };
        repos.git(repos.scratch.path(), &["init", "--bare", "--quiet", "origin.git"]);
        repos.git(repos.scratch.path(), &["clone", "--quiet", "origin.git", "person"]);
        fs::create_dir_all(repos.person.join(JOURNEYS).join("welcome-playground")).expect("journey directory");
        repos.write(PAGE, "<p>Cards</p>\n");
        repos.write(OTHER_PAGE, "<p>Banner</p>\n");
        // Blank context separates independent overlay edits in this fixture.
        repos.write(OVERLAY, "\n\n\n\n\n\n\n\n");
        repos.commit("The printed journey");
        repos.git(&repos.person, &["push", "--quiet", "origin", "main"]);
        repos.git(&repos.person, &["checkout", "--quiet", "-b", repos.branch]);
        repos
    }

    #[expect(clippy::disallowed_methods, reason = "cli C4: PATH, through temper's test table once it exists")]
    fn command(&self, program: &str, directory: &Path) -> Command {
        let original = std::env::var_os("PATH").expect("git and sh on PATH");
        let path = std::env::join_paths(
            std::iter::once(self.scratch.path().join("bin")).chain(std::env::split_paths(&original)),
        )
        .expect("prepend cargo stand-in to PATH");
        let mut command = Command::new(program);
        command
            .current_dir(directory)
            .env_clear()
            .env("PATH", path)
            .env("HOME", self.scratch.path().join("home"))
            .env("XDG_CONFIG_HOME", self.scratch.path().join("home"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.scratch.path().join("gitconfig"))
            .env("GIT_AUTHOR_NAME", "Look Person")
            .env("GIT_AUTHOR_EMAIL", "look@example.invalid")
            .env("GIT_COMMITTER_NAME", "Look Keeper")
            .env("GIT_COMMITTER_EMAIL", "keeper@example.invalid")
            .env("GIT_AUTHOR_DATE", "2026-01-01T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-01-02T00:00:00Z")
            .env("GIT_EDITOR", "true")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("LC_ALL", "C")
            .env("TZ", "UTC")
            .env("TMPDIR", self.scratch.path());
        // Also inherited by the script's git children; no machine hooks or
        // signing, line-ending conversion or default-branch setting applies.
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
        let output = self
            .command("git", directory)
            .args(["-c", "core.abbrev=7", "-c", "maintenance.auto=false"])
            .args(arguments)
            .output()
            .expect("run git in scratch");
        assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout).expect("git output is text").trim_end().to_owned()
    }

    fn write(&self, path: &str, contents: &str) {
        fs::write(self.person.join(path), contents).expect("write scratch journey");
    }

    fn commit(&self, message: &str) -> String {
        self.git(&self.person, &["add", "."]);
        self.git(&self.person, &["commit", "--quiet", "-m", message]);
        self.git(&self.person, &["rev-parse", "HEAD"])
    }

    fn request(&self, name: &str) -> String {
        let checkout = &self.person;
        // Separate pages/rules let a later request survive an earlier one's
        // removal without depending on the lines that earlier request edits.
        let (page, html, rule) = match name {
            "tighter-cards" => (PAGE, "<p>Tighter cards</p>\n", ".temper-card { padding: 1rem; }\n"),
            "calmer-banner" => (OTHER_PAGE, "<p>Calmer banner</p>\n", ".temper-banner { opacity: .8; }\n"),
            _ => unreachable!("request named by a witness"),
        };
        fs::write(checkout.join(page), html).expect("request page");
        let mut overlay = fs::read_to_string(checkout.join(OVERLAY)).expect("overlay");
        // Leave context between rules so dropping the first request does not
        // itself conflict with the second rule's patch.
        if name == "calmer-banner" {
            overlay.push_str(rule);
        } else {
            overlay.insert_str(0, rule);
        }
        fs::write(checkout.join(OVERLAY), overlay).expect("request rule");
        self.git(checkout, &["add", "."]);
        self.git(checkout, &["commit", "--quiet", "-m", &format!("Tune {name}\n\nLook-request: {name}")]);
        self.git(checkout, &["rev-parse", "HEAD"])
    }

    fn publish_requests(&self) -> String {
        self.git(&self.person, &["push", "--quiet", "origin", self.branch]);
        self.git(&self.origin, &["rev-parse", self.branch])
    }

    fn advance_main(&self, message: &str, page: Option<&str>) -> String {
        let old = self.git(&self.person, &["rev-parse", "origin/main"]);
        self.git(&self.person, &["checkout", "--quiet", "main"]);
        if let Some(page) = page {
            self.write(page, "<p>Reprinted on main</p>\n");
        } else {
            self.write("unrelated", "A main change\n");
        }
        let head = self.commit(message);
        self.git(&self.person, &["push", "--quiet", "origin", "main"]);
        // Restore the tracking ref to model main advancing in another clone.
        self.git(&self.person, &["update-ref", "refs/remotes/origin/main", &old]);
        self.git(&self.person, &["checkout", "--quiet", self.branch]);
        assert_eq!(self.git(&self.person, &["rev-parse", "origin/main"]), old, "stale until fetched");
        head
    }

    fn refresh(&self, status: i32) -> Output {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().expect("source root");
        let output = self
            .command("sh", &self.person)
            .env("LOOK_REFRESH_MANIFEST", root.join("Cargo.toml"))
            .env("LOOK_REFRESH_BINARY", env!("CARGO_BIN_EXE_temper-client-journeys"))
            .arg(root.join("temper/scripts/look-refresh.sh"))
            .output()
            .expect("run look-refresh");
        assert_eq!(
            output.status.code(),
            Some(status),
            "stdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }

    fn assert_pushed(&self) {
        assert_eq!(
            self.git(&self.origin, &["rev-parse", self.branch]),
            self.git(&self.person, &["rev-parse", "HEAD"]),
            "{}",
            self.branch
        );
    }

    fn requests(&self) -> String {
        self.git(&self.person, &["log", "--reverse", "--format=%s", "origin/main..HEAD"])
    }

    fn patch_and_author(&self, commit: &str) -> String {
        self.git(&self.person, &["show", "--format=%an <%ae>%n%aI%n%B", "--no-ext-diff", commit])
    }
}

#[test]
fn the_requests_follow_main() {
    for branch in ["look-requests", "tui-requests"] {
        let repos = Repositories::on(&format!("look-refresh-follow-{branch}"), branch);
        let tighter = repos.request("tighter-cards");
        let calmer = repos.request("calmer-banner");
        let tighter_before = repos.patch_and_author(&tighter);
        let calmer_before = repos.patch_and_author(&calmer);
        repos.publish_requests();
        let main = repos.advance_main("An unrelated change", None);
        let output = repos.refresh(0);
        assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD~2"]), main, "{branch}");
        assert_eq!(repos.requests(), "Tune tighter-cards\nTune calmer-banner", "{branch}");
        assert_eq!(repos.patch_and_author("HEAD~1"), tighter_before, "{branch}");
        assert_eq!(repos.patch_and_author("HEAD"), calmer_before, "{branch}");
        repos.assert_pushed();
        assert!(
            String::from_utf8_lossy(&output.stdout).starts_with(&format!(
                "pages not printed yet\nkept tighter-cards\nkept calmer-banner\npushed {branch} at "
            )),
            "{branch}"
        );
    }
}

#[test]
fn a_conflict_stops_for_the_person() {
    let repos = Repositories::new("look-refresh-conflict");
    repos.request("calmer-banner");
    let remote = repos.publish_requests();
    repos.advance_main("Reprint the banner", Some(OTHER_PAGE));
    let output = repos.refresh(1);
    assert!(String::from_utf8_lossy(&output.stderr).contains("stopped at request calmer-banner"));
    assert!(repos.person.join(".git/rebase-merge").is_dir());
    assert_eq!(repos.git(&repos.person, &["rev-parse", "REBASE_HEAD"]), remote);
    assert_eq!(repos.git(&repos.origin, &["rev-parse", "look-requests"]), remote);
    // The advertised recovery must work after resolving the stopped request.
    repos.write(OTHER_PAGE, "<p>Calmer reprinted banner</p>\n");
    repos.git(&repos.person, &["add", OTHER_PAGE]);
    repos.git(&repos.person, &["rebase", "--continue"]);
    repos.refresh(0);
    assert_eq!(repos.requests(), "Tune calmer-banner");
    repos.assert_pushed();
}
