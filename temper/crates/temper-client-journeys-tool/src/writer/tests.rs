//! The writer's decisions (temper/docs/design/client/journey-writer.md, 7), on exact host requests.
use crate::{
    host::{Answer, Ask, Gathered, Host, Program, Ran, Step, Stream},
    table::TABLE,
};
use skein_shell::Front;
use std::path::Path;

const CHECKOUT: &str = "/writer";
const OFFER: &str = "/writer/.git/journey-pages";
const LOCK: &str = "/writer/.git/journey-pages.lock";
const PID_FILE: &str = "/writer/.git/journey-pages.lock/pid";
const PATHS: &str = "/writer/.git/journey-pages.lock/paths";
const MESSAGE: &str = "/writer/.git/journey-pages.lock/message";
const PAGE: &str = "temper/docs/design/client/web/journeys/welcome/01.html";
const README: &str = "temper/docs/design/client/web/journeys/welcome/README.md";
const RECORDING: &str = "temper/docs/design/client/web/journeys/welcome/recording";
const TIP: &str = "0123456789abcdef0123456789abcdef01234567";
const COMMIT: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const NEXT: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const OTHER: &str = "cccccccccccccccccccccccccccccccccccccccc";
const SUBJECT: &str = "Journey pages printed at 0123456789ab";
const BRANCH: &str = "journey-pages";
const ALLOWED: &[&str] = &["--as-allowed"];
const BUDGET: &[&str] = &["--budget", "25"];
const PROJECT: &[&str] = &["--project", "P", "--as-allowed"];
const STATES: &[&str] = &["rebase-merge", "rebase-apply", "MERGE_HEAD", "CHERRY_PICK_HEAD", "sequencer"];
const WRITER_PID: u32 = 99;
const HOLDER_PID: u32 = 314;
const PRINTED: &str = "printed welcome\n";
const FRESH: &str = "fresh welcome\n";

#[derive(Default)]
struct Script {
    steps: Vec<Step>,
}

impl Script {
    fn ask(&mut self, ask: Ask, answer: Answer) {
        self.steps.push(Step { ask, answer });
    }

    fn git(&mut self, arguments: &[&str], stdout: &str) {
        self.command("git", arguments, 0, stdout, Stream::Keep);
    }

    fn command(&mut self, name: &str, arguments: &[&str], code: i32, stdout: &str, stream: Stream) {
        let mut program = program(name, arguments);
        program.directory = Some(CHECKOUT.into());
        program.stdout = stream;
        self.ask(Ask::Run(program), Answer::Ran(Ran::new(Some(code), stdout.as_bytes())));
    }

    fn git_path(&mut self, name: &str) {
        self.git(&["rev-parse", "--git-path", name], &format!(".git/{name}\n"));
    }

    fn write(&mut self, path: &str, contents: &str) {
        self.ask(Ask::Write(path.into(), contents.as_bytes().into()), Answer::Done);
    }

    fn remove(&mut self, path: &str) {
        self.ask(Ask::Remove(path.into()), Answer::Done);
    }

    fn start(&mut self, branch: &str, remote: Option<&str>) {
        self.steps.push(quiet(&["check-ref-format", "--branch", branch], 0));
        if let Some(remote) = remote {
            self.steps.push(quiet(&["remote", "get-url", remote], 0));
        }
        self.ask(
            Ask::Run(Program::new(b"git", &[b"rev-parse", b"--show-toplevel"])),
            Answer::Ran(Ran::new(Some(0), b"/writer\n")),
        );
        self.git_path("journey-pages");
        self.git_path("journey-pages.lock");
    }

    fn take_lock(&mut self) {
        self.ask(Ask::Directory(LOCK.into()), Answer::Done);
        self.write(PID_FILE, &format!("{WRITER_PID}\n"));
    }

    fn locked(branch: &str, remote: Option<&str>) -> Self {
        let mut script = Self::default();
        script.start(branch, remote);
        script.take_lock();
        script
    }

