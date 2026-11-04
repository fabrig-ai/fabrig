+++
id = "browser-walk"
part = "browser"
kind = "feature"
needs = ["bundle-playground", "native-walk-a"]
files = [
  "temper/tests/client-browser/Cargo.toml",
  "temper/tests/client-browser/src/bin/playground-serve.rs",
  "temper/tests/client-browser/src/lib.rs",
  "temper/tests/client-browser/tests/browser_all/main.rs",
  "temper/tests/client-browser/tests/browser_all/browser_playground.rs",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/docs/design/client/browser.md",
  "tools/tiers/run",
  "tools/tiers/test_run.py",
  "tools/tiers/design.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/development/playground-figures.md",
  "Cargo.lock",
  "Cargo.toml",
]
effort = "high"
minutes = 150
+++

# The browser tier walks story A on the built playground and measures it

## Goal

"In a browser, outside the gate: the browser tier walks story A on the built playground in
headless Chromium (`client/browser.md`, section 4), and measures the download from the bundle's
files (raw and compressed), and the memory's high water and the longest slice from what the frame
shows, reported against their bars" (`temper/docs/design/playground.md`, 9); the twelfth test of
the tier, "its log downloaded and replayed natively" (`temper/docs/design/client/browser.md`,
4.6, 12). The tier's runner builds the playground (`tools/tiers/design.md`, 2), its own static
host serves it (`notes/contracts.md`, C8), and the figures go to the user, who sets the bars
(`playground.md`, 8 and 11).

Acceptance:

- `tools/tiers/run`'s temper bundle step, for `browser-temper` alone, also runs
  `temper/scripts/bundle.sh --playground <target>/temper-playground` (its cargo builds through
  heavy as the step's other builds are) and sets `TEMPER_TEST_PLAYGROUND` to its absolute path;
  `qa-temper` is unchanged; `tools/tiers/test_run.py` covers the new step and variable
  (`python3 -I -B tools/tiers/test_run.py`, as the file runs today) and passes.
- `temper-client-browser-world`'s binary `playground-serve` (C8), with a unit test or two in
  `temper/tests/client-browser/src/lib.rs`'s tests (or beside the binary) of its mapping: a digest
  file with its `headers` lines and the long cache, any other path the page with `no-cache`, the
  policy fields on both, a path outside `OUT` refused.
- `temper/tests/client-browser/tests/browser_all/browser_playground.rs`,
  `story_a_walks_on_the_built_playground` (its `mod` line in `main.rs`): through skein's browser
  harness (`skein-fake-person-browser`'s `Setup`, `server` the `playground-serve` binary
  (`env!("CARGO_BIN_EXE_playground-serve")`), `args` the playground's `OUT`, `downloads` a scratch
  directory), the page opened at `/`: the frame and the loading line, then the welcome; the
  frame's 10× pressed; story A walked by the native walk's person
  (`temper_playground_world::stories::a()`) to the CSV change landed and the other two answered;
  no CSP violation; then "Download the session log" pressed, the downloaded log replayed natively
  as the native walk replays the kept one (`story_a_kept_log_replays_natively`): `Ok`. It prints
  one line `playground figures: download raw <B> B gzip <B> B; memory high water <B> B; longest
  slice <ms> ms`: the download summed over `OUT`'s files (gzip by `skein_http::gzip::encode`
  with its `bound`), the memory and the slice read from the frame's text.
- The tier holds 12 tests within its cap (`browser.md`, 4.3: 120 seconds); `scripts/browser.sh`
  (temper's, through `tools/tiers/run temper browser`) passes; over the cap is red, repaired by
  the testing strategy's three questions (`notes/common.md`), the frame's speed first.
- `temper/docs/development/playground-figures.md`: the figures of one run (the date, the host,
  Chromium's version), each against its bar where the user set one (the download's 6 MB gzipped)
  or marked "the user's, Open" (memory, slice); no bar is enforced by the test.
- Status, in place, never reflowed, date lines today's: `tools/tiers/design.status.md`'s **Not
  yet** 2 removed (the file then says "No gap."); `temper/docs/design/client/browser.status.md`'s
  **Not yet** "4.6: item 12, the playground's story A on the built playground and its log replayed
  natively (`../playground.status.md`)." removed.
- `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`;
  `temper/docs/plan/playground-browser/notes/contracts.md`, C3, C5, C6, C7 and C8.
- `temper/docs/design/playground.md`, 2, 6, 8 and 9; `temper/docs/design/client/browser.md`,
  4, 4.3 and 4.6; `tools/tiers/design.md`, 2 and 2.1.
- skein's browser kit: `skein/testing/skein-fake-person-browser/src/harness.rs` (`Setup`,
  `downloads`), `skein/testing/skein-browser/src/browser/download.rs`; the tier's crate as it
  is, `temper/tests/client-browser/src/lib.rs` (`Tier::setup`, `chromium`, `bundle`) and one of
  its tests (`tests/browser_all/browser_sign_in.rs`).
- A server the harness runs and reads: `temper/tests/client-browser/src/lib.rs`'s `printed`
  setup (~260) and skein's example server (`skein/examples/ui/server`, package
  `skein-ui-example-server`) for how a server prints its address.

