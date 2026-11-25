use std::{fmt::Write as _, path::PathBuf};

use crate::host::{Answer, Ask, Gathered, Host, Program, Ran, Step, Stream};

use super::run;

const LOOK: &str = "look-requests";
const TUI: &str = "tui-requests";
const ROOT: &str = "1111111111111111111111111111111111111111";
const DONE: &str = "2222222222222222222222222222222222222222";
const PRINT: &str = "3333333333333333333333333333333333333333";
const LATER: &str = "4444444444444444444444444444444444444444";
const TIP: &str = "5555555555555555555555555555555555555555";
const TIGHTER: &str = "6666666666666666666666666666666666666666";
const CALMER: &str = "7777777777777777777777777777777777777777";
const SIDE: &str = "8888888888888888888888888888888888888888";
const MERGED: &str = "9999999999999999999999999999999999999999";
const REBASED_HEAD: &str = "abcdeffedcba9000000000000000000000000000";
const SHORT: &str = "abcdeff";
const TODO_PATH: &str = "/checkout with spaces/.git/temper-look-refresh.41.todo";
const FETCH: &str = "From origin\n * branch            main       -> FETCH_HEAD\n";
const REBASED: &str = "Successfully rebased.\n";
const NO_PRINT: &str = "pages not printed yet\n";
const TIGHTER_DONE: &str = "kept tighter-cards, done on main by 222222222222, not yet printed\n";
const TIGHTER_DROPPED: &str = "dropped tighter-cards, done on main by 222222222222, printed at 222222222222\n";

#[derive(Default)]
struct Steps(Vec<Step>);

impl Steps {
    fn step(&mut self, ask: Ask, answer: Answer) {
        self.0.push(Step { ask, answer });
    }

    fn git(&mut self, args: &[&str], stdout: &str) {
        self.git_reply(args, (Stream::Keep, Stream::Err), 0, stdout, "");
    }

    fn git_reply(&mut self, args: &[&str], streams: (Stream, Stream), code: i32, stdout: &str, stderr: &str) {
        let mut program = Program::new(b"git", &args.iter().map(|arg| arg.as_bytes()).collect::<Vec<_>>());
        (program.stdout, program.stderr) = streams;
        self.program(program, code, stdout, stderr);
    }

    fn program(&mut self, program: Program, code: i32, stdout: &str, stderr: &str) {
        self.step(
            Ask::Run(program),
            Answer::Ran(Ran { code: Some(code), stdout: stdout.as_bytes().into(), stderr: stderr.as_bytes().into() }),
        );
    }

    fn quiet(&mut self, args: &[&str], stdout: &str, code: i32) {
        self.git_reply(args, (Stream::Keep, Stream::Keep), code, stdout, "");
    }

    fn head(&mut self, branch: &str, status: i32) {
        self.git_reply(
            &["symbolic-ref", "--quiet", "--short", "HEAD"],
            (Stream::Keep, Stream::Err),
            status,
            branch,
            "",
        );
    }

    fn state_paths(&mut self, present: Option<&str>) {
        for state in ["rebase-merge", "rebase-apply", "MERGE_HEAD", "CHERRY_PICK_HEAD", "sequencer"] {
            let path = format!(".git/{state}");
            self.git(&["rev-parse", "--git-path", state], &format!("{path}\n"));
            self.step(Ask::Exists(path.into()), Answer::Is(present == Some(state)));
            if present == Some(state) {
                break;
            }
        }
    }

    fn status(&mut self, porcelain: &str) {
        self.git(&["status", "--porcelain", "--untracked-files=no"], porcelain);
    }

    fn begin(&mut self, branch: &str, lease: &str, commits: &[&str]) {
        self.head(&format!("{branch}\n"), 0);
        self.state_paths(None);
        self.status("");
        let lease_output = if lease.is_empty() { String::new() } else { format!("{lease}\n") };
        self.quiet(
            &["rev-parse", "--verify", &format!("refs/remotes/origin/{branch}")],
            &lease_output,
            if lease.is_empty() { 128 } else { 0 },
        );
        self.git_reply(&["fetch", "origin", "main"], (Stream::Err, Stream::Err), 0, "", FETCH);
        self.git(&["rev-list", "--reverse", &format!("origin/main..{branch}")], &commit_lines(commits));
    }

