use super::{
    ALLOWED, Answer, Ask, BRANCH, CHECKOUT, COMMIT, FRESH, NEXT, OTHER, PAGE, PROJECT, Ran, START, Script, Stream, TIP,
    program,
};

pub(super) const WEB: &str = "temper/docs/design/client/web/journeys";
pub(super) const TUI: &str = "temper/docs/design/client/tui/journeys";
pub(super) const WEB_LOOKS: &str = "temper/docs/design/client/web/journeys/looks.html";
pub(super) const TUI_LOOKS: &str = "temper/docs/design/client/tui/journeys/looks.html";
const HEADER: &str = "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<title>Look requests rendered · temper</title>\n</head>\n<body>\n<h1>Look requests rendered · temper</h1>\n";
const FOOTER: &str = "</body>\n</html>\n";
const STATES: &str = "<p>States the request names in words aren't rendered: its landing's account covers them.</p>\n";
const TUI_NOTE: &str = "<p>These are the browser's renders of the TUI's frames, not a terminal's.</p>\n";
pub(super) const NAME: &str = "calmer-banner";
pub(super) const TUI_NAME: &str = "tui-terminal";
pub(super) const REQUEST: &str = "1111111111111111111111111111111111111111";
pub(super) const BLOB: &str = "2222222222222222222222222222222222222222";
const RESTORE: &str = "/writer/.git/journey-pages.lock/restore-paths";

impl Script {
    pub(super) fn preserved_git(&mut self, args: &[&str], code: i32, stdout: &str) {
        let mut git = program("git", args);
        git.directory = Some(CHECKOUT.into());
        git.stderr = Stream::Keep;
        self.ask(Ask::Run(git), Answer::Ran(Ran::new(Some(code), stdout.as_bytes())));
    }

    pub(super) fn blob(&mut self, revision: &str, path: &str, contents: Option<&str>) {
        self.preserved_git(
            &["--literal-pathspecs", "ls-tree", "--full-tree", "-z", revision, "--", path],
            0,
            &contents.map_or_else(String::new, |_| format!("100644 blob {BLOB}\t{path}\0")),
        );
        if let Some(contents) = contents {
            self.preserved_git(&["cat-file", "blob", BLOB], 0, contents);
        }
    }

    pub(super) fn request_fetch(&mut self, name: &str, exists: bool, code: i32) {
        let branch = format!("refs/heads/docs/look/{name}");
        self.command(
            "git",
            &["ls-remote", "--exit-code", "--refs", "origin", &branch],
            if exists { 0 } else { 2 },
            &if exists { format!("{REQUEST}\t{branch}\n") } else { String::new() },
            Stream::Keep,
        );
        if exists {
            self.preserved_git(
                &["fetch", "--no-tags", "--no-recurse-submodules", "--refmap=", "origin", &branch],
                code,
                "",
            );
        }
    }

    pub(super) fn request_tip(&mut self, parents: &str, trailers: Option<&str>) {
        self.preserved_git(&["rev-parse", "--verify", "FETCH_HEAD^{commit}"], 0, &format!("{REQUEST}\n"));
        self.preserved_git(&["show", "-s", "--format=%P", REQUEST], 0, &format!("{parents}\n"));
        if let Some(trailers) = trailers {
            self.preserved_git(&["show", "-s", "--format=%(trailers:key=Look-request,unfold)", REQUEST], 0, trailers);
        }
    }

    fn request(&mut self) {
        self.request_fetch(NAME, true, 0);
        self.request_tip(OTHER, Some(&format!("Look-request: {NAME}\n\n")));
    }

    fn render(&mut self, code: i32, stdout: &str) {
        self.render_request(NAME, code, stdout);
    }

    pub(super) fn render_request(&mut self, name: &str, code: i32, stdout: &str) {
        self.command(
            "cargo",
            &["run", "--locked", "--quiet", "-p", "temper-client-journeys", "--", "look-render", name, REQUEST],
            code,
            stdout,
            Stream::Keep,
        );
    }