    fn fresh(branch: &str, remote: Option<&str>) -> Self {
        let mut script = Self::locked(branch, remote);
        script.ask(Ask::Exists(OFFER.into()), Answer::Is(false));
        script
    }

    fn print_on(branch: &str, remote: Option<&str>, tip: &str, code: i32, lines: &str) -> Self {
        let mut script = Self::fresh(branch, remote);
        script.checkout(branch, tip);
        script.print(code, lines);
        script
    }

    fn kept(&mut self, offer: &str) {
        self.ask(Ask::Exists(OFFER.into()), Answer::Is(true));
        self.ask(Ask::Read(OFFER.into()), Answer::Bytes(offer.as_bytes().into()));
    }

    fn task_show(&mut self, doc: &str) {
        self.command("temper", &["ctl", "task", "show", "42", "--json"], 0, doc, Stream::Keep);
    }

    fn states(&mut self, in_progress: Option<&str>) {
        for &state in STATES {
            self.git_path(state);
            let exists = Some(state) == in_progress;
            self.ask(Ask::Exists(Path::new(CHECKOUT).join(".git").join(state)), Answer::Is(exists));
            if exists {
                break;
            }
        }
    }

    fn fetch(&mut self) {
        self.command("git", &["fetch", "origin", "main"], 0, "", Stream::Err);
    }

    fn checkout(&mut self, branch: &str, tip: &str) {
        self.states(None);
        self.git(&["status", "--porcelain", "--untracked-files=no"], "");
        self.fetch();
        // The commit's parent is the fetched origin/main, never the pre-fetch HEAD.
        self.command("git", &["checkout", "-B", branch, "origin/main"], 0, "", Stream::Err);
        self.git(&["rev-parse", "HEAD"], &format!("{tip}\n"));
    }

    fn print(&mut self, code: i32, lines: &str) {
        let mut program = program("cargo", &["run", "--locked", "-p", "temper-client-walk", "--", "pages"]);
        program.directory = Some(CHECKOUT.into());
        program.environment.push((b"TEMPER_INTERNAL_BUNDLE".as_slice().into(), b"none".as_slice().into()));
        program.stdout = Stream::Out;
        self.ask(Ask::Run(program), Answer::Ran(Ran::new(Some(code), lines.as_bytes())));
    }

    fn status(&mut self, status: &str) {
        self.git(&["status", "--porcelain", "-z", "--untracked-files=all", "--no-renames"], status);
    }

    fn commit(&mut self, paths: &str, body: &str, head: &str) {
        self.write(PATHS, paths);
        self.write(MESSAGE, &format!("{SUBJECT}\n\n{body}\n"));
        self.command(
            "git",
            &["--literal-pathspecs", "add", "-A", &format!("--pathspec-from-file={PATHS}"), "--pathspec-file-nul"],
            0,
            "",
            Stream::Err,
        );
        self.command("git", &["-c", "commit.cleanup=verbatim", "commit", "-F", MESSAGE], 0, "", Stream::Err);
        self.git(&["rev-parse", "HEAD"], &format!("{head}\n"));
    }

    fn ls_remote(&mut self, remote: &str, branch: &str, head: Option<&str>) {
        let reference = format!("refs/heads/{branch}");
        let answer = head.map_or_else(String::new, |head| format!("{head}\t{reference}\n"));
        self.git(&["ls-remote", remote, &reference], &answer);
    }

    fn push(&mut self, remote: &str, branch: &str, force: bool) {
        let reference = format!("HEAD:refs/heads/{branch}");
        let arguments =
            if force { vec!["push", "--force", remote, &reference] } else { vec!["push", remote, &reference] };
        self.command("git", &arguments, 0, "", Stream::Err);
    }

    fn open(&mut self, branch: &str, head: &str, body: &str, budget: &[&str]) {
        let mut arguments = vec![
            "ctl",
            "change",
            "open",
            "--project",
            "P",
            "--branch",
            branch,
            "--at",
            head,
            "--title",
            SUBJECT,
            "--body",
            body,
        ];
        arguments.extend_from_slice(budget);
        let key = format!("journey-pages-{TIP}");
        arguments.extend_from_slice(&["--key", &key]);
        self.command("temper", &arguments, 0, "task 42\n", Stream::Keep);
        self.write(OFFER, "task 42\n");
    }

