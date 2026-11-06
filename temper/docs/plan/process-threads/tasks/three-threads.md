+++
id = "three-threads"
part = "threads"
kind = "feature"
needs = []
files = [
  ".config/nextest.toml",
  "temper/docs/design/domain/testing.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "medium"
minutes = 15
+++

# `process-temper` at three threads

## Goal

Run the process tier three tests at a time, as `temper/docs/design/domain/testing.md`, 2.6 already says, and keep apart the one more story that holds the sign-in callback's fixed port. Say in the status files that the tier is no longer serial. The next after-landing run on main proves it, not this task.

Acceptance:

- `[profile.process-temper]` has `test-threads = 3`. Its `global-timeout`, every other profile and `tools/tiers/budgets.toml` are unchanged.
- The `temper-sign-in-callback` override selects four tests, listed by the command in *Steps*: the three of `fake_backends::walk::sign_in` and the status walk named below.
- Both status files read as given below.
- `heavy cargo nextest run -p fabrig-boundary` passes, with a nonzero count, and `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing broken.
- No process, browser or live story has run (`notes/common.md`).

## Inputs

- `temper/docs/design/domain/testing.md`, 2.6, **Three at a time**: read that point alone. It needs no edit.
- `temper/docs/plan/process-threads/notes/common.md`.
- `temper/docs/plan/process-threads/plan.md`, *What the reading found*: the tier has been read for what relies on running alone, and the status walk below is the one finding. Don't repeat that reading.

## Owns and may change

- `.config/nextest.toml`: the `process-temper` profile and its override, and nothing else in the file.
- `temper/docs/design/domain/testing.status.md` and `temper/docs/design/STATUS.md`: the sentences below alone. **This departs from your text's "Not yours: any `docs/design/`":** the entry is true until your change lands, so it goes in the same branch, and the words are the lead's, given here.

Leave these as they are:

- `temper/docs/design/domain/testing.md`: it already says three.
- `.github/workflows/macos.yml` and `temper/docs/design/draft/macos.md`: their "alone" is one selected story, which three threads don't change.
- `tools/tiers/**`: nothing there reads the thread count. `tools/tiers/cpu.py` copies the configuration through.

Options, each already decided:

- If either status passage has moved or been reworded by a merge, edit it where it now stands, keeping the given words for what it says of this tier, and say so in the report.
- If `cargo nextest list` refuses `--ignore-default-filter`, read the names from the test binary's own `--list` instead, which runs no story either.
- If you happen on another story that relies on running alone: a fix of a few lines is yours, named in the report. Anything larger goes under `## Left`, with its file and why. Don't go looking.

## Steps

1. In `.config/nextest.toml`, make the profile and its override read:

   ```toml
   # Three at a time (temper/docs/design/domain/testing.md, 2.6).
   [profile.process-temper]
   default-filter = "binary(=process) & (package(/^temper(-|$)/) | package(=git-transcripts))"
   global-timeout = "25m"
   test-threads = 3

   # The walks that sign in to ChatGPT on the fake box, where the callback's port is fixed.
   [[profile.process-temper.overrides]]
   filter = "package(=temper) & binary(=process) & (test(/^fake_backends::walk::sign_in::/) | test(=fake_backends::walk::status::a_plans_project_has_its_brief_on_a_return_past_the_sitting))"
   test-group = "temper-sign-in-callback"
   ```

   Why the status walk joins: it runs `account connect 1` on `Walk::owned`, as the sign-in walks do (`temper/crates/temper/tests/fake_backends/walk/status.rs`). The fake box's registration fixes the callback at 1455 (`temper/crates/temper-fake/src/backends.rs`), and a fixed binding has no fallback when the port is taken (`skein/crates/skein-oauth-accounts/src/listener.rs`, `Binding::Fixed`). So it can fail against a sign-in walk running beside it. The other `account connect` stories in the tier go through `support::sign_in::patch_configuration`, which removes `redirect_port`, so they don't hold 1455 and don't join. Name the walk exactly: the rest of `walk::status` doesn't sign in and stays free to share.

2. In `temper/docs/design/domain/testing.status.md`, replace the text from `- **Not yet:** 2.6: the process tier stays at one test at a time` through `2.6: Goal and Away keep the shipped 5 s poll, where that request has them` with this. What follows those last words (`wake on an event or take the status walks' paces...`) stays as it is:

   ```markdown
   - **Not yet:** 2.6: no whole run at three tests at a time has passed: the
     request `process-threads` of 2026-11-06 leaves that to the next after-landing
     run, in place of the three whole runs and a loaded one the request `test-speed`
     of that day asks for. The one sample at three, on a loaded builder, came before
     the retained ports' blocks and the update that decides again: it took 203.7 s
     of wall against 503.9 s at one, failed twice on what those address, and read
     12 tests over 25% more CPU than at one thread (the `test-speed` plan's figures
     on threads, at `b23901eab7`).
     2.6: the tier's CPU against its 360 s: of the seven whole runs in those
     figures, at differing loads, four read over it (367.8 to 433.7 s) and
     three within (287.1 to 349.7 s, the last that sample), and they hold no
     run with the plan's parts together. No budget moved, that being the user's.
     2.6: Goal and Away keep the shipped 5 s poll, where the request `test-speed` has them
   ```

3. In `temper/docs/design/STATUS.md`, replace the sentence that begins `The process tier runs one test` and ends `read over its CPU budget (`domain/testing.md`).` with:

   ```markdown
   The process tier's three tests at a time have no whole run on record until the next
   after-landing run, and most of the `test-speed` plan's whole runs read over its CPU
   budget (`domain/testing.md`).
   ```

4. Check:

   ```
   python3 -I -B -c "import tomllib; p = tomllib.load(open('.config/nextest.toml', 'rb'))['profile']['process-temper']; assert p['test-threads'] == 3 and p['global-timeout'] == '25m'"
   heavy cargo nextest list -p temper --test process --ignore-default-filter -E 'package(=temper) & binary(=process) & (test(/^fake_backends::walk::sign_in::/) | test(=fake_backends::walk::status::a_plans_project_has_its_brief_on_a_return_past_the_sitting))'
   heavy cargo nextest run -p fabrig-boundary
   python3 -I -B tools/citations/citations.py --base origin/main
   ```

   The list prints four tests. With three, the status walk's name is wrong: find its name in the binary's list and correct the filter.

   This branch changes no Rust unless you took the option above, so `cargo fmt`, clippy and `old_forms.py` have nothing to check; run them only if you did, for what you changed. No new test, so nothing to bite: the filter is held by the list above, and the threads by after-landing.

5. Commit, saying why, and push.

## Report

Beyond the usual: the four names the list printed, and any further story you met that relies on running alone, fixed or left.
