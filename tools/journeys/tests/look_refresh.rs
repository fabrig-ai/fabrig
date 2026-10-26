//! The look's requests use real git, exclusively in each story's scratch.
//! Run the worktree's script by absolute path, from the person's clone.
use skein_scratch::Scratch;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const JOURNEYS: &str = "docs/design/client/journeys";
const PAGE: &str = "docs/design/client/journeys/welcome-set-up/01.html";
const OTHER_PAGE: &str = "docs/design/client/journeys/welcome-set-up/02.html";
const OVERLAY: &str = "docs/design/client/journeys/overlay.css";

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
        fs::create_dir(scratch.path().join("home")).expect("isolated home");
        fs::create_dir(scratch.path().join("hooks")).expect("empty hooks");
        fs::write(scratch.path().join("gitconfig"), "").expect("empty global configuration");
        let repos = Self { scratch, person, origin };
        repos.git(repos.scratch.path(), &["init", "--bare", "--quiet", "origin.git"]);
        repos.git(repos.scratch.path(), &["clone", "--quiet", "origin.git", "person"]);
        fs::create_dir_all(repos.person.join(JOURNEYS).join("welcome-set-up")).expect("journey directory");
        repos.write(PAGE, "<p>Cards</p>\n");
        repos.write(OTHER_PAGE, "<p>Banner</p>\n");
        // Blank context separates independent overlay edits in this fixture.
        repos.write(OVERLAY, "\n\n\n\n\n\n\n\n");
        repos.commit("The printed journey");
        repos.git(&repos.person, &["push", "--quiet", "origin", "main"]);
        repos.git(&repos.person, &["checkout", "--quiet", "-b", "look-requests"]);
        repos
    }

    fn command(&self, program: &str, directory: &Path) -> Command {
        let mut command = Command::new(program);
        command
            .current_dir(directory)
            .env_clear()
            .env("PATH", std::env::var_os("PATH").expect("git and sh on PATH"))
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
        self.request_in(&self.person, name)
    }

    fn request_in(&self, checkout: &Path, name: &str) -> String {
        // Separate pages/rules let a later request survive an earlier one's
        // removal without depending on the lines that earlier request edits.
        let (page, html, rule) = match name {
            "tighter-cards" => (PAGE, "<p>Tighter cards</p>\n", ".temper-card { padding: 1rem; }\n"),
            "calmer-banner" => (OTHER_PAGE, "<p>Calmer banner</p>\n", ".temper-banner { opacity: .8; }\n"),
            _ => (PAGE, "<p>Third request</p>\n", ".temper-card { padding: .5rem; }\n"),
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
        self.git(&self.person, &["push", "--quiet", "origin", "look-requests"]);
        self.git(&self.origin, &["rev-parse", "look-requests"])
    }

    fn advance_main(&self, message: &str, page: Option<&str>) -> String {
        self.git(&self.person, &["checkout", "--quiet", "main"]);
        if let Some(page) = page {
            self.write(page, "<p>Reprinted on main</p>\n");
        } else {
            self.write("unrelated", "A main change\n");
        }
        let head = self.commit(message);
        // Push by path so the person's origin/main stays stale until the
        // script fetches it, as when main advances in somebody else's clone.
        self.git(&self.person, &["push", "--quiet", self.origin.to_str().expect("scratch path"), "main"]);
        self.git(&self.person, &["checkout", "--quiet", "look-requests"]);
        head
    }

    fn refresh(&self, status: i32) -> Output {
        let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/look-refresh.sh");
        let output = self.command("sh", &self.person).arg(script).output().expect("run look-refresh");
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
            self.git(&self.origin, &["rev-parse", "look-requests"]),
            self.git(&self.person, &["rev-parse", "HEAD"])
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
    let repos = Repositories::new("look-refresh-follow");
    let tighter = repos.request("tighter-cards");
    let calmer = repos.request("calmer-banner");
    let tighter_before = repos.patch_and_author(&tighter);
    let calmer_before = repos.patch_and_author(&calmer);
    repos.publish_requests();
    let main = repos.advance_main("An unrelated change", None);
    let output = repos.refresh(0);
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD~2"]), main);
    assert_eq!(repos.requests(), "Tune tighter-cards\nTune calmer-banner");
    assert_eq!(repos.patch_and_author("HEAD~1"), tighter_before);
    assert_eq!(repos.patch_and_author("HEAD"), calmer_before);
    repos.assert_pushed();
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .starts_with("kept tighter-cards\nkept calmer-banner\npushed look-requests at ")
    );
}