    fn offer_page(&mut self, head: &str) {
        self.status(&format!(" M {PAGE}\0"));
        let body = format!("Printed: {TIP}");
        self.commit(&format!("{PAGE}\0"), &body, head);
        self.push("origin", BRANCH, true);
        self.open(BRANCH, head, &body, ALLOWED);
    }

    fn kept_commit_lookup(&mut self, reachable: bool) {
        self.kept(&format!("commit {COMMIT}\n"));
        let mut cat = program("git", &["cat-file", "-e", &format!("{COMMIT}^{{commit}}")]);
        cat.directory = Some(CHECKOUT.into());
        cat.stderr = Stream::Keep;
        self.ask(Ask::Run(cat), Answer::Ran(Ran::new(Some(0), b"")));
        self.git(&["rev-parse", &format!("{COMMIT}^")], &format!("{TIP}\n"));
        self.fetch();
        self.command(
            "git",
            &["merge-base", "--is-ancestor", COMMIT, "origin/main"],
            i32::from(!reachable),
            "",
            Stream::Keep,
        );
    }

    fn locked_run(mut self, arguments: &[&str], code: u8, stdout: &str, stderr: &str) {
        self.remove(LOCK);
        self.run(arguments, code, stdout, stderr);
    }

    fn run(self, arguments: &[&str], code: u8, stdout: &str, stderr: &str) {
        let mut host = Host::canned(self.steps);
        assert_eq!(super::run(&words(arguments), &mut host, WRITER_PID), code, "{arguments:?}");
        assert_eq!(
            host.finish(),
            Gathered { stdout: stdout.as_bytes().to_vec(), stderr: stderr.as_bytes().to_vec() },
            "{arguments:?}"
        );
    }
}

fn words(words: &[&str]) -> Vec<Box<[u8]>> {
    words.iter().map(|word| word.as_bytes().into()).collect()
}

fn program(name: &str, arguments: &[&str]) -> Program {
    let words: Vec<_> = arguments.iter().map(|word| word.as_bytes()).collect();
    Program::new(name.as_bytes(), &words)
}

fn quiet(arguments: &[&str], code: i32) -> Step {
    let mut program = program("git", arguments);
    program.stderr = Stream::Keep;
    Step { ask: Ask::Run(program), answer: Answer::Ran(Ran::new(Some(code), b"")) }
}

fn warning(option: &str) -> String {
    format!("journey-pages.sh: {option} has no effect: temper-client-walk pages walks no box\n")
}

#[test]
fn unchanged_pages_start_no_change() {
    let lines = "fresh welcome\nstopped blocked: still stale after its walk\n";
    for (arguments, status) in [(PROJECT, 0), (&[][..], 1)] {
        let mut script = Script::print_on(BRANCH, None, TIP, i32::from(status), lines);
        script.status("");
        script.locked_run(arguments, status, &format!("{lines}main's pages are {TIP}'s print\n"), "");
    }
}

#[test]
fn one_walk_print_commits_removed_recording_and_stopped_report() {
    for status in [0, 1] {
        let report = if status == 0 { "" } else { "stopped blocked: line 2: missing mark" };
        let lines = format!("{PRINTED}{report}\n");
        let body = if report.is_empty() { format!("Printed: {TIP}") } else { format!("{report}\n\nPrinted: {TIP}") };
        let mut script = Script::print_on(BRANCH, None, TIP, i32::from(status), &lines);
        script.status(&format!(" M {PAGE}\0 D {RECORDING}\0"));
        script.commit(&format!("{PAGE}\0{RECORDING}\0"), &body, COMMIT);
        script.push("origin", BRANCH, true);
        script.open(BRANCH, COMMIT, &body, ALLOWED);
        script.locked_run(PROJECT, status, &format!("{lines}wrote {BRANCH} {COMMIT}\n"), "");
    }
}