    pub(super) fn verdict_only(&mut self, path: &str, line: &str) {
        self.status(&format!(" M {path}\0"));
        let body = if line.is_empty() { format!("Printed: {TIP}") } else { format!("{line}\n\nPrinted: {TIP}") };
        self.commit(&format!("{path}\0"), &body, COMMIT);
        self.push("origin", BRANCH, true);
        self.open(BRANCH, COMMIT, &body, ALLOWED);
    }

    pub(super) fn restore_journeys(&mut self) {
        let paths = format!("{PAGE}\0{TUI}/terminal/01.txt\0");
        self.git(&["diff", "--name-only", "--no-renames", "-z", TIP, "--", WEB, TUI], &paths);
        self.write(RESTORE, &paths);
        self.command(
            "git",
            &[
                "--literal-pathspecs",
                "restore",
                "--source",
                TIP,
                "--staged",
                "--worktree",
                &format!("--pathspec-from-file={RESTORE}"),
                "--pathspec-file-nul",
            ],
            0,
            "",
            Stream::Err,
        );
        self.command("git", &["clean", "-ffdx", "--", WEB, TUI], 0, "", Stream::Err);
    }
}

pub(super) fn section(name: &str, commit: &str, landed: &str, result: &str, detail: &str) -> String {
    let tui = if name == TUI_NAME { TUI_NOTE } else { "" };
    format!(
        "<section data-look-request=\"{name}\" data-look-commit=\"{commit}\" data-look-landed=\"{landed}\" data-look-tip=\"{TIP}\" data-look-result=\"{result}\">\n<h2>{name}</h2>\n<p>Request {commit}, landed by {landed}, printed at {TIP}: {result}.</p>\n{detail}{STATES}{tui}</section>\n"
    )
}

pub(super) fn page(entries: &str) -> String {
    let compact = format!("{HEADER}{entries}{FOOTER}").lines().skip(1).collect::<String>();
    format!("<!doctype html>\n{}\n", compact.replace('>', "\n>"))
}

pub(super) fn start_with_print(code: i32, lines: &str, landings: &[(&str, &[&str])]) -> Script {
    let mut script = Script::fresh(BRANCH, None);
    script.checkout(BRANCH, TIP);
    script.print(code, lines);
    script.landings(TIP, landings);
    script
}

fn with_web_render(print: &str, report: &str) -> Script {
    let mut script = start_with_print(0, print, &[(TIP, &[NAME])]);
    script.blob(TIP, WEB_LOOKS, None);
    script.request();
    script.render(0, report);
    script
}

impl Script {
    fn offers_web_verdict(mut self, result: &str, detail: &str) {
        let html = section(NAME, REQUEST, TIP, result, detail);
        self.write(&format!("{CHECKOUT}/{WEB_LOOKS}"), &page(&html));
        let line = format!("checked {NAME}: {result}");
        self.verdict_only(WEB_LOOKS, &line);
        self.locked_run(PROJECT, 0, &format!("{FRESH}{line}\nwrote {BRANCH} {COMMIT}\n"), "");
    }
}

#[test]
fn a_landing_without_page_changes_still_offers_its_escaped_verdict() {
    let script = with_web_render(
        FRESH,
        concat!(
            "uncompared\tweb/journeys/old/01.html\thand-made\n",
            "differs\tweb/journeys/welcome/01.html\t1440\t41 button.temper-chip\tcontent\t<>&\"\\t\\\\\tlanded\n",
            "result\tdiffers\n",
        ),
    );
    let detail = concat!(
        "<h3><a href=\"old/01.html\">web/journeys/old/01.html</a></h3>\n<p>not compared: hand-made</p>\n",
        "<h3><a href=\"welcome/01.html\">web/journeys/welcome/01.html</a></h3>\n",
        "<table>\n<thead>\n<tr>\n<th>Width</th>\n<th>Element</th>\n<th>Property</th>\n<th>Requested</th>\n<th>Landed</th>\n</tr>\n</thead>\n<tbody>\n",
        "<tr>\n<td>1440</td>\n<td>41 button.temper-chip</td>\n<td>content</td>\n<td>&lt;&gt;&amp;&quot;\t\\</td>\n<td>landed</td>\n</tr>\n</tbody>\n</table>\n",
    );
    script.offers_web_verdict("differs", detail);
}

