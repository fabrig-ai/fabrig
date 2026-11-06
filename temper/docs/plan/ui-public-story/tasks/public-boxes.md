+++
id = "public-boxes"
part = "public-story"
kind = "feature"
needs = []
files = [
  "temper/tests/client-browser/tests/browser_all/browser_public.rs",
  "skein/testing/skein-fake-person-browser/src/face.rs",
  "skein/testing/skein-fake-person-browser/src/tests.rs",
  "skein/docs/design/browser.md",
  "temper/docs/design/client/browser.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 45
+++

# The public geometry story measures from one boxed query, and its cause is read once

## Goal

`browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` has failed in about half of after-landing's runs since `ui-public-2` landed, at `TextBoxes::next`: `visible text has a box`. The repair is done on a branch fab can't land. Merge it here, keep its last question answered as the request answers it, and read the cause once from what the merged story prints.

This task departs from the implementer's text, and from the plan's rule that a slow tier's stories are not run, in exactly three ways:

- **It runs this one story,** by name, alone, in the tier's profile, because it repairs that story's red (`AGENTS.md`, 5). It never runs the tier, `tools/tiers/run`, `scripts/browser.sh` or another story of the tier, and reads no tier CPU figure.
- **It reruns that story unchanged,** where the rule is no unchanged rerun: at most five runs to show a flake, and at most five more to prove a fix, if there is one.
- **It carries design lines:** the three design files in `files` arrive by the merge, and the task writes the status entry and the summary sentence below.

Acceptance:

1. `d3ca9d872e` is an ancestor of your branch, and `git diff --stat <your base> HEAD` names the six files of the merge and `temper/docs/design/STATUS.md`, with only what an option below adds. Nothing of the idle-connection work differs from your base: main's `c592067aa0` holds it, and the branch's reverts cancel on a merge.
2. `BrowserFace::boxed_matches` and its two regressions are in, `a_boxed_find_keeps_every_match_and_reports_truncation` and `a_boxed_find_measures_its_matches_without_measuring_the_scope`, and both pass.
3. `skein_browser::boundary::Seen` and `parse_rect`'s reading of the first quad are unchanged, and `browser.status.md` holds the 8.4 entry given in Step 1.
4. `TextBoxes::check` is unchanged, and so is every field, gutter, wrapping, navigation and overlap assertion. No budget, deadline or tier membership changes.
5. The story ran by name, alone, at most five times, and the report gives each run's outcome. Either no missing box showed, and the report says the original cause stays unreproduced rather than fixed; or one showed, the report names its text and stage, the fix is in with a focused test where the options below ask for one, and the story passes its proving runs. A fix you can't finish is `Failed:`, with the text, the stage, the reply and what you ruled out, your work pushed.
6. The report says whether sign-out step 19's unexpected `/jig/v1/stream` 401 recurred, and it is fixed only if it did.

## Inputs

- `temper/docs/plan/ui-public-story/inputs/request.md`, whole. It holds the plan's standing rules.
- `temper/docs/design/client/browser.md`, 8.4, as the merge leaves it; 8.2, *It reads again by itself*.
- `temper/docs/design/client/browser.status.md`, 8.4.
- `skein/docs/design/browser.md`, 5, the query's `boxes`, as the merge leaves it.
- `tools/tiers/design.md`, 2, the `browser-temper` row: what the story's run needs built.

Red-50's own report sits on another machine and you need nothing more of it than this spec. Its mutation checks of the two regressions were made and reported there: don't repeat them.

## Owns and may change

- **Owns:** this spec's `files`.
- **Not to change, whatever the runs show:** `Seen`; the first-quad reading (the request's answer, option 2: it stays the limit the status records); `tools/tiers/**`; `.config/nextest.toml`; `browser_pages.rs`, which `browser-cap` follows on.
- **Readers of what the merge changes,** already correct and read at the cut: `find_boxed`'s other callers, `harness/script/keep_box.rs` and `harness/script/growth.rs`, read the terminal node's box alone, so a scope that now asks no rectangles doesn't touch them. `TextBoxes` is the only reader of `boxed_matches`. This is a change of meaning for scopes, so the focused suite in Step 5 runs every package that reads the kit.

**What can make a box missing, read at the cut.** Nothing was run for this, and which one happened is unknown.

- `Seen.rect` is `None` only when `DOM.getContentQuads` answered and `parse_rect` (`skein/testing/skein-browser/src/browser.rs`) read nothing from the answer: no `quads`, an empty `quads`, or a coordinate `whole_pixels` refuses, which is an exponent form (`skein/testing/skein-browser/src/wire/decode.rs`).
- A node that has left the document doesn't reach that path. Its error reply goes through `command_error` to a refusal, and `BrowserFace::refused` answers the search `Found::Absent`. So the request's likely reading, a page reading again on a timer, would show in the merged story as `Found::Absent`: `TextBoxes::next` ends the stage with no boxes, and `check` fails on `long title stays readable:` with empty text, after that stage's `measure public text:` line. It would not print `has no box`.
- Stages 3 and 5 measure straight after `Step::Viewport`, with no `see`. Stages 2 and 4 see their page first.

**Options, each pre-approved, by what the failure shows:**

- **`visible text "…" has no box`, with a whitespace-only name or a zero-size node the page really draws:** fix what the query counts. Default: a small predicate beside `boxed_matches` in `skein-fake-person-browser`, true for a name holding a non-whitespace byte; `TextBoxes::next` skips what it rejects; a regression in `src/tests.rs`; one design line in `skein/docs/design/browser.md`, 5. Substantive text with no box still panics.
- **`has no box`, with substantive text:** it stays a failure and is never skipped. Read the reply first: a throwaway `eprintln!` in `Step::Box` on a `None`, gone before you commit, with its run counted among the five. Then, by what it shows:
  - An exponent-form coordinate: `whole_pixels` reads that form, with a unit test beside its own in `wire/decode.rs`. `skein/testing/skein-browser/src/wire/decode.rs` becomes yours.
  - Stage 3 or 5, and an empty `quads`: that stage sees `LONG_TITLE` after its `Step::Viewport` before it measures, as 2 and 4 do. The story's file is yours already.
  - Text the page draws with no box: fix it in the lowest tier that owns it, with that tier's test. `temper-client-view` or `temper-client-shell-web/temper.css` (under `/* ==== ui: public ==== */`) become yours, for that fix alone.
- **`long title stays readable:` with empty text after a `measure public text:` line:** a refused search, which is not a page without text, since the stage's page was just seen. Default: `TextBoxes::next` takes `now` and asks again on `Found::Absent` until `WITHIN` past the stage's first ask, then ends the stage as it does today, so `check` fails. Name this in the report as the request's reading, shown.
- **Sign-out step 19's 401 recurs** (`see(tier.states().signed_out)` in `owner_steps`): read its log. Default: a 401 on a stream reopened around sign-out is the server's right answer to a session that has ended, so `Walk::expected` gains the words, as `browser_sign_in.rs` lists `401` in its setup. `expected` is read once at the start and matched on the console's text, so it hides any 401 of this story's reader: say so in the report. If the log shows the client reading with no reason after it signed out, fix that order in the lowest tier instead, with its test.

A design line a fix makes untrue is yours to change, and named in the report. Anything past these options that truly stops your acceptance is a `Question:`.

## Steps

1. **Merge.**

   ```sh
   git fetch origin watch-room/red-50
   git merge --no-ff d3ca9d872e
   ```

   The one conflict is `temper/docs/design/client/browser.status.md`, as a trial merge at the cut showed. Take main's side of what is already there: keep main's blank line and its `7.2` entry on `accounts-try-stopped`. Drop main's 8.4 entry, the story that "is compiled" and "awaits the browser tier": after-landing runs that story now. Keep the branch's 8.4 entry, word for word:

   ```text
   - **Not yet:** 8.4: the browser kit returns the first content quad of each text
     node, so the public geometry story cannot check every wrapped fragment.
     Its result needs a browser testing interface decision before it can retain those
     fragments. Missing rectangles remain failures.
   ```

   In `temper/docs/design/STATUS.md`, the summary says the same stale thing. Replace its sentence "The public geometry story's runtime check awaits the browser tier after landing" with "The public geometry story checks a wrapped text by its first quad alone", keeping the citation that follows. Check acceptance 1 now, against `git diff --stat`.

2. **The regressions.**

   ```sh
   heavy cargo nextest run -p skein-fake-person-browser --lib -E 'test(a_boxed_find)'
   ```

   Two tests, at least, must be selected.

3. **Build what the story needs, once,** from the repository's root. `T` is `cargo metadata`'s `target_directory`, and `S` is `temper/crates/temper-client-shell-web`. Build again only if the client's code changes.

   ```sh
   TEMPER_BUNDLE=none heavy cargo build --target wasm32-unknown-unknown --features loopback-navigation,small-window -p temper-client-shell-web
   TEMPER_BUNDLE=none heavy cargo build -p skein-ui-bundle
   "$T/debug/skein-ui-bundle" --module "$T/wasm32-unknown-unknown/debug/temper_client_shell_web.wasm" \
     --name temper_client_shell_web --prefix /bundle/ --title temper \
     --placeholder "$S/loading.html" --placeholder-stylesheet "$S/loading.css" \
     --stylesheet "$S/temper.css" --stylesheet "$S/task-pages.css" --files "$S/files" \
     --noscript 'temper needs WebAssembly and scripts.' --out "$T/temper-bundle"
   TEMPER_BUNDLE=none heavy cargo build -p temper --bin temper
   ```

   This story reads no playground bundle.

4. **Read the cause: at most five runs, stopping at the first that shows a failure you must act on.** The profile holds the tier's timeouts and one thread, and its filter admits the binary. Check the count is one test each time.

   ```sh
   TEMPER_BUNDLE=none TEMPER_TEST_BUNDLE="$T/temper-bundle" NEXTEST_DOUBLE_SPAWN=0 \
     heavy cargo nextest run --profile browser-temper -p temper-client-browser-world --test browser_all \
     -E 'test(=browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths)'
   ```

   Read each failure's log before the next run. A failure names its text in the panic and its stage in the `measure public text: stage=… width=… task=…` line before it. Then follow the options. Five clean runs end this step: the report says the cause stays unreproduced, and after-landing's next runs are the check.

5. **Finish,** once: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; then, one command at a time:

   ```sh
   heavy cargo clippy -p skein-fake-person-browser -p temper-client-browser-world --all-targets -- -D warnings
   heavy cargo nextest run -p skein-fake-person-browser -p skein-browser-world -p skein-ui-web-tests -p jig-client-fake-person-browser -p jig-client-browser-world -p temper-client-journeys -p temper-client-browser-world
   ```

   Add any package an option made yours, to both. A new test you add to pin a rule must bite, as the implementer's text says. Commit, with the status and summary edits in the merge or in one small commit of their own, and push.

## Report

The implementer's report, and also:

- That the merge holds what red-50 said it held, and what you took from main's side.
- Each story run: its number, its outcome, its wall time, the machine, its log's path.
- The cause: the text and the stage if a box went missing, and which option you took; or that it stayed unreproduced, in those words. Say whether what you saw bears out the request's likely reading.
- The sign-out flake: whether it recurred, and what you did.
- What stays open: the first-quad limit is the status's 8.4 entry, by the request's answer.
