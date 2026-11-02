+++
id = "pair-design"
part = "live-pair"
kind = "docs"
needs = []
files = [
  "temper/docs/design/domain/testing.md",
  "temper/docs/design/domain/testing.status.md",
]
effort = "medium"
minutes = 30
+++

# testing.md says what runs after every landing and what stays the owner's

## Goal

The user decided (11-02) that the two owned-forge landing stories run after every landing,
on shelf logins, and that every other live story stays the owner's. Write it into temper's
testing design, section 2.7, and its status file.

**Departure from the implementer's text:** this task edits `docs/design/` (the two files
below, nothing else), because the request asks for the design text
(`inputs/request.md`, "Tests"). Write what the spec gives, in the design's register: present
tense, no history, no emphasis words; dates only in the status file.

Acceptance:

- `temper/docs/design/domain/testing.md, 2.7` says, in the places below: what runs after
  every landing (the pair, the profile, `--run-ignored only`, the models, no person); one
  `TEMPER_LIVE_STORE` and a store per story; the runner's part; what stays the owner's; that
  harness step checks run once, in `suite`; and the `live-temper` profile's filter in
  "Outside the suites".
- `testing.status.md` gains one dated line for it (below).
- No other file changes. Every name the text uses exists in the code or in
  `notes/contracts.md` (step 3).

## Inputs

- `temper/docs/plan/live-temper/notes/common.md` and `notes/contracts.md` (all of it: the
  text states its calls).
- `temper/docs/plan/live-temper/inputs/request.md`.
- `temper/docs/design/domain/testing.md, 2.7` (lines ~1092-1362), and
  `testing.status.md`.

## Owns and may change

- `temper/docs/design/domain/testing.md`: section 2.7 only, the edits below.
- `temper/docs/design/domain/testing.status.md`: one new line.

Read only: `temper/crates/temper/tests/live.rs`, `live_github.rs`, `live_github_managed.rs`,
`proof_live.rs` (the stories' names), `tests/suite/support_tests.rs`,
`temper/scripts/proof-live.sh`. Two other tasks of this plan change the code and scripts to
`notes/contracts.md` at the same time: describe the contract, not today's code.

Pre-approved options:

- Rewrap and reword within the given text to fit 2.7's flow; keep every fact it states.
- If a sentence of 2.7 not named below contradicts the contract (a store path, the Claude
  model, the profile), fix that sentence alone and list it.

## Steps

1. Edit `testing.md` 2.7:

   a. **The owned-forge landing story** bullet: its store sentence ("Its config, data, cache
      and client roots stay in `TEMPER_LIVE_STORE` (by default
      `$HOME/src/rust/tmp/live-store/forge-land`) on pass or failure.") becomes:

      > Its config, data, cache and client roots stay in its own store, `forge-land` under
      > `TEMPER_LIVE_STORE` (by default `$HOME/src/rust/tmp/live-store`), on pass or failure.

   b. **The owned-forge landing story on Claude Code** bullet: "every role on Claude Code"
      becomes "every role on Claude Code on `claude-haiku-5-5`", and add: "Its store is
      `forge-land-claude` under `TEMPER_LIVE_STORE`."

   c. A new bullet before **Outside the suites**, as given (rewrap freely):

      > - **After every landing,** the owned-forge landing pair, needing no person:
      >   `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` on a held ChatGPT
      >   record (`TEMPER_LIVE_ACCOUNT`) and
      >   `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` on a held Claude
      >   Code login (`TEMPER_LIVE_CLAUDE`, with `claude` on the machine's `PATH`), both
      >   from fab's shelf of logins. fab's after-landing run with `--live` runs profile
      >   `live-temper` under `--run-ignored only`, whose default filter selects these two
      >   alone. Automatic live runs take the cheapest models (the user, 11-01):
      >   `gpt-6-luna` and `claude-haiku-5-5`.
      >   - **One invocation, a store per story:** the runner sets one fresh
      >     `TEMPER_LIVE_STORE`; each story takes its own store beneath it, named by the
      >     script's argument for it, `forge-land` and `forge-land-claude`, so neither
      >     reuses the other's deployment. Each store keeps the story's rules: fresh when
      >     absent or empty, kept when it holds a deployment, and a held record moves into a
      >     fresh store only.
      >   - **The runner's part,** as `scripts/proof-live.sh`'s: a private `TMPDIR`; after
      >     the invocation, whatever its end, the held record returned by
      >     `forge-land/give-back` in `TEMPER_LIVE_STORE` where it exists, both logins
      >     checked by `stat` alone, and any process left under the stores stopped.
      >   - **Its bounds** are each story's own, within the profile's 50 minutes for the
      >     pair. Never at the gate, and never run by an implementer.
      > - **What stays the owner's,** ignored and run by hand: the process tier's chat on a
      >   real model, `a_real_model_answers_a_chat_and_pushes_a_small_change` (a person's
      >   sitting); `rung_b0_a_person_agrees_a_plan_before_the_change_lands` (the rung's
      >   walk); `live_github`'s `a_chat_lands_a_change_on_github_com_through_a_restart`
      >   (github.com); `live_github_managed`'s two; and `proof_live`'s three offline driver
      >   proofs. The owner also runs the pair, one story at a time, through
      >   `scripts/proof-live.sh forge-land` and `forge-land-claude`.
      > - **Harness step checks run once,** in `suite` (`tests/suite/support_tests.rs`): no
      >   live or process test repeats a focused one.

   d. **Outside the suites** becomes:

      > - **Outside the suites:** nextest profile `live-temper`, whose default filter is the
      >   landing pair above, run after every landing; `scripts/proof-live.sh` and
      >   `scripts/proof-live-github.sh` select their own story over it
      >   (`--ignore-default-filter` and their own filter), one at a time. Never at the gate.

2. `testing.status.md`: after the last **Built** line, one line:

   > - **After landing (11-02):** 2.7: profile `live-temper` selects the owned-forge landing
   >   pair alone, run by fab's `--live` under `--run-ignored only` on shelf logins; each
   >   story takes its own store under one `TEMPER_LIVE_STORE`; Claude Code runs on
   >   `claude-haiku-5-5`. The pair's first after-landing result is fab's to record; fab's
   >   runner (its LIVE table, the store, the logins' return) is the agents repository's.
   >   The 42 harness step tests counted as duplicates (the after-landing audit at
   >   de07a0b2c) need no deletion: 1425451fb7 (11-01) moved them into
   >   `suite/support_tests.rs`.

3. Check every name: `rg -n` each test name, profile, script argument and path the new text
   uses, in `temper/crates/temper/tests/`, `temper/scripts/`, `.config/nextest.toml` and
   `notes/contracts.md`. A name only the contract holds (`forge-land-claude` as a store,
   `claude-haiku-5-5`) is expected; any other miss is a question.
4. No test reads these files (`rg -n 'testing.md' temper/crates --glob '*.rs'` names them in
   comments only); no build or test runs. One commit; push your branch.

## Report

As the implementer's text says, and also: each sentence of 2.7 you changed beyond step 1,
and step 3's misses, if any.
