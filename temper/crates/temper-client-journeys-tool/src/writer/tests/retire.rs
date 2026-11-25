use std::fmt::Write as _;

use super::{
    BRANCH, CHECKOUT, COMMIT, FRESH, NEXT, OTHER, PRINTED, PROJECT, START, Script, TIP,
    verdicts::{
        BLOB, NAME, REQUEST, STOPPED, TUI, TUI_LOOKS, TUI_NAME, WEB, WEB_LOOKS, page, section, start_with_print,
    },
};

const NEW: &str = "web/journeys/new-walk";
const CHILD: &str = "tui/journeys/child";
const WALK: (&str, &str) = (BLOB, "walk 2\n");
const AFTER_WALK: (&str, &str) = (NEXT, "walk 2\nafter new-walk\ntui mark\n");

impl Script {
    pub(super) fn request_commit(&mut self, code: i32) {
        let revision = format!("{REQUEST}^{{commit}}");
        let stdout = if code == 0 { format!("{REQUEST}\n") } else { String::new() };
        self.preserved_git(&["rev-parse", "--verify", "--quiet", &revision], code, &stdout);
    }

    pub(super) fn added_paths(&mut self, code: i32, paths: &str) {
        self.preserved_git(&["show", "-s", "--format=%P", REQUEST], 0, &format!("{OTHER}\n"));
        self.preserved_git(
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "--no-renames",
                "--diff-filter=A",
                "-r",
                "-z",
                OTHER,
                REQUEST,
                "--",
                WEB,
                TUI,
            ],
            code,
            paths,
        );
    }

    fn adds_walks(&mut self, paths: &[&str]) {
        let mut added = String::new();
        for path in paths {
            write!(added, "temper/docs/design/client/{path}/walk\0").expect("write to String");
        }
        self.added_paths(0, &added);
    }

    fn tip_walks(&mut self, walks: &[(&str, (&str, &str))]) {
        let mut tree = String::new();
        for (path, (blob, _)) in walks {
            write!(tree, "100644 blob {blob}\ttemper/docs/design/client/{path}/walk\0").expect("write to String");
        }
        self.preserved_git(&["ls-tree", "-r", "--full-tree", "-z", TIP, "--", WEB, TUI], 0, &tree);
        for (_, (blob, body)) in walks {
            self.preserved_git(&["cat-file", "blob", blob], 0, body);
        }
    }

    fn write_looks(&mut self, path: &str, body: &str) {
        self.write(&format!("{CHECKOUT}/{path}"), &page(body));
    }

    fn only_foot_changed(mut self, entries_and_foot: &str, stderr: &str) {
        self.write_looks(WEB_LOOKS, entries_and_foot);
        self.verdict_only(WEB_LOOKS, "");
        self.locked_run(PROJECT, 0, &format!("{FRESH}wrote {BRANCH} {COMMIT}\n"), stderr);
    }

    fn no_foot_change(mut self, stderr: &str) {
        self.unchanged(TIP, START);
        self.locked_run(PROJECT, 0, &format!("{FRESH}main's pages are {TIP}'s print\n"), stderr);
    }
}

fn foot(items: &[(&str, &str)]) -> String {
    let mut html = String::from(concat!(
        "<footer>\n<h2>Journeys to retire</h2>\n",
        "<p>Journeys look requests added whose verdicts stand identical, which no walk starts after. ",
        "Whoever starts look requests' goals proposes which go.</p>\n<ul>\n",
    ));
    for (path, name) in items {
        writeln!(html, "<li>{path}, added by {name}</li>").expect("write to String");
    }
    html.push_str("</ul>\n</footer>\n");
    html
}

fn standing(result: &str, foot: &str) -> (Script, String) {
    let mut script = start_with_print(0, FRESH, &[(TIP, &[NAME])]);
    let entry = section(NAME, REQUEST, TIP, result, "");
    script.blob(TIP, WEB_LOOKS, Some(&page(&format!("{entry}{foot}"))));
    (script, entry)
}

fn with_added_walk(foot: &str) -> (Script, String) {
    let (mut script, entry) = standing("identical", foot);
    script.request_commit(0);
    script.adds_walks(&[NEW]);
    (script, entry)
}

#[test]
fn a_standing_identical_request_lists_the_walk_it_added() {
    let (mut script, entry) = with_added_walk("");
    script.tip_walks(&[(NEW, WALK)]);
    script.only_foot_changed(&format!("{entry}{}", foot(&[(NEW, NAME)])), "");
}