#[test]
fn incomplete_walk_print_offers_nothing_with_or_without_a_walk() {
    // No request inspects the roots or their walks before the one print, on either fetched tip.
    for tip in [TIP, NEXT] {
        for status in [2, 3] {
            let lines = "stopped welcome: unreadable root\n";
            let script = Script::print_on(BRANCH, None, tip, status, lines);
            script.locked_run(PROJECT, 1, lines, "");
        }
    }
}

#[test]
fn changed_pages_offer_the_fetched_tip_with_the_given_budget() {
    for budget in [ALLOWED, BUDGET] {
        let mut args = vec!["--project", "P"];
        args.extend_from_slice(budget);
        args.extend_from_slice(&["--prepared", "prepared inputs"]);
        let lines = "printed welcome\nfresh other\n";
        let mut script = Script::print_on(BRANCH, None, TIP, 0, lines);
        script.status(&format!(" M {PAGE}\0"));
        let body = format!("Printed: {TIP}");
        script.commit(&format!("{PAGE}\0"), &body, COMMIT);
        script.push("origin", BRANCH, true);
        script.open(BRANCH, COMMIT, &body, budget);
        script.locked_run(&args, 0, &format!("{lines}wrote {BRANCH} {COMMIT}\n"), &warning("--prepared"));
    }
}