    fn request(&mut self, hash: &str, parent: &str, trailers: &str) {
        self.git(&["show", "-s", "--format=%P", hash], &format!("{parent}\n"));
        self.git(&["show", "-s", "--format=%(trailers:key=Look-request)", hash], trailers);
    }

    fn tighter(&mut self) {
        self.request(TIGHTER, ROOT, "Look-request: tighter-cards\n\n");
    }

    fn calmer(&mut self) {
        self.request(CALMER, TIGHTER, "Look-request: calmer-banner\n\n");
    }

    fn history(&mut self, commits: &[(&str, &[&str], &[&str])]) {
        self.look_landings("");
        self.legacy_history(commits);
    }

    fn look_landings(&mut self, log: &str) {
        self.git(
            &["log", "--topo-order", "-z", "--format=%H%n%(trailers:key=Look-landed,valueonly,unfold)", "origin/main"],
            log,
        );
    }

    fn verdicts(&mut self, root: &str, entries: Option<&str>) {
        let path = format!("temper/docs/design/client/{root}/journeys/looks.html");
        let tree = entries.map_or_else(String::new, |_| format!("100644 blob {SIDE}\t{path}\0"));
        self.quiet(&["--literal-pathspecs", "ls-tree", "--full-tree", "-z", "origin/main", "--", &path], &tree, 0);
        if let Some(entries) = entries {
            self.quiet(&["cat-file", "blob", SIDE], entries, 0);
        }
    }

    fn legacy_history(&mut self, commits: &[(&str, &[&str], &[&str])]) {
        let mut done = String::new();
        let mut prints = String::new();
        for &(id, requests, printed) in commits {
            writeln!(done, "commit {id}").unwrap();
            let keys = vec!["Printed"; printed.len()].join(",");
            writeln!(prints, "commit {id} {keys}").unwrap();
            for request in requests {
                writeln!(done, "{request}").unwrap();
            }
            for print in printed {
                writeln!(prints, "{print}").unwrap();
            }
            done.push('\n');
            prints.push('\n');
        }
        self.git(&["log", "--format=commit %H%n%(trailers:key=Look-request,valueonly)", "origin/main"], &done);
        self.git(
            &[
                "log",
                "--topo-order",
                "--format=commit %H %(trailers:key=Printed,keyonly,separator=,)%n%(trailers:key=Printed,valueonly)",
                "origin/main",
            ],
            &prints,
        );
    }

    fn ancestor(&mut self, older: &str, newer: &str, code: i32) {
        self.git_reply(&["merge-base", "--is-ancestor", older, newer], (Stream::Keep, Stream::Err), code, "", "");
    }

    fn valid_print(&mut self, printed: &str) {
        self.quiet(&["cat-file", "-t", printed], "commit\n", 0);
        self.ancestor(printed, "origin/main", 0);
    }

    fn first_parent(&mut self, carrier: &str, contains: &[&str], before: &[&str], since: &str) {
        let main_line: Vec<_> = contains.iter().chain(before).copied().collect();
        self.git(&["rev-list", "--first-parent", "origin/main"], &commit_lines(&main_line));
        for landing in contains {
            self.ancestor(carrier, landing, 0);
        }
        if let Some(older) = before.first() {
            self.ancestor(carrier, older, 1);
        }
        self.git(
            &["rev-list", "--first-parent", "--count", &format!("{}..origin/main", contains.last().unwrap())],
            &format!("{since}\n"),
        );
    }

