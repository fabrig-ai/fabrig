+++
id = "in-turn"
part = "in-turn"
kind = "interface"
needs = []
files = [
  "skein/testing/skein-fake-person-browser/src/harness/turns.rs",
  "skein/testing/skein-fake-person-browser/src/harness.rs",
  "skein/testing/skein-fake-person-browser/src/lib.rs",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 25
+++

# One person plays a run's stories in turn

## Goal

Stories that share a server and can share its state become one test, one Chromium. Give the harness that form, `Harness::run_stories`, and a failure that names the story. `rig-cases` and `example-merges` build on it.

Acceptance:

- `harness::Story`, `Harness::run_stories` and `harness::name_failure` exist as `skein/docs/plan/skein-browser-starts/notes/contracts.md` has them.
- A step test holds `name_failure`: a failure in the first story, at the first step of a later one and at a later story's later step each names that story and its step within it; `person`, `step` and the rest of the report stay; `Outcome::Done` passes through; a step past every start names the last story.
- A step test holds that `run_stories`' script is the stories' steps in order, with the starts `name_failure` reads.
- Each new test bites (your run's text).
- `skein/docs/design/ui-web.status.md`'s line `- **Not yet:** 14.2: \`Harness::run_stories\` is not built.` is gone.

## Inputs

- `skein/docs/design/ui-web.md`, 14.2, *Stories in turn*.
- `skein/docs/plan/skein-browser-starts/notes/common.md`, and `notes/contracts.md`, *Stories in turn*.
- `skein/testing/skein-fake-person-browser/src/harness/failure_tests.rs`: how a step's failure gets its number.

## Owns and may change

Owns the files in the header, and only that line of the status file. `Harness::run`, `run_lending`, `Setup` and `Script` don't change, and nothing outside this crate has to.

Options, each decided:

- **The steps' numbering:** a `Script`'s `at` counts each step once, so a story's start is the sum of the steps before it. Compute the starts from the stories' lengths, before they move into the script.
- **A failure the run gives at step 0** (a child's exit, `fail(0, 0, ..)`) names the first story, and its report says what happened. Leave it.
- **`Script::expected`** collects the `Trouble` words of all of a script's steps, so a person's stories expect each other's trouble. Leave it, and say so on `run_stories`' doc comment: a story that awaits trouble that another must not meet takes a person of its own.

## Steps

1. Add `harness/turns.rs`, with `mod turns; pub use turns::{Story, name_failure};` in `harness.rs`, and `Story` in `lib.rs`'s re-exports. Sketch:

   ```rust
   //! A person's stories in turn, named in a failure (skein/docs/design/ui-web.md, 14.2).
   use super::{Harness, Outcome, Script, Step};

   fn script(stories: Vec<Story>) -> (Script, Vec<(&'static str, u32)>) { /* steps in order, and each start */ }

   impl Harness {
       pub fn run_stories(&mut self, stories: Vec<Story>) -> Outcome {
           let (mut person, starts) = script(stories);
           name_failure(self.run(&mut [&mut person]), &starts)
       }
   }
   ```

   The report opens `story NAME, its step K: `, K counted from that story's first step, zero-based as `Outcome`'s step is.
2. Test in the same file, under `#[cfg(test)]`: no process and no browser. `Step::StopServer` is a step with no fields beyond its name, for building scripts.
3. Remove the status line.
4. Your run's checks: `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-fake-person-browser --all-targets -- -D warnings`, `heavy cargo nextest run -p skein-fake-person-browser`, and `heavy cargo check -p skein-ui-web-tests -p skein-browser-world --tests`. Commit and push.

## Report

Your run's text. Name the failure you saw for each bite.