#[test]
fn a_live_change_waits_and_every_ended_phase_releases_it() {
    let mut first = Script::print_on(BRANCH, None, TIP, 0, PRINTED);
    first.offer_page(COMMIT);
    first.locked_run(PROJECT, 0, &format!("{PRINTED}wrote {BRANCH} {COMMIT}\n"), "");

    let mut waiting = Script::locked(BRANCH, None);
    waiting.kept("task 42\n");
    waiting.task_show(r#"{"task":{"chip":{"task":42,"phase":"running"}}}"#);
    waiting.locked_run(PROJECT, 0, "waiting on change 42\n", "");

    for (phase, head) in [("ended", NEXT), ("done", OTHER), ("failed", NEXT), ("cancelled", OTHER), ("ended", "")] {
        let mut script = Script::locked(BRANCH, None);
        script.kept("task 42\n");
        script.task_show(&format!(r#"{{"task":{{"chip":{{"task":42,"phase":"{phase}"}}}}}}"#));
        script.remove(OFFER);
        script.checkout(BRANCH, TIP);
        if head.is_empty() {
            script.print(0, "");
            script.status("");
            script.locked_run(PROJECT, 0, &format!("main's pages are {TIP}'s print\n"), "");
        } else {
            script.print(0, PRINTED);
            script.offer_page(head);
            script.locked_run(PROJECT, 0, &format!("{PRINTED}wrote {BRANCH} {head}\n"), "");
        }
    }
}

#[test]
fn an_unreadable_task_document_starts_nothing() {
    for doc in ["not json", "{}", r#"{"task":{"chip":{"phase":"unfamiliar"}}}"#] {
        let mut script = Script::locked(BRANCH, None);
        script.kept("task 42\n");
        script.task_show(doc);
        script.locked_run(PROJECT, 1, "", "journey-pages: cannot read the task document's phase\n");
    }
}

#[test]
fn paths_outside_the_written_set_are_never_offered() {
    for path in [
        README,
        "outside",
        "temper/docs/design/client/web/journeys/welcome/1.html",
        "temper/docs/design/client/web/journeys/welcome/nested/01.html",
        "temper/docs/design/client/web/journeys/welcome/01.txt",
        "temper/docs/design/client/web/journeys/theme.css",
        "temper/docs/design/client/tui/journeys/skein-ui.css",
    ] {
        let mut script = Script::print_on(BRANCH, None, TIP, 0, "");
        script.status(&format!(" M {PAGE}\0?? {path}\0"));
        script.locked_run(PROJECT, 1, "", &format!("journey-pages: outside the written files: b'{path}'\n"));
    }
}

#[test]
fn written_additions_and_deletions_are_committed_together() {
    let tui = "temper/docs/design/client/tui/journeys/a space/100.txt";
    let css = "temper/docs/design/client/web/journeys/skein-ui.css";
    let web = "temper/docs/design/client/web/journeys/line\nbreak/02.html";
    let mut script = Script::print_on(BRANCH, None, TIP, 0, "");
    script.status(&format!(" D {PAGE}\0?? {tui}\0?? {web}\0?? {css}\0"));
    let body = format!("Printed: {TIP}");
    script.commit(&format!("{PAGE}\0{tui}\0{web}\0{css}\0"), &body, COMMIT);
    script.push("origin", BRANCH, true);
    script.open(BRANCH, COMMIT, &body, ALLOWED);
    script.locked_run(PROJECT, 0, &format!("wrote {BRANCH} {COMMIT}\n"), "");
}

#[test]
fn a_stopped_walk_offers_other_pages_but_an_incomplete_print_does_not() {
    let report = "stopped blocked: line 7: missing control";
    let lines = format!("fresh current\n{PRINTED}{report}; scratch kept at /private/scratch\n");
    let mut script = Script::print_on(BRANCH, None, TIP, 1, &lines);
    script.status(&format!(" M {PAGE}\0"));
    let body = format!("{report}\n\nPrinted: {TIP}");
    script.commit(&format!("{PAGE}\0"), &body, COMMIT);
    script.push("origin", BRANCH, true);
    script.open(BRANCH, COMMIT, &body, ALLOWED);
    script.locked_run(PROJECT, 1, &format!("{lines}wrote {BRANCH} {COMMIT}\n"), "");

    for status in [2, 3] {
        let lines = "stopped blocked: unreadable root\n";
        // No status request after an incomplete print, even if the child wrote a page.
        let script = Script::print_on(BRANCH, None, TIP, status, lines);
        script.locked_run(PROJECT, 1, lines, "");
    }
}

#[test]
fn unsuitable_checkouts_are_refused_before_starting() {
    for dirty in [" M", "M "] {
        let mut script = Script::fresh(BRANCH, None);
        script.states(None);
        script.git(&["status", "--porcelain", "--untracked-files=no"], &format!("{dirty} {PAGE}\n"));
        script.locked_run(PROJECT, 2, "", "journey-pages: tracked files must be clean\n");
    }
    for &state in STATES {
        let mut script = Script::fresh(BRANCH, None);
        script.states(Some(state));
        script.locked_run(PROJECT, 2, "", &format!("journey-pages: {state} is in progress\n"));
    }
}

#[test]
fn refused_arguments_start_nothing() {
    let cases: &[(&[&str], &str)] = &[
        (&["--unknown"], "unknown argument: --unknown"),
        (&["--budget", "25"], "--budget and --as-allowed need --project"),
        (&["--within", "90"], "--within needs digits then s, m or h"),
        (&["--within", "1.5h"], "--within needs digits then s, m or h"),
        (&["--within", "-1m"], "--within needs digits then s, m or h"),
        (&["--within", ""], "--within needs a value"),
        (&["--project", "P", "--as-allowed", "--project", "P"], "repeated --project"),
        (&["--project", "P", "--budget", "25", "--budget", "25"], "repeated --budget"),
        (&["--project", "P", "--as-allowed", "--as-allowed"], "repeated --as-allowed"),
        (&["--prepared", "inputs", "--prepared", "inputs"], "repeated --prepared"),
        (&["--branch", "pages", "--branch", "pages"], "repeated --branch"),
        (&["--push", "origin", "--push", "origin"], "repeated --push"),
        (&["--within", "90m", "--within", "90m"], "repeated --within"),
        (&["--project"], "--project needs a value"),
        (&["--branch"], "--branch needs a value"),
        (&["--push"], "--push needs a value"),
        (&["--within"], "--within needs a value"),
        (&["--project", "P", "--budget", "25", "--as-allowed"], "use --budget or --as-allowed, not both"),
        (&["--project", "P"], "needs --budget AMOUNT or --as-allowed"),
        (&["--as-allowed"], "--budget and --as-allowed need --project"),
        (&["--project", "P", "--budget"], "--budget needs a value"),
        (&["--project", "P", "--as-allowed", "--prepared"], "--prepared needs a value"),
    ];
    for &(arguments, error) in cases {
        Script::default().run(arguments, 2, "", &format!("journey-pages: {error}\n"));
    }
    Script { steps: vec![quiet(&["check-ref-format", "--branch", "bad..name"], 1)] }.run(
        &["--branch", "bad..name"],
        2,
        "",
        "journey-pages: invalid branch: bad..name\n",
    );
    Script {
        steps: vec![quiet(&["check-ref-format", "--branch", BRANCH], 0), quiet(&["remote", "get-url", "absent"], 2)],
    }
    .run(&["--push", "absent"], 2, "", "journey-pages: not a configured remote: absent\n");
}

#[test]
fn the_project_pushes_to_the_named_remote_and_opens_the_named_branch() {
    for branch in [BRANCH, "my-pages"] {
        let mut arguments = vec!["--project", "P", "--budget", "25", "--push", "R"];
        let named = branch != BRANCH;
        if named {
            arguments.extend_from_slice(&["--branch", branch]);
        }
        let mut script = Script::print_on(branch, Some("R"), TIP, 0, "");
        script.status(&format!(" M {PAGE}\0"));
        let body = format!("Printed: {TIP}");
        script.commit(&format!("{PAGE}\0"), &body, COMMIT);
        if named {
            script.ls_remote("R", branch, None);
        }
        script.push("R", branch, !named);
        script.open(branch, COMMIT, &body, BUDGET);
        script.locked_run(&arguments, 0, &format!("wrote {branch} {COMMIT}\n"), "");
    }
}

#[test]
fn without_a_project_the_commit_is_local_unless_a_push_is_asked() {
    for (branch, remote, args) in
        [(BRANCH, None, &[][..]), ("my-pages", Some("origin"), &["--branch", "my-pages", "--push", "origin"][..])]
    {
        let mut script = Script::print_on(branch, remote, TIP, 0, FRESH);
        script.status(&format!(" M {PAGE}\0"));
        script.commit(&format!("{PAGE}\0"), &format!("Printed: {TIP}"), COMMIT);
        if let Some(remote) = remote {
            script.ls_remote(remote, branch, None);
            script.push(remote, branch, false);
        }
        script.write(OFFER, &format!("commit {COMMIT}\n"));
        script.locked_run(args, 0, &format!("{FRESH}wrote {branch} {COMMIT}\n"), "");
    }
}

#[test]
fn only_the_projects_default_branch_can_overwrite_another_commit() {
    for (remote, branch, arguments) in [
        ("origin", BRANCH, &["--push", "origin"][..]),
        ("R", "my-pages", &["--push", "R", "--project", "P", "--as-allowed", "--branch", "my-pages"][..]),
    ] {
        for remote_head in [TIP, OTHER] {
            let mut script = Script::print_on(branch, Some(remote), TIP, 0, "");
            script.status(&format!(" M {PAGE}\0"));
            script.commit(&format!("{PAGE}\0"), &format!("Printed: {TIP}"), COMMIT);
            script.ls_remote(remote, branch, Some(remote_head));
            script.locked_run(arguments, 1, "", &format!("journey-pages: {remote} holds {branch} at {remote_head}\n"));
        }
    }
}

#[test]
fn a_kept_commit_waits_until_main_holds_that_print_merged_or_squashed() {
    let mut first = Script::print_on(BRANCH, None, TIP, 0, "");
    first.status(&format!(" M {PAGE}\0"));
    first.commit(&format!("{PAGE}\0"), &format!("Printed: {TIP}"), COMMIT);
    first.write(OFFER, &format!("commit {COMMIT}\n"));
    first.locked_run(&[], 0, &format!("wrote {BRANCH} {COMMIT}\n"), "");

    let mut waiting = Script::locked(BRANCH, None);
    waiting.kept_commit_lookup(false);
    waiting.git(&["log", "--format=%B", "origin/main"], &format!("Other print\n\nPrinted: {COMMIT}\n"));
    // Fetches main, but never checks whether tracked files are dirty while waiting.
    waiting.locked_run(&[], 0, &format!("waiting on {COMMIT}\n"), "");

    for squash in [false, true] {
        let mut script = Script::locked(BRANCH, None);
        script.kept_commit_lookup(!squash);
        if squash {
            script.git(
                &["log", "--format=%B", "origin/main"],
                &format!("Squashed work\n\nAnother paragraph\n\nPrinted: {TIP}\n\nA final paragraph\n"),
            );
        }
        script.remove(OFFER);
        script.checkout(BRANCH, NEXT);
        script.print(0, FRESH);
        script.status("");
        script.locked_run(&[], 0, &format!("{FRESH}main's pages are {NEXT}'s print\n"), "");
    }
}

#[test]
fn unreadable_offers_and_missing_commits_are_kept_and_start_nothing() {
    for offer in ["42\n", "task 42\nextra\n", "task x\n", "commit bad\n"] {
        let mut script = Script::locked(BRANCH, None);
        script.kept(offer);
        script.locked_run(&[], 1, "", "journey-pages: cannot read kept offer in .git/journey-pages\n");
    }
    let mut script = Script::locked(BRANCH, None);
    script.kept(&format!("commit {COMMIT}\n"));
    let mut cat = program("git", &["cat-file", "-e", &format!("{COMMIT}^{{commit}}")]);
    cat.directory = Some(CHECKOUT.into());
    cat.stderr = Stream::Keep;
    script.ask(Ask::Run(cat), Answer::Ran(Ran::new(Some(1), b"")));
    script.locked_run(&[], 1, "", &format!("journey-pages: kept commit {COMMIT} is missing (.git/journey-pages)\n"));
}

#[test]
fn a_live_lock_is_left_alone_and_a_gone_process_is_replaced() {
    for live in [true, false] {
        let mut script = Script::default();
        script.start(BRANCH, None);
        script.ask(Ask::Directory(LOCK.into()), Answer::Failed("already exists".into()));
        script.ask(Ask::Read(PID_FILE.into()), Answer::Bytes(format!("{HOLDER_PID}\n").into_bytes().into()));
        script.ask(Ask::Alive(HOLDER_PID), Answer::Is(live));
        if live {
            script.run(&[], 2, "", "journey-pages: lock .git/journey-pages.lock held by process 314\n");
        } else {
            script.remove(LOCK);
            script.take_lock();
            script.ask(Ask::Exists(OFFER.into()), Answer::Is(false));
            script.checkout(BRANCH, TIP);
            script.print(0, "");
            script.status("");
            script.locked_run(&[], 0, &format!("main's pages are {TIP}'s print\n"), "");
        }
    }
}

#[test]
fn prepared_and_within_are_validated_but_affect_neither_print_nor_commit() {
    for arguments in
        [&[][..], &["--prepared", "prepared inputs"], &["--within", "90m"], &["--prepared", "inputs", "--within", "0s"]]
    {
        let mut script = Script::print_on(BRANCH, None, TIP, 0, PRINTED);
        script.status(&format!(" M {PAGE}\0"));
        // The same parent, paths, and message bytes are requested in all four rows.
        script.commit(&format!("{PAGE}\0"), &format!("Printed: {TIP}"), COMMIT);
        script.write(OFFER, &format!("commit {COMMIT}\n"));
        let expected: String =
            ["--prepared", "--within"].into_iter().filter(|option| arguments.contains(option)).map(warning).collect();
        script.locked_run(arguments, 0, &format!("{PRINTED}wrote {BRANCH} {COMMIT}\n"), &expected);
    }
}

#[test]
fn help_names_the_ignored_options_without_starting() {
    let args = words(&["writer", "--help"]);
    let Front::Print(page) = TABLE.front(&args) else { panic!("help must print without dispatching") };
    let joined = page.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(joined.contains("--prepared DIR and --within DURATION have no effect"), "{page}");
}