    fn rebase(&mut self, branch: &str, base: &str, todo: &str, code: i32, stderr: &str) {
        self.git(&["merge-base", "origin/main", branch], &format!("{base}\n"));
        self.git(
            &["rev-parse", "--path-format=absolute", "--git-path", "temper-look-refresh.41.todo"],
            &format!("{TODO_PATH}\n"),
        );
        let path = PathBuf::from(TODO_PATH);
        self.step(Ask::Write(path.clone(), todo.as_bytes().into()), Answer::Done);
        let mut rebase = Program::new(
            b"git",
            &[
                b"-c",
                b"rebase.autoStash=false",
                b"-c",
                b"rebase.updateRefs=false",
                b"-c",
                b"rebase.abbreviateCommands=false",
                b"rebase",
                b"--interactive",
                b"--no-autosquash",
                b"--no-autostash",
                b"--no-update-refs",
                b"--reapply-cherry-picks",
                b"--keep-empty",
                b"--empty=keep",
                b"--onto",
                b"origin/main",
                base.as_bytes(),
            ],
        );
        rebase.stdout = Stream::Err;
        rebase.environment = vec![
            (b"LOOK_REFRESH_TODO".as_slice().into(), TODO_PATH.as_bytes().into()),
            (b"GIT_SEQUENCE_EDITOR".as_slice().into(), b"cp \"$LOOK_REFRESH_TODO\"".as_slice().into()),
        ];
        self.program(rebase, code, "", stderr);
        self.step(Ask::Remove(path), Answer::Done);
    }

    fn push(&mut self, branch: &str, lease: &str, response: &str, code: i32) {
        let lease = format!("--force-with-lease={branch}:{lease}");
        let mut push = Program::new(
            b"sh",
            &[
                b"-c",
                b"exec git \"$@\" 2>&1",
                b"sh",
                b"push",
                b"--porcelain",
                lease.as_bytes(),
                b"origin",
                branch.as_bytes(),
            ],
        );
        push.stderr = Stream::Keep;
        self.program(push, code, response, "");
    }

    fn pushed(&mut self, branch: &str, base: &str, todo: &str, lease: &str, head: &str, forced: bool) -> String {
        self.rebase(branch, base, todo, 0, REBASED);
        let (flag, summary) = if lease.is_empty() {
            ('*', "[new branch]".to_owned())
        } else if forced {
            ('+', format!("{lease:.7}...{head:.7} (forced update)"))
        } else {
            (' ', format!("{lease:.7}..{head:.7}"))
        };
        let response = format!("To origin\n{flag}\trefs/heads/{branch}:refs/heads/{branch}\t{summary}\nDone\n");
        self.push(branch, lease, &response, 0);
        self.git(&["rev-parse", "--short", "HEAD"], &format!("{head:.7}\n"));
        response
    }

    fn check(self, code: u8, stdout: &str, stderr: &str) {
        check(self.0, &[], code, stdout, stderr);
    }
}

fn commit_lines(commits: &[&str]) -> String {
    let mut lines = String::new();
    for commit in commits {
        writeln!(lines, "{commit}").unwrap();
    }
    lines
}

fn check(steps: Vec<Step>, args: &[Box<[u8]>], code: u8, stdout: &str, stderr: &str) {
    let mut host = Host::canned(steps);
    let exit = run(args, &mut host, 41);
    let Gathered { stdout: out, stderr: err } = host.finish();
    assert_eq!(exit, code);
    assert_eq!(String::from_utf8(out).unwrap(), stdout);
    assert_eq!(String::from_utf8(err).unwrap(), stderr);
}

fn stack() -> Steps {
    let mut steps = Steps::default();
    steps.begin(LOOK, TIGHTER, &[TIGHTER]);
    steps.tighter();
    steps
}

#[test]
fn the_requests_follow_main() {
    for branch in [LOOK, TUI] {
        let mut steps = Steps::default();
        steps.begin(branch, CALMER, &[TIGHTER, CALMER]);
        steps.tighter();
        steps.calmer();
        steps.history(&[(TIP, &[], &[]), (ROOT, &[], &[])]);
        let push = steps.pushed(branch, ROOT, &format!("pick {TIGHTER}\npick {CALMER}\n"), CALMER, REBASED_HEAD, true);
        steps.check(
            0,
            &format!("{NO_PRINT}kept tighter-cards\nkept calmer-banner\npushed {branch} at {SHORT}\n"),
            &format!("{FETCH}{REBASED}{push}"),
        );
    }
}

