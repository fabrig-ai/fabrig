+++
id = "why-slow"
part = "slow-walks"
kind = "feature"
needs = []
files = ["temper/docs/plan/browser-trim/figures.md", "temper/tests/client-browser/src/**", "temper/tests/client-browser/tests/browser_all/browser_public.rs", "temper/tests/client-browser/tests/browser_all/browser_playground.rs"]
effort = "high"
minutes = 90
+++

# Where the slow walks' time goes

## Goal

`history_keeps_the_chat_focus_and_filter_across_reload_and_back` took 15.9 CPU s and 23.1 s wall; it was 4.7 CPU s on 11-03. `a_visitor_reads_a_public_project_and_loses_it_when_made_private` took 14.1 CPU s and 27.3 s wall. Say by measurement where that time goes, and fix each cause where it lies. A page of a person's that's slower to settle is a product finding, and the most valuable answer here. Then look at the playground and the rest for a missing cache or a needless rebuild, and only that.

Measure before anything changes: these walks are cut by the next task of this plan, and the cut would hide a cause.

Acceptance:

- **Where the wall goes:** for each of the two tests, a table of wall by range of steps that names at least 90% of the run, teardown included, and CPU by process (test binary, `temper`, Chromium's browser, renderer and GPU).
- **The page, profiled:** on the box, a chat's page and a task's page held open for at least 5 s with nothing arriving, and a public page: the deltas of CDP's `Performance.getMetrics` (`ScriptDuration`, `TaskDuration`, `LayoutCount`, `RecalcStyleCount`), and the requests and stream connections opened meanwhile. A settled page's figures stand still. Say whether they do.
- **A verdict with its figure on each candidate below,** and on any you find.
- **All of that is committed** in `temper/docs/plan/browser-trim/figures.md`, under a heading *Before*, with the base commit, the machine and the load: the next task reads it, and the plan's last measurement sets its own figures beside these.
- **A product cause** is fixed red first: a commit with a test in the lowest tier that shows it, seen failing, then the fix in the code under test.
- **A test-side cause** is fixed in the test or the tier's harness, unless it's in steps the next task cuts (below).
- Every test of the tier that a fix touches passes, and the product's readers' tests run.

## The candidates, read from the code and not measured

- **The box's pace.** `38fbd8093d` (2026-11-05) moved the history story onto `Tier::demo` and has it walk a change to `Landed`. Story A's documents pace the fakes: `temper/crates/temper-playground/stories/a/llm.json`'s `pace.reply_ms` is 1200 and `forge.json`'s `ci_ms` is 4000. `temper init --box` writes a deployment with `application.demo.llm`: check whether the box runs at those paces. `latest_returns…`, on the same tier with no walk to landing, took 4.0 s wall.
- **Polling while waiting.** A `See` retries through the kit's 10 ms `POLL` (`skein/testing/skein-fake-person-browser/src/harness.rs`), each retry a query over CDP, so CPU may grow with the wall.
- **`temper ctl change show --until`,** which the public walk runs twice: what it polls and how often.
- **The public walk's reading:** `Find::text(b"").nth(i)` reads each page's text one node at a time, and its deployment's freshness is two seconds (`configuration::public_demo`).
- **Teardown.** `browser_authorize.rs` records that a temper page left in the tab's history keeps its connection open, and that stopping the server then waits out jig's 30 s close deadline. The history story ends after Back and Forward. The harness's run deadline is 36 s, so a 27 s run is near it.
- **The page itself:** a re-render loop, a poll, a stream reconnect, a settling wait.
- **Story A's `download_size`** gzips the whole built bundle in the dev profile's unoptimized code. `temper/docs/design/playground.md`, section 9 keeps the figure.
- **Every test's floor.** `browser_github`, at 1.9 CPU s, is about what a test costs before its story: name what that's spent on, wasm compilation in Chromium's fresh profile included. `Tier::accounts`' `copy_bundle` copies the whole bundle for one document.

## Inputs

- `temper/docs/plan/browser-trim/notes/common.md`.
- `temper/docs/plan/browser-trim/inputs/request.md`, items 3 and 4.
- `skein/docs/foundation/testing-strategy.md`, 8 (*A slow test is understood*, *A tool is not the code under test*) and 8.1.
- `temper/docs/design/client/browser.md`, 4.3 and 8.4; `temper/docs/design/playground.md`, section 9.

## Owns and may change

- Owns `temper/docs/plan/browser-trim/figures.md`, `temper/tests/client-browser/src/**`, `browser_public.rs`'s reading mechanics and `browser_playground.rs`. Not `browser_task_pages.rs`, which `draft-link` changes alongside you.
- **The product code a cause lies in,** in any crate of temper, with its test. A cause in skein or jig is reproduced and fixed in that tree's lowest tier (`AGENTS.md`, 5), and that tree's browser tier runs what it touches. `skein-browser`'s launch flags may change if the floor shows a needless Chromium feature; run skein's and jig's browser tiers' narrow tests if so.
- **Not here, the next task's:** cutting a story's steps. If the wall is the walk to `Landed` itself (`change_and_close_steps` in `browser_pages.rs`; the owner's wait for `ci:passed` and `landed` in `browser_public.rs`), report its figure and fix nothing there.

Options, each decided:

- **If the box's pace is the cause:** don't zero it in `Tier::demo`. With nothing waiting on it, an unpaced fake does the whole landing in the background and costs more CPU. The cut removes the wait.
- **If one-at-a-time reading costs over 1 CPU s in the public walk:** read each page's text in fewer queries, checking the same words on the same pages.
- **If `download_size`'s gzip costs over 1 CPU s:** compress with the machine's `gzip` as a tool, keeping the printed figure's meaning, and say so; if that would change what the figure means against its 8 MB bar, leave it and report the cost.
- **If a compile or load cache would save a test's floor:** add it only if each test keeps its own server, page state and profile semantics. Otherwise report the figure and leave it.
- **If nothing is wrong beyond the walks:** that's a finding. Change no code; `figures.md` says so, with its tables, and is your commit.

## Steps

1. A timing `Play` around the script, throwaway, that prints each step's number and time (`browser_public.rs`'s `Walk` already prints per page). `heavy perf record` is pre-approved, as `sudo perf` for that command alone if it refuses; split by process. A throwaway CDP script for the page's metrics, handling `Page.javascriptDialogOpening`.
2. Run each test alone on as idle a slot as you can; note the load, and repeat a figure that looks off.
3. Fix what the verdicts name, red first for a product cause, breaking each new test once by hand.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` for every package changed; the focused tests of those packages and their direct dependents; `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As the implementer's text asks, with the tables, the verdicts, the settled page's figures and the machine. The first line says whether a product cause was found. What the design should say about it goes here: `docs/design` isn't yours.
