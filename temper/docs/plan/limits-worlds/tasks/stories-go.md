+++
id = "stories-go"
part = "belonging"
kind = "migration"
needs = ["lower-stories"]
files = [
  "temper/tests/real-loop/tests/suite/main.rs",
  "temper/tests/real-loop/tests/suite/fuzzy/**",
  "temper/tests/real-loop/src/case.rs",
  "temper/tests/real-loop/src/case/**",
  "temper/tests/real-loop/src/calls.rs",
  "temper/docs/design/domain/testing.md",
  "temper/docs/plan/limits-worlds/figures/stories-go.md",
]
effort = "medium"
minutes = 75
+++

# The stories whose behaviours are all held elsewhere go

## Goal

`plan.md`'s *Belonging* table reads each real-loop fuzzy story for what it checks that no other test holds. Four hold nothing of their own. Remove them, and what only they read, once their holders pass. Then make `temper/docs/design/domain/testing.md`, 5.3, say what is so. **No test moves, and nothing else leaves a story.**

| Goes | |
|---|---|
| `fuzzy::landing::a_chat_lands_its_change_on_the_forge` | its test alone: `a_chat_keeps_its_live_delegate_until_the_change_lands` stays in the file |
| `fuzzy::case_5_6` | the file, and what only it reads |
| `fuzzy::case_6_1` | the file, and what only it reads, if `lower-stories` holds its behaviour |
| `fuzzy::case_6_2` | the file, and what only it reads, if `lower-stories` holds its behaviour |

Acceptance:

- **The holders pass first.** Each test in `plan.md`'s *What holds them now* has run, by name, on your base before anything is removed, and `temper/docs/plan/limits-worlds/figures/stories-go.md` lists each with its result. A holder that's missing or failing keeps its story: name it in your report.
- **The four are gone,** with their `mod` lines in `temper/tests/real-loop/tests/suite/main.rs`, and what only they read: `case::fixture_5_6`, `fixture_6_1` and `fixture_6_2`; `case::scripts`' and `case::acts`' steps, reads and constants for them; `case::oracle`'s `step_5_6`, `step_6_1` and `step_6_2` with their mutation tests; and `Observed` fields and `Read` variants nothing else reads. What a surviving story reads stays, whatever its name says.
- **`calls.rs`:** `blocker_kept_wait_rejects_a_running_snapshot_with_saved_activity` reads `acts::step_6_2`. If what it pins is a wait in `calls.rs` that no surviving story uses, the test and that wait go together. If a surviving story uses it, rewrite the test on a hand-made step with the same response.
- **`temper/docs/design/domain/testing.md`, 5.3:** rows 5.6, 6.1 and 6.2 read as `4.2` and `6.3` do, `none`, then what holds each, by test name, and what it leaves out (keep what's still true of the *leaves out* column). Each row's number stays; nothing renumbers (`DESIGN.md`, 4).
- **Rows that lean on them** say the same thing themselves: 6.2's "on 6.1's deployment", 6.5's and 6.8's "as 6.1's do", and every other mention your search finds, in that table and in the act's introductions.
- **The stale claims this reading found are corrected in the same rows,** against the code:
  - the process tier's stories: `temper/crates/temper/tests/` holds only 4.6's, and it's `#[ignore]`d. The bullet "The process tier reruns a step only where processes are the point" and rows 4.3, 4.4 and 4.5 name stories that are gone. Say what's there, with its ignore's reason. Keep the user's rule itself.
  - row 5.1: A's question arrives after the release, not while the chat is held.
  - row 5.2: the story's name, and that Stop holds B's delegates, whose work finishes after the release.
  - row 5.5: the refusal's words, as the oracle has them.
  - row 6.6: the story's name, and that repin's producer starts early, before e1 lands.
- `heavy cargo nextest run -p temper-real-loop-world` and the same `--profile fuzzy` pass, with the count four lower than your base's fuzzy count for `temper-real-loop-world`. `heavy cargo check -p temper --tests` builds, since the process tier reads this library (`temper/crates/temper/tests/support/case.rs`). The process tier isn't run.
- `python3 -I -B temper/scripts/old_forms.py --write` has lowered what you removed, if it counted any, and `python3 -I -B tools/citations/citations.py --base origin/main` shows nothing new.

## Inputs

- `temper/docs/plan/limits-worlds/plan.md`, *Belonging* and *What holds them now*.
- `temper/docs/plan/limits-worlds/notes/common.md`.
- `temper/docs/design/domain/testing.md`, 2.4, 5.3 and 6.1; `DESIGN.md`, 4.
- `temper/docs/plan/limits-worlds/inputs/request.md`, *For every task of this plan*, "Nothing is lost".
- `lower-stories`' two tests, in `temper/tests/engine/domain/tests/suite/root_routes/added_dependencies.rs`.

## Owns and may change

Owns the files in the header. A departure from the implementer's text: the design edits above are yours. No other design text changes: `temper/docs/design/domain/testing.md`, 2.4, still cites `suite::fuzzy::landing`, whose module stays.

Options, each decided:

- **`lower-stories` didn't hold a behaviour:** check for each that its test exists in `added_dependencies.rs`. If one is missing, that story stays, its row stands, and your report says so.
- **The process tier's `support/case.rs` reads an item you'd remove:** it stays. A process-tier reader counts as a reader.
- **A surviving story's oracle step calls a removed step's helper:** the helper stays, where it is. Don't move code you needn't.
- **A row's *leaves out* column cites a removed story for what it holds:** rewrite that clause to its holder.
- **`model-catalogue-2/process-fixtures`** holds `temper/tests/real-loop/tests/fixtures/**`, and this task needs nothing there. `b0/` and `b1/` stay: `green_light` and `repair` read them.

## Steps

1. Run the holders by name on your base, and write their results. `heavy cargo nextest run -p <package> --profile <default|fuzzy> -E 'test(=<name>)'`, one at a time, and check each count.
2. Remove the stories and what only they read. Let the compiler and `rg` find the orphans: a `pub` item in a library doesn't warn, so search for each.
3. Edit 5.3.
4. `cargo fmt`; `heavy cargo clippy -p temper-real-loop-world --all-targets -- -D warnings`; the runs and checks in the acceptance; commit and push.

## Report

The four stories and each holder's result, the lines removed, the fuzzy count before and after, anything that stayed and why, and your implementation and check minutes apart.