#[test]
fn a_done_request_is_dropped() {
    for landings in [0, 2] {
        let mut steps = Steps::default();
        steps.begin(LOOK, CALMER, &[TIGHTER, CALMER]);
        steps.tighter();
        steps.calmer();
        let (printed, tail, since) = if landings == 0 {
            steps.history(&[(PRINT, &[], &[DONE]), (DONE, &["tighter-cards"], &[]), (ROOT, &[], &[])]);
            (DONE, vec![PRINT], vec![DONE, ROOT])
        } else {
            steps.history(&[
                (TIP, &[], &[]),
                (LATER, &[], &[]),
                (PRINT, &[], &[SIDE]),
                (SIDE, &[], &[]),
                (DONE, &["tighter-cards"], &[]),
                (ROOT, &[], &[]),
            ]);
            (SIDE, vec![TIP, LATER, PRINT], vec![SIDE, DONE, ROOT])
        };
        steps.valid_print(printed);
        steps.first_parent(PRINT, &tail, &since, &landings.to_string());
        steps.ancestor(DONE, printed, 0);
        let push = steps.pushed(LOOK, ROOT, &format!("drop {TIGHTER}\npick {CALMER}\n"), CALMER, REBASED_HEAD, true);
        steps.check(
            0, &format!("pages printed at {printed:.12}, {landings} landings since\ndropped tighter-cards, done on main by 222222222222, printed at {printed:.12}\nkept calmer-banner\npushed look-requests at {SHORT}\n"),
            &format!("{FETCH}{REBASED}{push}"),
        );
    }
}

#[test]
fn a_done_request_stays_until_printed() {
    let mut steps = stack();
    steps.history(&[(DONE, &["tighter-cards"], &[]), (ROOT, &[], &[])]);
    let push = steps.pushed(LOOK, ROOT, &format!("pick {TIGHTER}\n"), TIGHTER, REBASED_HEAD, true);
    steps.check(
        0,
        &format!("{NO_PRINT}{TIGHTER_DONE}pushed look-requests at {SHORT}\n"),
        &format!("{FETCH}{REBASED}{push}"),
    );
}

#[test]
fn a_print_before_the_landing_keeps_the_request() {
    let mut steps = stack();
    steps.history(&[(DONE, &["tighter-cards"], &[]), (PRINT, &[], &[ROOT]), (ROOT, &[], &[])]);
    steps.valid_print(ROOT);
    steps.first_parent(PRINT, &[DONE, PRINT], &[ROOT], "1");
    steps.ancestor(DONE, ROOT, 1);
    let push = steps.pushed(LOOK, ROOT, &format!("pick {TIGHTER}\n"), TIGHTER, REBASED_HEAD, true);
    steps.check(
        0,
        &format!("pages printed at 111111111111, 1 landings since\n{TIGHTER_DONE}pushed look-requests at {SHORT}\n"),
        &format!("{FETCH}{REBASED}{push}"),
    );
}

#[test]
fn the_newest_print_decides() {
    for (newer, older, ancestry, verb, line) in
        [(DONE, ROOT, 0, "drop", TIGHTER_DROPPED), (ROOT, DONE, 1, "pick", TIGHTER_DONE)]
    {
        let mut steps = stack();
        steps.history(&[
            (LATER, &[], &[newer]),
            (PRINT, &[], &[older]),
            (DONE, &["tighter-cards"], &[]),
            (ROOT, &[], &[]),
        ]);
        steps.valid_print(newer);
        steps.first_parent(LATER, &[LATER], &[PRINT, DONE, ROOT], "0");
        steps.ancestor(DONE, newer, ancestry);
        let head = if ancestry == 0 { LATER } else { REBASED_HEAD };
        let push = steps.pushed(LOOK, ROOT, &format!("{verb} {TIGHTER}\n"), TIGHTER, head, true);
        steps.check(
            0,
            &format!("pages printed at {newer:.12}, 0 landings since\n{line}pushed look-requests at {head:.7}\n"),
            &format!("{FETCH}{REBASED}{push}"),
        );
    }
}