#[test]
fn a_hand_made_pages_verdict_links_its_own_file() {
    let script = with_web_render(
        FRESH,
        "uncompared\tweb/journeys/usage-model-week/accounts-after.html\thand-made\nresult\tnot compared\n",
    );
    let detail = concat!(
        "<h3><a href=\"usage-model-week/accounts-after.html\">web/journeys/usage-model-week/accounts-after.html</a></h3>\n",
        "<p>not compared: hand-made</p>\n",
    );
    script.offers_web_verdict("not compared", detail);
}

#[test]
fn a_nested_report_path_restores_both_roots_without_finishing_or_offering() {
    let mut script =
        with_web_render(super::PRINTED, "uncompared\tweb/journeys/J/sub/page.html\thand-made\nresult\tnot compared\n");
    script.restore_journeys();
    script.locked_run(PROJECT, 1, super::PRINTED, "journey-pages: invalid look-render page path\n");
}

#[test]
fn a_verdict_for_the_newest_landing_skips_the_request() {
    let mut script = start_with_print(0, FRESH, &[(TIP, &[NAME]), (OTHER, &[NAME])]);
    let html = section(NAME, REQUEST, TIP, "identical", "");
    script.blob(TIP, WEB_LOOKS, Some(&page(&html)));
    script.request_commit(0);
    script.added_paths(0, "");
    script.unchanged(TIP, START);
    script.locked_run(&[], 0, &format!("{FRESH}main's pages are {TIP}'s print\n"), "");
}

#[test]
fn a_repair_checks_again_and_preserves_old_and_malformed_entries() {
    let mut script = start_with_print(0, FRESH, &[(TIP, &[NAME]), (NEXT, &[NAME])]);
    let older = section(NAME, REQUEST, NEXT, "differs", "");
    let malformed = format!(
        "<section data-look-request=\"{NAME}\" data-look-commit=\"{REQUEST}\" data-look-landed=\"{TIP}\" data-look-result=\"identical\">\n</section>\n"
    );
    script.blob(TIP, WEB_LOOKS, Some(&page(&format!("{malformed}{older}"))));
    script.request();
    script.render(0, "result\tnothing to render\n");
    let newer = section(NAME, REQUEST, TIP, "nothing to render", "");
    script.write(&format!("{CHECKOUT}/{WEB_LOOKS}"), &page(&format!("{newer}{malformed}{older}")));
    let line = format!("checked {NAME}: nothing to render");
    script.verdict_only(WEB_LOOKS, &line);
    script.locked_run(
        PROJECT,
        0,
        &format!("{FRESH}{line}\nwrote {BRANCH} {COMMIT}\n"),
        &format!("journey-pages: ignoring malformed verdict at {WEB_LOOKS}:11\n"),
    );
}

#[test]
fn a_missing_tui_request_gets_a_not_checked_verdict_in_its_root() {
    let mut script = start_with_print(0, FRESH, &[(TIP, &[TUI_NAME])]);
    script.blob(TIP, TUI_LOOKS, None);
    script.request_fetch(TUI_NAME, false, 0);
    let html = section(
        TUI_NAME,
        &"0".repeat(TIP.len()),
        TIP,
        "not checked",
        &format!("<p>origin lacks docs/look/{TUI_NAME}</p>\n"),
    );
    script.write(&format!("{CHECKOUT}/{TUI_LOOKS}"), &page(&html));
    let line = format!("checked {TUI_NAME}: not checked");
    script.verdict_only(TUI_LOOKS, &line);
    script.locked_run(PROJECT, 0, &format!("{FRESH}{line}\nwrote {BRANCH} {COMMIT}\n"), "");
}