#[test]
fn a_done_request_is_dropped() {
    let repos = Repositories::new("look-refresh-done");
    repos.request("tighter-cards");
    repos.request("calmer-banner");
    repos.publish_requests();
    let main = repos.advance_main("Implement cards\n\nLook-request: tighter-cards", None);
    let output = repos.refresh(0);
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD~1"]), main);
    assert_eq!(repos.requests(), "Tune calmer-banner");
    assert_eq!(fs::read_to_string(repos.person.join(PAGE)).expect("cards page"), "<p>Cards</p>\n");
    repos.assert_pushed();
    let short = repos.git(&repos.person, &["rev-parse", "--short", &main]);
    assert!(String::from_utf8_lossy(&output.stdout).starts_with(&format!(
        "dropped tighter-cards, done on main by {short}\nkept calmer-banner\npushed look-requests at "
    )));
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

#[test]
fn a_branch_moved_elsewhere_is_not_overwritten() {
    let repos = Repositories::new("look-refresh-lease");
    repos.request("tighter-cards");
    repos.publish_requests();
    repos.advance_main("An unrelated change", None);
    repos.git(repos.scratch.path(), &["clone", "--quiet", "--branch", "look-requests", "origin.git", "elsewhere"]);
    let elsewhere = repos.scratch.path().join("elsewhere");
    let remote = repos.request_in(&elsewhere, "smaller-cards");
    repos.git(&elsewhere, &["push", "--quiet", "origin", "look-requests"]);
    let output = repos.refresh(3);
    assert!(String::from_utf8_lossy(&output.stderr).contains("moved elsewhere"));
    assert_eq!(repos.git(&repos.origin, &["rev-parse", "look-requests"]), remote);
    assert_eq!(repos.git(&repos.origin, &["log", "-1", "--format=%s", "look-requests"]), "Tune smaller-cards");
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD~1"]), repos.git(&repos.origin, &["rev-parse", "main"]));
    assert!(!repos.person.join(".git/rebase-merge").exists());
}

#[test]
fn a_first_push_creates_the_branch() {
    let repos = Repositories::new("look-refresh-first");
    repos.request("tighter-cards");
    assert_eq!(repos.git(&repos.origin, &["branch", "--list", "look-requests"]), "");
    assert_eq!(repos.git(&repos.person, &["branch", "--remotes", "--list", "origin/look-requests"]), "");
    repos.refresh(0);
    assert_eq!(repos.requests(), "Tune tighter-cards");
    repos.assert_pushed();
}

#[test]
fn a_request_without_its_trailer_is_refused() {
    let repos = Repositories::new("look-refresh-missing");
    repos.write(PAGE, "<p>Unlabelled request</p>\n");
    let commit = repos.commit("A request with no trailer");
    let remote = repos.publish_requests();
    let output = repos.refresh(2);
    assert!(String::from_utf8_lossy(&output.stderr).contains(&commit));
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD"]), commit);
    assert_eq!(repos.git(&repos.origin, &["rev-parse", "look-requests"]), remote);
    assert_eq!(repos.git(&repos.person, &["status", "--porcelain"]), "");
    assert!(!repos.person.join(".git/rebase-merge").exists());
}

#[test]
fn a_completed_stack_can_be_refreshed_again() {
    let repos = Repositories::new("look-refresh-completed");
    repos.request("tighter-cards");
    repos.request("calmer-banner");
    repos.publish_requests();
    let main = repos.advance_main("Complete both\n\nLook-request: tighter-cards\nLook-request: calmer-banner", None);
    let output = repos.refresh(0);
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD"]), main);
    assert_eq!(String::from_utf8_lossy(&output.stdout).matches("dropped ").count(), 2);
    repos.assert_pushed();
    // No requests remain: an empty todo must still advance/push the branch.
    let next_main = repos.advance_main("Another main change", Some(PAGE));
    repos.refresh(0);
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD"]), next_main);
    assert_eq!(repos.requests(), "");
    repos.assert_pushed();
}

#[test]
fn malformed_requests_are_refused_before_rebasing() {
    let repos = Repositories::new("look-refresh-malformed");
    let original = repos.request("tighter-cards");
    let remote = repos.publish_requests();
    for trailer in [
        "Look-request: Bad_name",
        "Look-request: calmer-banner\nLook-request: extra",
        "Look-request: calmer-banner\nLook-request:",
        "Look-request:\nLook-request: calmer-banner",
        "Look-request: tighter-cards",
    ] {
        repos.write(OTHER_PAGE, "<p>Malformed request</p>\n");
        let commit = repos.commit(&format!("Another request\n\n{trailer}"));
        let output = repos.refresh(2);
        assert!(String::from_utf8_lossy(&output.stderr).contains(&commit));
        assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD"]), commit);
        assert_eq!(repos.git(&repos.origin, &["rev-parse", "look-requests"]), remote);
        assert_eq!(repos.git(&repos.person, &["status", "--porcelain"]), "");
        assert!(!repos.person.join(".git/rebase-merge").exists());
        repos.git(&repos.person, &["reset", "--hard", &original]);
    }
    // A merge is refused even when its message has one valid trailer.
    repos.git(&repos.person, &["checkout", "--quiet", "-b", "side", "main"]);
    repos.write("side", "Side history\n");
    repos.commit("Side history\n\nLook-request: side-look");
    repos.git(&repos.person, &["checkout", "--quiet", "look-requests"]);
    repos.git(&repos.person, &["merge", "--no-ff", "side", "-m", "Merge\n\nLook-request: merged-look"]);
    let merged = repos.git(&repos.person, &["rev-parse", "HEAD"]);
    let output = repos.refresh(2);
    assert!(String::from_utf8_lossy(&output.stderr).contains(&merged));
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD"]), merged);
    assert_eq!(repos.git(&repos.origin, &["rev-parse", "look-requests"]), remote);
}

#[test]
fn an_unsuitable_checkout_is_refused_before_fetching() {
    let repos = Repositories::new("look-refresh-checkout");
    repos.request("tighter-cards");
    let remote = repos.publish_requests();
    let old_main = repos.git(&repos.person, &["rev-parse", "origin/main"]);
    repos.advance_main("Unfetched main change", None);
    let original = repos.git(&repos.person, &["rev-parse", "HEAD"]);
    repos.write(PAGE, "<p>Unsaved change</p>\n");
    repos.refresh(2);
    assert_eq!(repos.git(&repos.person, &["rev-parse", "origin/main"]), old_main);
    assert_eq!(fs::read_to_string(repos.person.join(PAGE)).expect("unsaved page"), "<p>Unsaved change</p>\n");
    repos.git(&repos.person, &["add", PAGE]);
    repos.refresh(2);
    assert_eq!(repos.git(&repos.person, &["rev-parse", "HEAD"]), original);
    repos.git(&repos.person, &["reset", "--hard", &original]);
    // Each state is sufficient to refuse before any fetch or branch change.
    for state in ["rebase-merge", "rebase-apply", "MERGE_HEAD", "CHERRY_PICK_HEAD", "sequencer"] {
        let path = repos.person.join(".git").join(state);
        fs::write(&path, "").expect("in-progress sentinel");
        repos.refresh(2);
        fs::remove_file(path).expect("remove sentinel");
    }
    repos.git(&repos.person, &["checkout", "--quiet", "main"]);
    repos.refresh(2);
    repos.git(&repos.person, &["checkout", "--quiet", "--detach"]);
    repos.refresh(2);
    assert_eq!(repos.git(&repos.person, &["rev-parse", "origin/main"]), old_main);
    assert_eq!(repos.git(&repos.origin, &["rev-parse", "look-requests"]), remote);
}