#[test]
fn invalid_prints_do_not_hide_the_last_valid_print() {
    let mut steps = stack();
    let missing_sha1 = "a".repeat(40);
    let missing_sha256 = "b".repeat(64);
    let blob = "c".repeat(40);
    let invalid = [blob.as_str(), TIGHTER, &missing_sha256, &missing_sha1, "2222222", "not-a-hash", ""];
    let ids: Vec<_> = (1..=7).map(|n| format!("aa{n:x}{:037x}", 0)).collect();
    let values: Vec<_> = invalid.iter().map(|value| [*value]).collect();
    let mut commits: Vec<(&str, &[&str], &[&str])> =
        ids.iter().zip(&values).map(|(id, value)| (id.as_str(), &[][..], &value[..])).collect();
    commits.extend([(PRINT, &[][..], &[DONE][..]), (DONE, &["tighter-cards"][..], &[][..]), (ROOT, &[][..], &[][..])]);
    steps.history(&commits);
    steps.quiet(&["cat-file", "-t", &blob], "blob\n", 0);
    steps.quiet(&["cat-file", "-t", TIGHTER], "commit\n", 0);
    steps.ancestor(TIGHTER, "origin/main", 1);
    steps.quiet(&["cat-file", "-t", &missing_sha256], "", 128);
    steps.quiet(&["cat-file", "-t", &missing_sha1], "", 128);
    steps.valid_print(DONE);
    let contains: Vec<_> = ids.iter().map(String::as_str).chain([PRINT]).collect();
    steps.first_parent(PRINT, &contains, &[DONE, ROOT], "7");
    steps.ancestor(DONE, DONE, 0);
    let push = steps.pushed(LOOK, ROOT, &format!("drop {TIGHTER}\n"), TIGHTER, &ids[0], true);
    let mut stderr = FETCH.to_owned();
    for (carrier, value) in ids.iter().zip(invalid) {
        writeln!(
            stderr,
            "look-refresh: ignoring Printed: {value} in {carrier}; expected a full commit hash in origin/main history"
        )
        .unwrap();
    }
    steps.check(
        0,
        &format!("pages printed at 222222222222, 7 landings since\n{TIGHTER_DROPPED}pushed look-requests at aa10000\n"),
        &format!("{stderr}{REBASED}{push}"),
    );
}

#[test]
fn the_print_lands_through_a_merge() {
    let mut steps = stack();
    steps.history(&[
        (TIP, &[], &[]),
        (LATER, &[], &[]),
        (MERGED, &[], &[]),
        (PRINT, &[], &[DONE]),
        (SIDE, &[], &[]),
        (DONE, &["tighter-cards"], &[]),
        (ROOT, &[], &[]),
    ]);
    steps.valid_print(DONE);
    // PRINT reached main through MERGED, so the first-parent line skips PRINT.
    steps.first_parent(PRINT, &[TIP, LATER, MERGED], &[SIDE, DONE, ROOT], "2");
    steps.ancestor(DONE, DONE, 0);
    let push = steps.pushed(LOOK, ROOT, &format!("drop {TIGHTER}\n"), TIGHTER, TIP, true);
    steps.check(
        0,
        &format!("pages printed at 222222222222, 2 landings since\n{TIGHTER_DROPPED}pushed look-requests at 5555555\n"),
        &format!("{FETCH}{REBASED}{push}"),
    );
}

#[test]
fn the_print_lands_squashed_with_unrelated_work() {
    let mut steps = stack();
    // The squash commit carries Printed itself; there is no separate writer commit.
    steps.history(&[(TIP, &[], &[DONE]), (DONE, &["tighter-cards"], &[]), (ROOT, &[], &[])]);
    steps.valid_print(DONE);
    steps.first_parent(TIP, &[TIP], &[DONE, ROOT], "0");
    steps.ancestor(DONE, DONE, 0);
    let push = steps.pushed(LOOK, ROOT, &format!("drop {TIGHTER}\n"), TIGHTER, TIP, true);
    steps.check(
        0,
        &format!("pages printed at 222222222222, 0 landings since\n{TIGHTER_DROPPED}pushed look-requests at 5555555\n"),
        &format!("{FETCH}{REBASED}{push}"),
    );
}