#[test]
fn a_new_verdict_lists_paths_in_order_on_its_request_root() {
    const FIRST: &str = "web/journeys/bright<&\">";
    const FIRST_ESCAPED: &str = "web/journeys/bright&lt;&amp;&quot;&gt;";
    const LAST: &str = "tui/journeys/a-terminal";
    let mut script = start_with_print(0, FRESH, &[(TIP, &[TUI_NAME])]);
    script.blob(TIP, TUI_LOOKS, None);
    script.request_fetch(TUI_NAME, true, 0);
    script.request_tip(OTHER, Some(&format!("Look-request: {TUI_NAME}\n\n")));
    script.render_request(TUI_NAME, 0, "result\tidentical\n");
    script.request_commit(0);
    script.adds_walks(&[LAST, FIRST, NEW]);
    script.tip_walks(&[(LAST, WALK), (FIRST, WALK), (NEW, WALK)]);
    let entry = section(TUI_NAME, REQUEST, TIP, "identical", "");
    let foot = foot(&[(FIRST_ESCAPED, TUI_NAME), (NEW, TUI_NAME), (LAST, TUI_NAME)]);
    script.write_looks(TUI_LOOKS, &format!("{entry}{foot}"));
    let line = format!("checked {TUI_NAME}: identical");
    script.verdict_only(TUI_LOOKS, &line);
    script.locked_run(PROJECT, 0, &format!("{FRESH}{line}\nwrote {BRANCH} {COMMIT}\n"), "");
}

#[test]
fn an_after_in_the_other_root_excludes_the_parent() {
    let (mut script, _) = with_added_walk("");
    script.tip_walks(&[(CHILD, AFTER_WALK), (NEW, WALK)]);
    script.no_foot_change("");
}

#[test]
fn a_different_verdict_does_not_read_request_history() {
    let (script, _) = standing("differs", "");
    script.no_foot_change("");
}

#[test]
fn a_missing_commit_is_fetched_before_reading_its_additions() {
    let (mut script, entry) = standing("identical", "");
    script.request_commit(1);
    script.request_fetch(NAME, true, 0);
    script.request_commit(0);
    script.adds_walks(&[NEW]);
    script.tip_walks(&[(NEW, WALK)]);
    script.only_foot_changed(&format!("{entry}{}", foot(&[(NEW, NAME)])), "");
}

#[test]
fn a_missing_branch_warns_without_listing_any_of_its_walks() {
    let (mut script, _) = standing("identical", "");
    script.request_commit(1);
    script.request_fetch(NAME, false, 0);
    script.no_foot_change(&format!("journey-pages: cannot read what {NAME} added: origin lacks docs/look/{NAME}\n"));
}

#[test]
fn a_fetch_without_the_standing_commit_warns_and_continues() {
    let (mut script, _) = standing("identical", "");
    script.request_commit(1);
    script.request_fetch(NAME, true, 0);
    script.request_commit(1);
    script.no_foot_change(&format!(
        "journey-pages: cannot read what {NAME} added: fetching docs/look/{NAME} did not supply {REQUEST}\n"
    ));
}

#[test]
fn a_journey_gone_from_tip_removes_the_stale_footer() {
    let (mut script, entry) = with_added_walk(&foot(&[(NEW, NAME)]));
    script.tip_walks(&[]);
    script.only_foot_changed(&entry, "");
}

#[test]
fn an_unchanged_footer_finishes_without_rewriting_the_page() {
    let (mut script, _) = with_added_walk(&foot(&[(NEW, NAME)]));
    script.tip_walks(&[(NEW, WALK)]);
    script.no_foot_change("");
}

#[test]
fn a_pending_repair_does_not_reuse_the_older_identical_verdict() {
    let mut script = start_with_print(1, STOPPED, &[(TIP, &[NAME]), (NEXT, &[NAME])]);
    let older = section(NAME, REQUEST, NEXT, "identical", "");
    let stale = foot(&[(NEW, NAME)]);
    script.blob(TIP, WEB_LOOKS, Some(&page(&format!("{older}{stale}"))));
    script.stopped_request();
    script.write_looks(WEB_LOOKS, &older);
    let line = format!("pending {NAME}: welcome stopped");
    script.verdict_only(WEB_LOOKS, &format!("{line}\n\n{}", STOPPED.trim_end()));
    script.locked_run(PROJECT, 1, &format!("{STOPPED}{line}\nwrote {BRANCH} {COMMIT}\n"), "");
}

#[test]
fn a_failed_read_of_additions_reverts_both_roots_without_offering() {
    let mut script = start_with_print(0, PRINTED, &[(TIP, &[NAME])]);
    let entry = section(NAME, REQUEST, TIP, "identical", "");
    script.blob(TIP, WEB_LOOKS, Some(&page(&entry)));
    script.request_commit(0);
    script.added_paths(128, "");
    script.restore_journeys();
    let why = format!(
        "git diff-tree --no-commit-id --name-only --no-renames --diff-filter=A -r -z {OTHER} {REQUEST} -- {WEB} {TUI}: exit 128"
    );
    script.locked_run(PROJECT, 1, PRINTED, &format!("journey-pages: {why}\n"));
}