## Owns and may change

- The files in `files`. `Cargo.toml` gains the `[[bin]]` (or `src/bin/` discovery) and the
  dependencies the test uses (`temper-playground`, `temper-playground-world`, `skein-playground`,
  `skein-ui-web`, `skein-http`), workspace dependencies.
- `tools/tiers/run` and `tools/tiers/test_run.py`: the temper bundle step only; the design is
  written (`tools/tiers/design.md`, 2). `tools/tiers/packages.toml` stays: the server is the
  test's own package's binary, which nextest builds.
- Not yours: the page, the bundle script, temper-playground; a change they would need is a
  question.
- Pre-approved options:
  - **The person in Chromium:** if skein's `Person` does not run over the browser's face as it
    does over a tree, write story A's steps for the browser harness's own play, the same words,
    in `browser_playground.rs`, and say so.
  - **Reading the figures:** the frame's text by the browser face's `Find::text`; if the frame
    shows them only in a form the face cannot read, stop and ask: the frame is the page's task's.
  - **Over the cap:** at 10× first; if story A cannot fit at any of the frame's speeds, report the
    measured times and stop and ask (the cap is the user's).
  - **The playground's build time** in the tier's setup is not the tests' time; report it.

## Steps

1. `playground-serve` and its tests.
2. The tier's runner and its test.
3. The browser test; run the tier once (`temper/scripts/browser.sh`); read its CPU and wall.
4. The figures' document; the status edits; the citations check.
5. `cargo fmt`, old-form count, clippy; commit per step; push.

## Report

As the implementer's text says, and also: the figures' line; the tier's wall and CPU with 12
tests; the playground's build time in the setup; the speed used; Chromium's version.

## Answers

2026-11-03. **`temper-playground-world` is no workspace dependency: add it to the root
`Cargo.toml`, or a path dependency in `client-browser/Cargo.toml`?** Add it to the root manifest;
no path dependency. Test worlds already sit among the workspace dependencies
(`temper-worker-protocol-git-world`), and every dependency of this crate is a workspace one.
The root `Cargo.toml` joins this task's files for this alone: one line,
`temper-playground-world = { path = "temper/tests/playground" }`, in `[workspace.dependencies]`
beside `temper-playground`; no other change there. Then verify 3b1099ce84's draft and go on as the
acceptance says.

2026-11-04. **Native story A fails at the same startup and its kept log is absent: does
native-walk-a repair them first, or may this task change the playground?** Neither: native-walk-a
has landed, and its repairs are what this branch lacks. The branch last merged native-walk-a at
`eaf89d4b6f`, before the startup repairs and the kept log; the trunk holds them (`24000c6ed8`,
`9678bee8e1`, `3133264738`, and `tests/fixtures/story-a.log`). Its last run reports the walk and the
replay passing: 6 passed, 1 skipped, in `temper-playground-world`, world time 44.8 s, 17 person
steps. So first merge `origin/playground-browser/integration` into this branch, keeping its commits
and its name, and run `heavy cargo nextest run -p temper-playground-world --test suite -E
'test(/walk::/)'`: three pass, or it is a question, since native-walk-a is landed and closed. Then
go on as the acceptance says, from the browser test. The merge changes the person's script:
`stories::a()` now presses each suggestion straight into its chat and sees `Landed` there, with a
wait of 60 s; `story_steps` takes it as it stands, its `/chats` arm still needed for the second and
third requests. The merge's one conflict is `tests/browser_all/main.rs`: keep both sides' `mod`
lines. The kept log is the native walk's, recorded by that task: this task never records it.
If the browser still stops where the native walk passes, that is a fault of its own and the task's
boundary holds: a change to the page, the bundle script or temper-playground is a question, with
the cause found in the browser's log and snapshot; the file list does not widen.

2026-11-04. **The merge brings `browser_public`: with the playground's the tier holds 13 tests
against a cap of 12. What leaves?** No test's proof leaves and the cap stays: 4.3's rule applies,
stories that share a setup become one test of several steps. The acceptance's 12 tests cannot hold
otherwise, so `browser_pages.rs` and `browser.md` join this task's files for this alone:
- `browser_pages.rs`: `a_non_page_address_opens_the_inbox_at_slash` goes; its proof,
  `Go /nowhere`, then the `Inbox` heading, then `Step::Address("/")`, becomes the last steps of
  `history_keeps_the_chat_focus_and_filter_across_reload_and_back`, whose name and setup stay. The
  module's first lines stay true.
- `temper/docs/design/client/browser.md`, 4.6: item 2 gains ", and an address that is no page opens
  the inbox at `/` (4.1, 2)"; item 3's line goes, its number unused (`DESIGN.md`, 4); an item 13
  after 12 lists the public reading, "**Reading a public project** (8.4), with no sign-in, and
  losing it when it is made private."; line ~923's "Browser test 3" becomes "Browser test 2". The
  title stays "The twelve": 1, 2 and 4 to 13 are twelve.
- Check the count: `scripts/browser.sh` reports 12 tests, and `browser_public` is run as it stands:
  if it fails, it is not this task's: run it once alone and report it under `## Left` with its log.