#[test]
fn a_conflict_stops_for_the_person() {
    let mut steps = Steps::default();
    steps.begin(LOOK, CALMER, &[CALMER]);
    steps.request(CALMER, ROOT, "Look-request: calmer-banner\n\n");
    steps.history(&[(TIP, &[], &[]), (ROOT, &[], &[])]);
    let conflict =
        "CONFLICT (content): Merge conflict in 02.html\nerror: could not apply 7777777... Tune calmer-banner\n";
    steps.rebase(LOOK, ROOT, &format!("pick {CALMER}\n"), 1, conflict);
    steps.quiet(&["rev-parse", "--verify", "REBASE_HEAD"], &format!("{CALMER}\n"), 0);
    // No push or rebase cleanup beyond removing the todo; the person can continue.
    steps.check(1, "", &format!("{FETCH}{conflict}look-refresh: stopped at request calmer-banner; settle it, git rebase --continue, then run this script again; nothing pushed\n"));

    // After the person continues, the resolved request is atop main but the lease is still the old tip.
    let mut steps = Steps::default();
    steps.begin(LOOK, CALMER, &[LATER]);
    steps.request(LATER, TIP, "Look-request: calmer-banner\n\n");
    steps.history(&[(TIP, &[], &[]), (ROOT, &[], &[])]);
    let push = steps.pushed(LOOK, TIP, &format!("pick {LATER}\n"), CALMER, LATER, true);
    steps.check(
        0,
        &format!("{NO_PRINT}kept calmer-banner\npushed look-requests at 4444444\n"),
        &format!("{FETCH}{REBASED}{push}"),
    );
}

#[test]
fn a_branch_moved_elsewhere_is_not_overwritten() {
    let mut steps = stack();
    steps.history(&[(TIP, &[], &[]), (ROOT, &[], &[])]);
    steps.rebase(LOOK, ROOT, &format!("pick {TIGHTER}\n"), 0, REBASED);
    let rejected = "error: failed to push some refs to 'origin'\nTo origin\n!\trefs/heads/look-requests:refs/heads/look-requests\t[rejected] (stale info)\nDone\n";
    steps.push(LOOK, TIGHTER, rejected, 1);
    // No retry, new lease, or reset of the rebased branch.
    steps.check(3, &format!("{NO_PRINT}kept tighter-cards\n"), &format!("{FETCH}{REBASED}{rejected}look-refresh: remote look-requests moved elsewhere; fetch it and settle it by hand before running again; branch left rebased\n"));
}

#[test]
fn a_push_failure_without_a_stale_lease_returns_one() {
    let mut steps = stack();
    steps.history(&[(TIP, &[], &[]), (ROOT, &[], &[])]);
    steps.rebase(LOOK, ROOT, &format!("pick {TIGHTER}\n"), 0, REBASED);
    let rejected = "remote: (stale info) in an unrelated check\nTo origin\n!\trefs/heads/look-requests:refs/heads/look-requests\t[rejected] (non-fast-forward)\nDone\n";
    steps.push(LOOK, TIGHTER, rejected, 1);
    steps.check(1, &format!("{NO_PRINT}kept tighter-cards\n"), &format!("{FETCH}{REBASED}{rejected}"));
}

#[test]
fn a_first_push_creates_the_branch() {
    let mut steps = Steps::default();
    steps.begin(LOOK, "", &[TIGHTER]);
    steps.tighter();
    steps.history(&[(ROOT, &[], &[])]);
    let created = steps.pushed(LOOK, ROOT, &format!("pick {TIGHTER}\n"), "", TIGHTER, false);
    steps.check(
        0,
        &format!("{NO_PRINT}kept tighter-cards\npushed look-requests at 6666666\n"),
        &format!("{FETCH}{REBASED}{created}"),
    );
}

fn bad_trailer(commit: &str) -> String {
    format!("look-refresh: {commit} must carry exactly one Look-request trailer with NAME [a-z0-9][a-z0-9-]*\n")
}