#[test]
fn an_invalid_request_names_its_tip_instead_of_running_the_renderer() {
    let no_parent = format!("{REQUEST} must have exactly one parent");
    let bad_trailer = format!("{REQUEST} must carry exactly one Look-request: {NAME} trailer");
    for (parents, trailers, why) in [
        (String::new(), None, &no_parent),
        (format!("{OTHER} {NEXT}"), None, &no_parent),
        (OTHER.into(), Some("Look-request: calmer-banner\nLook-request: extra\n\n"), &bad_trailer),
    ] {
        let mut script = start_with_print(0, FRESH, &[(TIP, &[NAME])]);
        script.blob(TIP, WEB_LOOKS, None);
        script.request_fetch(NAME, true, 0);
        script.request_tip(&parents, trailers);
        let html = section(NAME, REQUEST, TIP, "not checked", &format!("<p>{why}</p>\n"));
        script.write(&format!("{CHECKOUT}/{WEB_LOOKS}"), &page(&html));
        let line = format!("checked {NAME}: not checked");
        script.verdict_only(WEB_LOOKS, &line);
        script.locked_run(PROJECT, 0, &format!("{FRESH}{line}\nwrote {BRANCH} {COMMIT}\n"), "");
    }
}

pub(super) const STOPPED: &str = "stopped welcome: line 4: missing mark\n";

impl Script {
    pub(super) fn stopped_request(&mut self) {
        self.request();
        self.preserved_git(
            &["diff-tree", "--no-commit-id", "--name-only", "--no-renames", "-r", "-z", OTHER, REQUEST, "--", WEB, TUI],
            0,
            &format!("{PAGE}\0"),
        );
        let walk = format!("{WEB}/welcome/walk");
        let mut tree = program("git", &["--literal-pathspecs", "ls-tree", "--full-tree", "-z", REQUEST, "--", &walk]);
        tree.stderr = Stream::Keep;
        self.ask(Ask::Run(tree), Answer::Ran(Ran::new(Some(0), format!("100644 blob {BLOB}\t{walk}\0").as_bytes())));
        let mut cat = program("git", &["cat-file", "blob", BLOB]);
        cat.stderr = Stream::Keep;
        self.ask(Ask::Run(cat), Answer::Ran(Ran::new(Some(0), b"walk 2\n")));
    }
}

#[test]
fn a_request_for_a_stopped_walk_is_pending_without_an_entry() {
    let mut script = start_with_print(1, STOPPED, &[(TIP, &[NAME])]);
    script.blob(TIP, WEB_LOOKS, None);
    script.stopped_request();
    script.unchanged(TIP, START);
    script.locked_run(
        PROJECT,
        1,
        &format!("{STOPPED}pending {NAME}: welcome stopped\nmain's pages are {TIP}'s print\n"),
        "",
    );
}

#[test]
fn a_failed_fetch_or_render_restores_both_roots_without_finishing_or_offering() {
    for fetch in [false, true] {
        let mut script = start_with_print(0, super::PRINTED, &[(TIP, &[NAME])]);
        script.blob(TIP, WEB_LOOKS, None);
        let why = if fetch {
            script.request_fetch(NAME, true, 128);
            format!(
                "git fetch --no-tags --no-recurse-submodules --refmap= origin refs/heads/docs/look/{NAME}: exit 128"
            )
        } else {
            script.request();
            script.render(3, "");
            format!("look-render could not check {NAME}")
        };
        script.restore_journeys();
        script.locked_run(PROJECT, 1, super::PRINTED, &format!("journey-pages: {why}\n"));
    }
}
