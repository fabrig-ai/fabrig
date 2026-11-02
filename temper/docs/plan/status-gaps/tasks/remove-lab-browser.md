+++
id = "remove-lab-browser"
part = "lab-browser"
kind = "migration"
needs = []
files = [
  "skein/testing/skein-lab-browser/**",
  "skein/testing/skein-browser/**",
  "skein/tests/browser/**",
  "skein/crates/skein-ui-web/src/mapping.rs",
  "skein/crates/skein-ui-web/src/tests/mapping.rs",
  "skein/testing/skein-fake-person-browser/**",
  "skein/tests/clippy.toml",
  "skein/tests/forge/github/clippy.toml",
  "skein/tests/forge/conformance/clippy.toml",
  "Cargo.toml",
  "Cargo.lock",
  ".config/nextest.toml",
  "tools/tiers/packages.toml",
  "tools/tiers/design.md",
  "skein/docs/design/lab-browser.md",
  "skein/docs/design/lab-browser.status.md",
  "skein/docs/design/browser.md",
  "skein/docs/design/browser.status.md",
  "skein/docs/design/ui-web.md",
  "skein/docs/design/ui-web.status.md",
  "skein/docs/design/testing.md",
  "skein/docs/design/README.md",
  "skein/docs/design/STATUS.md",
]
effort = "high"
minutes = 100
+++

# Remove the retired lab's browser, the look's reads and the mapping back

## Goal