#[test]
fn a_request_without_its_trailer_is_refused() {
    let mut steps = Steps::default();
    steps.begin(LOOK, TIGHTER, &[TIGHTER]);
    steps.request(TIGHTER, ROOT, "\n");
    steps.check(2, "", &format!("{FETCH}{}", bad_trailer(TIGHTER)));
}

#[test]
fn a_completed_stack_can_be_refreshed_again() {
    let mut steps = Steps::default();
    steps.begin(LOOK, CALMER, &[TIGHTER, CALMER]);
    steps.tighter();
    steps.calmer();
    steps.history(&[(PRINT, &[], &[DONE]), (DONE, &["tighter-cards", "calmer-banner"], &[]), (ROOT, &[], &[])]);
    steps.valid_print(DONE);
    steps.first_parent(PRINT, &[PRINT], &[DONE, ROOT], "0");
    steps.ancestor(DONE, DONE, 0);
    steps.ancestor(DONE, DONE, 0);
    let push = steps.pushed(LOOK, ROOT, &format!("drop {TIGHTER}\ndrop {CALMER}\n"), CALMER, PRINT, true);
    steps.check(0, &format!("pages printed at 222222222222, 0 landings since\n{TIGHTER_DROPPED}dropped calmer-banner, done on main by 222222222222, printed at 222222222222\npushed look-requests at 3333333\n"), &format!("{FETCH}{REBASED}{push}"));

    let mut steps = Steps::default();
    steps.begin(LOOK, PRINT, &[]);
    steps.legacy_history(&[
        (TIP, &[], &[]),
        (PRINT, &[], &[DONE]),
        (DONE, &["tighter-cards", "calmer-banner"], &[]),
        (ROOT, &[], &[]),
    ]);
    steps.valid_print(DONE);
    steps.first_parent(PRINT, &[TIP, PRINT], &[DONE, ROOT], "1");
    let push = steps.pushed(LOOK, PRINT, "noop\n", PRINT, TIP, false);
    steps.check(
        0,
        "pages printed at 222222222222, 1 landings since\npushed look-requests at 5555555\n",
        &format!("{FETCH}{REBASED}{push}"),
    );
}

fn verdict(name: &str, landed: &str, result: &str) -> String {
    format!(
        "<section data-look-request=\"{name}\" data-look-commit=\"{TIGHTER}\" data-look-landed=\"{landed}\" data-look-tip=\"{TIP}\" data-look-result=\"{result}\">\n</section>\n"
    )
}

#[test]
fn a_landed_request_stays_until_main_has_the_newest_landing_verdict() {
    let old = verdict("tighter-cards", DONE, "differs");
    let checked = format!("{}{old}", verdict("tighter-cards", LATER, "differs"));
    for (entries, has_newest) in [(None, false), (Some(old.as_str()), false), (Some(checked.as_str()), true)] {
        let mut steps = stack();
        steps.look_landings(&format!("{TIP}\n\0{LATER}\ntighter-cards\n\0{DONE}\ntighter-cards\n\0"));
        steps.verdicts("web", entries);
        // Printed covers the old request, but only the newest Look-landed's verdict can drop it.
        steps.legacy_history(&[(TIP, &[], &[LATER]), (LATER, &[], &[]), (DONE, &["tighter-cards"], &[])]);
        steps.valid_print(LATER);
        steps.first_parent(TIP, &[TIP], &[LATER, DONE, ROOT], "0");
        let (todo, report, head) = if has_newest {
            ("drop", "dropped tighter-cards, landed by 444444444444, checked: differs", TIP)
        } else {
            ("pick", "kept tighter-cards, landed by 444444444444, not yet checked", REBASED_HEAD)
        };
        let push = steps.pushed(LOOK, ROOT, &format!("{todo} {TIGHTER}\n"), TIGHTER, head, true);
        steps.check(
            0,
            &format!("pages printed at 444444444444, 0 landings since\n{report}\npushed look-requests at {head:.7}\n"),
            &format!("{FETCH}{REBASED}{push}"),
        );
    }
}

