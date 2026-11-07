+++
id = "harness-world"
part = "harness-world"
kind = "migration"
needs = []
files = [
  "skein/tests/ui-web/src/bin/harness_fixture.rs",
  "skein/tests/ui-web/src/bin/harness_fixture/browser.rs",
  "skein/tests/ui-web/src/lib.rs",
  "skein/tests/ui-web/tests/suite/main.rs",
  "skein/tests/ui-web/tests/suite/harness.rs",
  "skein/tests/ui-web/tests/browser_harness.rs",
  "skein/testing/skein-fake-person-browser/src/harness/failure_tests.rs",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 75
+++

# The harness's child and peer tests leave the browser tier

## Goal

`browser_harness` starts Chromium 35 times, and in 31 of them no real browser's behaviour is what's checked: they check what the harness does with a child or a peer. Those go to a focused world, the harness as it ships at a fake browser, which `Setup.chromium` already lets a test name. What a real browser shows stays, in two tests. This is `testing-strategy.md`, 8.1's first question: what each checks decides its place.

Acceptance:

- `harness_fixture`, started with `--headless` first, is the fake browser of `skein/docs/design/ui-web.md`, 14.2.
- `skein/tests/ui-web/tests/suite/harness.rs` holds the 13 tests below, and they pass by name in focused: `heavy cargo nextest run -p skein-ui-web-tests --test suite -E 'test(/^harness::/)'`, 13 selected.
- `browser_harness.rs` holds the 2 tests below, compiled and **not run**: `heavy cargo nextest run -p skein-ui-web-tests --test browser_harness --no-run`.
- `a_failed_keep_reports_its_own_step` has the two rows below, and passes.
- Every assertion of `plan.md`'s rows 68 to 102 is where its row says: the table is the checklist, row by row.
- The four rules under *Bite* each fail, for the reason named.
- The report gives nextest's time for each of the 13. **A test past about 100 ms is a finding:** read what it waits on, and fix a wait in the test or the fixture. Report what's left.
- `skein/docs/design/ui-web.status.md`'s line beginning `- **Not yet:** 14.2: the harness's child and peer tests` is gone, whole.

## Inputs

- `skein/docs/design/ui-web.md`, 14.2: *The harness*, and *What the harness does with a child or a peer is tested below the tier*.
- `skein/docs/plan/skein-browser-starts/notes/common.md`.
- `skein/docs/plan/skein-browser-starts/plan.md`, the table's `browser_harness` rows, and *Belonging*.
- `skein/tests/browser/src/world.rs`, `World::reply` and the notifications in `drive`: what the kit needs answered. It's the shape to copy, not a dependency: a fake shares no types with what it stands in for, and `skein-browser-world` isn't a dependency of this crate.
- `skein/testing/skein-browser/src/command.rs`: what Chromium is started with.

## Owns and may change

Owns the files in the header, and only that line of the status file. `Harness` and its modules don't change: **the seam is `Setup.chromium`**, a path. If the harness can't run one of these without Chromium after all, that test stays in `browser_harness.rs`, as it is, and the report says why. That's an answer, not a question.

Neighbours: nothing outside this package calls `lending_harness`, `harness_fixture` or `harness_peer`. `tools/tiers/design.md` names `harness_fixture` as a binary cargo builds for this package's tests, which stays so.

Options, each decided:

- **`src/bin/harness_fixture/browser.rs`:** the manifest's `[[bin]]` path stays `src/bin/harness_fixture.rs`, with `mod browser;` there. Cargo finds no second binary in that directory.
- **The crate forbids `unsafe`,** so no `File::from_raw_fd`: open `/proc/self/fd/3` to read and `/proc/self/fd/4` to write.
- **`lending_harness` names the fake.** No browser test uses it after this task, so change its `chromium` to `harness_fixture()`.
- **`ready_person`'s `See`** waited for the page and checked nothing a peer test asserts. Those people have no steps: `Script::new(Box::new([]))`, and the run is `Done` once each page has opened. A person with no steps reaches `Next::Done` from `Script::next`.
- **The keeps need no origin.** The fake's fixed page is what they read, so they drop `Visit` and their step numbers fall by one. The real browser's reading of an origin's page stays in the merged tier test.
- **The fixed page's `value` is `fixture value`,** which `a_link_keep_and_a_selector_keep...` already expects from the fixture's page. Rows that read `first` read that now.
- **Anything else the kit asks** gets `{}`, as `World::reply`'s last arm does. A failed run asks a snapshot, then a screenshot: the snapshot is `{"nodes":[]}`, the screenshot a CDP error, so that no failure leaves a PNG in the temporary directory.

## Steps

1. **The fake browser.** Sketch of `browser.rs`, ordinary Rust on `std`, and `skein_browser::wire::decode::{Document, Value}` to read each command:

   ```rust
   //! A browser at Chromium's path: the kit's CDP on descriptors 3 and 4, and no page
   //! but one fixed one (skein/docs/design/ui-web.md, 14.2).
   pub(super) fn run() { /* read NUL-framed commands until `Browser.close` or end of input */ }
   ```

   A reply is `{"id":ID[,"sessionId":"S"],"result":R}\0`, with the command's own session. A notification carries that session too.

   | Command | Result |
   |---|---|
   | `Browser.getVersion` | `{"product":"Chrome/fixture"}` |
   | `Target.createBrowserContext` | `{"browserContextId":"cN"}`, N counting up |
   | `Target.createTarget` | `{"targetId":"tN"}` |
   | `Target.attachToTarget` | `{"sessionId":"sN"}` |
   | `DOM.getDocument` | `{"root":{"nodeId":1}}` |
   | `DOM.querySelector` | `{"nodeId":0}` for the selector `#missing`, else `{"nodeId":7}` |
   | `DOM.getAttributes` | `{"attributes":["value","fixture value","data-note","named","data-empty",""]}` |
   | `Accessibility.getFullAXTree` | `{"nodes":[]}` |
   | `Page.captureScreenshot` | the error `{"code":-32000,"message":"no screen"}` |
   | `Page.navigate` | below |
   | `Browser.close` | `{}`, then exit 0 |
   | any other | `{}` |

   `Page.navigate`, in `World::drive`'s order: `Page.frameStartedLoading` (`frameId` `f1`), the reply, then for an `http://` address the fixture's `request(address, path)` read whole, then `Page.loadEventFired`. An address that isn't `http://` is fetched not at all. A fetch that fails panics, and the harness reports the child's exit.

   `main` gains `Some("--headless") => browser::run(),`.
2. **The world,** `tests/suite/harness.rs`, with `mod harness;` in `suite/main.rs`. Move `command`, `exited`, `command_story` and `command_harness` here from `browser_harness.rs`, with `command_harness`'s `chromium` as `harness_fixture()`. The 13:

   | Test | Holds | Shape |
   |---|---|---|
   | `lent_peers_listen_first_and_return_empty_and_usable` | rows 74, 75, 77, 79 | `lending_harness(&["{first}", "{first}", "{second}"], vec![])`, two peers, a person with no steps. `first`'s observations are the exact request, twice; `second`'s, once; each `handle.get()` is its count, read through a clone taken before the run; each `is_empty()` |
   | `a_lent_peers_assertion_names_it_in_the_failure` | row 76 | moved, with a person with no steps |
   | `a_peer_shutdown_assertion_preserves_a_failed_outcome` | row 78 | moved, likewise |
   | `keeps_append_in_completion_order_and_a_new_run_clears_them` | rows 83, 84 | two people, as row 83, on one `Harness`; then a second `run` on it, whose `kept()` is that run's one entry |
   | `a_failed_run_retains_completed_keeps` | rows 81 (its run), 85 | keep `input`, keep `#missing`: `Failed { person: 0, step: 1, .. }`, the report `keep selector #missing attribute value: no element`, one entry kept |
   | `a_held_line_reaches_a_command_and_closed_stdin_only_eof` | rows 86, 87 | `command_harness(Sent::Command)`, one person: `Run` the `stdin` command printed-stdin, `Exited(0)`, `Run` the `closed` command, `Exited(0)`; `kept()[0]` ends with `/opened` |
   | `people_run_commands_beside_their_pages` | rows 73, 88, 90, 93, 95, 96, 97, 98, 99 | below |
   | `a_command_without_the_link_fails_naming_the_command` | row 89 | moved |
   | `exited_rejects_a_different_code_naming_the_command` | row 91 | moved |
   | `a_live_command_at_story_end_fails_and_settles` | row 92 | moved |
   | `a_second_run_while_the_command_runs_fails` | row 94 | moved |
   | `a_missing_link_deadline_fails_and_command_teardown_settles` | row 100 | moved, its 50 ms as it is |
   | `an_exited_deadline_fails_and_command_teardown_settles` | row 101 | moved, its 50 ms as it is |

   `people_run_commands_beside_their_pages`: `command_harness(Sent::Page)`, `run_lending` with one peer `neighbor`, four people:

   - **0:** `Run` `link http://{address}/opened?kept=whole` opening, keep `input` as `field`, `Exited(0)`. Kept, in order: `link`, whole, starting `http://127.0.0.1:` and ending `/opened?kept=whole`; then `field`, `fixture value`.
   - **1:** `Run` `code 7`, `Exited(7)`, `Run` `code 8`, `Exited(8)`, then row 73's `Step::Request` with the host `other.test`, status 200. Keeps nothing.
   - **2:** `Run` `args {neighbor} {address}`, `Exited(0)`, then `Run` `env` with row 96's two values, `Exited(0)`. The peer has two observations.
   - **3:** `Run` `wait {address}` opening, `Exited(0)`. Keeps its `link`.

   `Outcome::Done`, then each person's own kept entries, by `person`, which is row 98's assertion. The fixture's server answers any host, and row 73 asserted no more of the example's: the report says so.
3. **Two rows at step level,** in `failure_tests.rs`: `a_failed_keep_reports_its_own_step` takes a selector and an attribute per row, and gains `("#missing", None, NoElement, "keep selector #missing attribute value: no element")` and `("input", Some("absent"), NoAttribute, "keep selector input attribute absent: no attribute")`. Those are rows 81 and 82's words. No browser, and no process.
4. **`browser_harness.rs`** is left with two tests:
   - `the_harness_fails_a_page_whose_module_throws`, as it is.
   - `people_read_a_bundled_page_and_pages_of_another_origin`: one `fixture_bundle` whose module draws what rows 69, 70 and 102 need (the `h1` `Ready`; the number input `Count`; the field `Title` holding `Old words`, with its `h1` and its `input` listener); `fixture_harness_with` one origin at `localhost`, its pages `/` (`Elsewhere`), `/framing` (row 72's iframe) and `/keep` (`KEEP_PAGE`); three people:
     - **0:** rows 69, 70 and 102's steps, each `within` as it is.
     - **1:** `Visit` `/`, see `Elsewhere`; `Visit` `/framing`, `Trouble` `frame-ancestors`.
     - **2:** `Visit` `/keep`, then row 80's four keeps.

     `Outcome::Done`; the entries with `person: 2` read row 80's values, each failure naming its selector and attribute.
5. Remove what's left unused (`READY`, `ready_person`, `keep_harness`, `keep_script` and so on), here and in `lib.rs`. `use`s go with them.
6. **Bite,** each once, on `harness::` tests, and restore:
   - `substitute` ignores the peers (`lines.rs`): `people_run_commands_beside_their_pages` fails on the child's parse.
   - `Process::input` sends nothing and doesn't finish on `Stdin::Closed` (`commands.rs`): `a_held_line...` hits its deadline.
   - `run_lending` doesn't `kept.clear()` (`harness.rs`): `keeps_append...` fails on the second run's entries.
   - The fake doesn't fetch a navigated address: `people_run...` fails, person 3 waiting on its signal.

   These edits outside your files are for the bite alone.
7. Remove the status line.
8. Your run's checks: `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-ui-web-tests -p skein-fake-person-browser --all-targets -- -D warnings`, `heavy cargo nextest run -p skein-ui-web-tests -p skein-fake-person-browser`, which selects no `browser_*` binary. Commit and push.

## Report

Your run's text, with the 13 tests' times; what each bite showed; the steps whose numbers changed; and that `browser_harness` was compiled and not run.