The lab is retired. Remove what served it alone, in skein: the crate
`skein/testing/skein-lab-browser`, the browser kit's look reads (`skein_browser::boundary::Look`,
`browser.md`, 5.1) and `skein_ui_web::mapping::person` (`ui-web.md`, 14.2's mapping back), with
their tests, their design (`lab-browser.md`) and the status entries that record them
(`lab-browser.status.md`; `browser.status.md` 5.1; `ui-web.status.md` 14.2; `STATUS.md`).

Acceptance:

- `skein/testing/skein-lab-browser/` is gone; the root `Cargo.toml` no longer lists it,
  `Cargo.lock` has no `skein-lab-browser` package, `.config/nextest.toml`'s `browser-skein`
  filter and `tools/tiers/packages.toml`'s `browser-skein` set no longer name it, and
  `tools/tiers/design.md` 2.1's list of binaries' packages drops it.
- `skein_browser::boundary` has no `Look`, `Looked`, `Scheme`, `Motion`, `Entry`, `Property`,
  `Face`, no `Request::Look` and no `Event::Looked`; `skein-browser/src/browser/look.rs` and
  `look_tests.rs` are gone, with every arm, step and helper that served only the look (the
  caret stylesheet for shots, the look's CDP domains' enabling if only the look used them).
- `skein_ui_web::mapping::person` and its test are gone.
- No `clippy.toml` names `skein_browser::boundary::Look`.
- `rg -n 'skein-lab-browser|skein_lab_browser|lab-browser\.md|boundary::Look\b|Looked\b|mapping::person' --glob '!temper/docs/plan/**' --glob '!**/docs/development/**' --glob '!target/**'`
  finds nothing.
- The design: `lab-browser.md` and `lab-browser.status.md` are deleted; `browser.md`, `ui-web.md`,
  `testing.md`, `README.md` and `STATUS.md` say nothing of the lab's browser or the look's reads
  (below); `browser.status.md` and `ui-web.status.md` hold only their other gaps.
- clippy is clean and the focused tests pass for every package changed (Steps).

## Inputs

- `temper/docs/plan/status-gaps/notes/common.md` (this task edits the skein design files listed
  in `files`, as below).
- `skein/AGENTS.md`.
- The status entries: `skein/docs/design/lab-browser.status.md`, `browser.status.md` (5.1),
  `ui-web.status.md` (14.2), `STATUS.md` (its summary's clause "where the lab's browser, the
  look's reads and `ui-web`'s mapping back from a browser role serve the retired lab ...").
- `skein/docs/design/browser.md`, 5.1 (what goes) and its sections 1, 4, 6, 8, 9 and 10 for the
  lines that name it; `skein/docs/design/ui-web.md`, 14.2, the bullet "and back, for the lab's
  hand".

## Owns and may change

Owns (skein, and the workspace's lists of packages):

- `skein/testing/skein-lab-browser/**`: deleted whole.
- `skein/testing/skein-browser/**` (`skein-browser`): `boundary.rs`, `browser.rs`,
  `browser/look.rs`, `browser/look_tests.rs`, `limits.rs`, `command.rs`, `wire*`, `lib.rs` as the
  look's removal needs. A `Limits` field (`properties`, `entries`, `screenshot` or another) goes
  only if nothing but the look reads it once the look is gone; its constructors change with it.
- `skein/tests/browser/**` (`skein-browser-world`): delete `tests/support/look_world.rs`,
  `look_memory.rs`, `real/look.rs`, `src/tests/look.rs`, the test
  `chromium_reads_the_pages_look_and_captures_a_distant_part_at_double_scale` and its fixture
  (`look_fixture`), the referee's look registry (`src/referee.rs`, `looks`, `look_op`, the
  `Event::Looked` check), and `tests/support/real.rs`'s look state and `Request::Look` /
  `Event::Looked` arms. `browser_facts.rs`'s `Look { components, .. }` is `skein_ui_web`'s look
  (ui-web.md 3.2), not the browser's: keep it.
- `skein/crates/skein-ui-web/src/mapping.rs`: `person` (and a helper only it uses);
  `src/tests/mapping.rs`: the test of `person` and the `person` import.
- `skein/testing/skein-fake-person-browser/**`: `clippy.toml`'s `Look` entry; `src/origins.rs`'s
  `Pages::verbatim` (only the lab calls it; its doc cites `lab-browser.md`, 2.3) with its test
  `verbatim_origins_serve_placeholders_as_written` in `src/tests/origins.rs`, and the
  `verbatim` field and its branch if nothing else sets it.
- `skein/tests/clippy.toml`, `skein/tests/forge/github/clippy.toml`,
  `skein/tests/forge/conformance/clippy.toml`: the `skein_browser::boundary::Look` line alone.
- `Cargo.toml` (the member line), `Cargo.lock` (by cargo, not by hand), `.config/nextest.toml`
  (line ~116: drop `| package(=skein-lab-browser)` from the `browser-skein` filter, nothing else),
  `tools/tiers/packages.toml` (the `browser-skein` entry, and a `[binaries]` record of the
  crate if one exists), `tools/tiers/design.md` (line ~69: drop `skein-lab-browser` from the
  list; the sentence stays true of the rest).
- The skein design, as the request asks, in the design's words of now:
  - delete `skein/docs/design/lab-browser.md` and `lab-browser.status.md`;
  - `browser.md`: remove section 5.1 whole (its heading too; the number stays unused, nothing
    renumbered), and in the rest every sentence, table row or clause about the look or the lab's
    browser: section 1's "The lab's browser (lab-browser.md) uses it too, and is alone in using
    its look reads (5.1)" and the bullet "The look's reads are the lab's, never a test's"; 4's
    `Emulation` / `CSS` / `Accessibility` mentions that serve only the look, the `Look` request
    and `Looked` event lines, and `Looked` in its list; 6's five `look:` table rows and the
    look's sentences on waiting; 8's "a look's tree entries and properties"; 9's look tests and
    their fixture (the "lab's browser's story names" line); 10's `skein_browser::boundary::Look`
    disallowed-types item. Keep every other word; where a list loses an item, its grammar is
    mended, nothing more;
  - `browser.status.md`: its 5.1 entry goes; with no other gap it says `No gap.`;
  - `ui-web.md` 14.2: the bullet "and back, for the lab's hand (lab-browser.md, 8.4) ..." goes;
  - `ui-web.status.md`: the 14.2 "Departs" entry goes; 18's "Not yet" stays word for word;
  - `testing.md`: the crate list's `testing/skein-lab-browser` line goes;
  - `README.md`: the table row "the lab's browser" goes;
  - `STATUS.md`: the summary loses the clause on the lab's browser, the look's reads and the
    mapping back (the sentence's other clauses stay, mended), and the index loses
    `lab-browser.md`'s line; `browser.md`'s index line stays.

Readers in other trees, checked, not edited: `jig-client-fake-person-browser`,
`jig-client-browser-world`, `temper-client-journeys`, `temper-client-browser-world` depend on
`skein-browser` or `skein-fake-person-browser`; the survey found none naming `Look`,
`Pages::verbatim` or a look `Limits` field (they use `boundary::Key`, `Go::Reload`). If one of
them stops compiling, fix its use only if the fix is a removed field in a `Limits` literal, and
say so; anything else is a question.

Pre-approved options:

- If a test that is not about the look uses a look read for its own check (as
  `tests/support/real.rs` asks `Look::Tree` after a snapshot when its scenario holds a `tree`),
  delete that check with its scenario field if the scenario exists for the look; otherwise
  keep the scenario and its other checks, and list the dropped check in the report.
- If a CDP domain (`Emulation`, `CSS`, `Accessibility`, `DOM`) is enabled for the look and for
  something else, keep its enabling; remove only what the look alone used.
- If `skein/scripts/old-forms.txt` counts fall (`python3 -I -B scripts/old_forms.py` from
  `skein/`), write them (`--write`); they must not rise.
- jig's `protocol/browser.md, 5.1` and "jig's look" in `jig-client-shell-web` are jig's own
  document and its client's look: unrelated, untouched.

## Steps

1. Delete the crate; drop it from `Cargo.toml`, `.config/nextest.toml`,
   `tools/tiers/packages.toml` and `tools/tiers/design.md`. `cargo metadata --format-version 1
   >/dev/null` (builds nothing) refreshes nothing; let the first `heavy cargo check` update
   `Cargo.lock`, and check `git diff Cargo.lock` removes the package alone.
2. Remove the look from `skein-browser` and its tests from `skein-browser-world`; remove
   `Pages::verbatim`; remove `mapping::person`; drop the `clippy.toml` lines. Compile with
   `heavy cargo check -p skein-browser -p skein-browser-world -p skein-fake-person-browser -p skein-ui-web -p skein-ui-web-tests --tests`
   while errors last.
3. Narrow tests: `heavy cargo nextest run -p skein-browser --lib`;
   `heavy cargo nextest run -p skein-ui-web --lib`;
   `heavy cargo nextest run -p skein-fake-person-browser --lib`;
   `heavy cargo nextest run -p skein-browser-world` (the focused profile; its `browser_*`
   binaries need Chromium and belong to the browser tier: they must build, and run only if the
   focused profile selects them). Check each count is not zero where tests should run.
4. The tiers' map: `tools/tiers/run --verify skein browser`; report its output. If it needs a
   builder you lack, report that and leave it to fab's check.
5. Cross-tree readers: `heavy cargo check -p jig-client-fake-person-browser -p jig-client-browser-world -p temper-client-journeys -p temper-client-browser-world --tests`.
6. `cargo fmt`; `heavy cargo clippy -p skein-browser -p skein-browser-world -p skein-fake-person-browser -p skein-ui-web -p skein-ui-web-tests --all-targets -- -D warnings`.
7. The design and status edits above; then the acceptance's `rg`.
8. Commits: the crate's removal with the workspace lists; the look's and `person`'s removal with
   their tests; the design and status text. Push your branch.

## Report

As the implementer's text says, and also: each `Limits` field removed or kept and why; each
CDP domain's enabling removed or kept; any check dropped under the first option; the
`--verify` output; the `old-forms` counts before and after.