#[test]
fn tui_refresh_drops_a_landed_request_with_a_not_checked_verdict() {
    let name = "tui-tighter-cards";
    let mut steps = Steps::default();
    steps.begin(TUI, TIGHTER, &[TIGHTER]);
    steps.request(TIGHTER, ROOT, &format!("Look-request: {name}\n\n"));
    steps.look_landings(&format!("{LATER}\n{name}\n\0"));
    steps.verdicts("tui", Some(&verdict(name, LATER, "not checked").replace(TIGHTER, &"0".repeat(TIP.len()))));
    steps.legacy_history(&[(LATER, &[], &[])]);
    let push = steps.pushed(TUI, ROOT, &format!("drop {TIGHTER}\n"), TIGHTER, LATER, true);
    steps.check(
        0,
        &format!(
            "{NO_PRINT}dropped {name}, landed by 444444444444, checked: not checked\npushed tui-requests at 4444444\n"
        ),
        &format!("{FETCH}{REBASED}{push}"),
    );
}

#[test]
fn malformed_requests_are_refused_before_rebasing() {
    for trailers in [
        "Look-request: Bad_name\n\n",
        "Look-request: calmer-banner\nLook-request: extra\n\n",
        "Look-request: calmer-banner\nLook-request: \n\n",
        "Look-request: \nLook-request: calmer-banner\n\n",
        "Look-request: tighter-cards\n\n",
    ] {
        let mut steps = Steps::default();
        steps.begin(LOOK, TIGHTER, &[TIGHTER, CALMER]);
        steps.tighter();
        steps.request(CALMER, TIGHTER, trailers);
        let why = if trailers == "Look-request: tighter-cards\n\n" {
            format!("look-refresh: {CALMER} repeats Look-request: tighter-cards\n")
        } else {
            bad_trailer(CALMER)
        };
        steps.check(2, "", &format!("{FETCH}{why}"));
    }

    let mut steps = Steps::default();
    steps.begin(LOOK, TIGHTER, &[TIGHTER, SIDE, MERGED]);
    steps.tighter();
    steps.request(SIDE, ROOT, "Look-request: side-look\n\n");
    steps.git(&["show", "-s", "--format=%P", MERGED], &format!("{TIGHTER} {SIDE}\n"));
    steps.check(2, "", &format!("{FETCH}look-refresh: {MERGED} is a merge commit\n"));
}

#[test]
fn an_unsuitable_checkout_is_refused_before_fetching() {
    for branch in ["look-requests\n", "tui-requests\n"] {
        for porcelain in [" M 01.html\n", "M  01.html\n"] {
            let mut steps = Steps::default();
            steps.head(branch, 0);
            steps.state_paths(None);
            steps.status(porcelain);
            steps.check(2, "", "look-refresh: tracked files must be clean\n");
        }
        for state in ["rebase-merge", "rebase-apply", "MERGE_HEAD", "CHERRY_PICK_HEAD", "sequencer"] {
            let mut steps = Steps::default();
            steps.head(branch, 0);
            steps.state_paths(Some(state));
            steps.check(2, "", &format!("look-refresh: {state} is in progress\n"));
        }
    }
    for (branch, code) in [("main\n", 0), ("", 1)] {
        let mut steps = Steps::default();
        steps.head(branch, code);
        steps.check(2, "", "look-refresh: check out look-requests or tui-requests first\n");
    }
}

#[test]
fn the_front_shows_help_but_the_run_refuses_arguments() {
    use skein_shell::Front;

    let help = [Box::from(b"look-refresh".as_slice()), Box::from(b"--help".as_slice())];
    let Front::Print(text) = crate::TABLE.front(&help) else { panic!("the front must handle help") };
    assert!(text.contains("internal: not part of temper-client-journeys's public command line"));
    assert!(text.contains("temper-client-journeys look-refresh\n"));

    let args = [Box::from(b"look-refresh".as_slice()), Box::from(b"unexpected".as_slice())];
    let Front::Run { command, rest } = crate::TABLE.front(&args) else { panic!("the run must handle arguments") };
    assert_eq!(*command, crate::table::LOOK_REFRESH);
    check(Vec::new(), rest, 2, "", "look-refresh: takes no arguments\n");
}
