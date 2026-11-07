+++
id = "run-words"
part = "words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-smith/src/calls.rs",
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/docs/help/reference/host-tools.md",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 75
+++

# A run's tools say when to call goal, and a refused branch read says what to read

## Goal

In live-temper's owned-forge landing story, the shipped chat on `gpt-6-luna` called `goal` for a plain change ("add a line to README.md and land it"). Its words say to call `change` for that, but `goal`'s declaration says only what a goal may do, never when to start one. Its goal then had `read_branch` refused `Missing` and `read_branches` refused `Empty`, and was told no more than `Forge read failed: missing` and `Forge read failed: empty`. Both refusals are right. What is wrong is the words, which name nothing to read instead. The design now gives `goal`'s declaration its when (`agent.md`, 4.3.2) and the two refusals their next reads (`agent.md`, 4.3). This task builds both in `temper-engine-smith`.

This departs from the implementer's text in two ways: its first commit is a failing test (the common note's *failing test first*), and three status entries are yours to remove (acceptance 5).

Acceptance:

1. `goal`'s declared description is, byte for byte, `agent.md`, 4.3.2's: "Have one goal in your words tracked and reported; it may plan, ask you and call change. Call it only for work your person wants agreed first (set agree) or a survey that changes nothing; with advisor set, it reads and reports and changes nothing. For a change to a repository, call change." Its schema is unchanged.
2. `read_branches`'s `prefix` member is described "A nonempty prefix of the names, as temper/." (`BRANCHES_SCHEMA`), the rest of that schema unchanged.
3. `answer_for_tool` renders `read_branches` with `Err(Empty)` and `read_branch` with `Err(Missing)` as `agent.md`, 4.3, *A refused branch read says what to read instead* has them: the first line, a full stop, a space, then the bullet's words exactly. Each is marked an error and cut to fit the bound as the compare's guidance is. Every other tool's `Missing` and `Empty`, and these two tools' other errors, keep their one line. `read_compare`'s `Forge read failed: missing` row in `read_compare_too_large_names_bounds_and_paged_fallback` stays as it is.
4. The tests of Steps 1 pass and bite. `temper/docs/help/reference/host-tools.md` is regenerated (Steps 3), and `the_page_host_tools_is_what_the_declarations_render` passes.
5. The **Not yet** entries in `temper/docs/design/domain/agent.status.md` that begin "4.3, *a refused branch read says what to read instead*" and "4.3.2: `goal`'s declaration does not say when" are gone. The 4.3.2 entry on the live request `live-change-call` stays, as does the rest of the file.
6. `heavy cargo nextest run -p temper-engine-smith` passes, and so do the direct dependents' focused tests: `-p temper --lib`, `-p temper-service`, `-p temper-engine-domain-world`, `-p temper-client-world`, `-p temper-system-world`, `-p temper-playground`, each with its count named (take the packages' names from their `Cargo.toml` where these differ). Clippy is clean for `temper-engine-smith`.

## Inputs

- `temper/docs/plan/live-change-call/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 4.3, the bullet *A refused branch read says what to read instead*, and the compare bullets just above it for how a tool-specific recovery is cut to fit.
- `temper/docs/design/domain/agent.md`, 4.3.2, its last two paragraphs, from "Otherwise the root refuses a goal's `goal`" to the end of the section.
- `skein/docs/design/forge.md`, section 9, on `Branches`' nonempty prefix: why the `Empty` stays.

## Owns and may change

**Owned:** the `files` above. `host-tools.md` is regenerated, never edited by hand.

**Readers and neighbours, edits allowed:**

- A test in a dependent package of acceptance 6 that asserts on `goal`'s description, on `read_branches`' prefix description, or on the exact text `Forge read failed: missing` / `Forge read failed: empty` for `read_branch` / `read_branches`: that literal only, named in the report. The survey found none outside `temper-engine-smith/src/tests.rs`.

**Options, each with its default:**

- Where the tool-specific words go: `forge_read` in `answers.rs` already takes the tool and adds `read_compare`'s recovery after `TooLarge`. Default: two more arms there, one `fn` per recovery beside `compare_recovery`, the words as `&[u8]` constants.
- The descriptions' bound: if `every_description_is_within_its_bound` (`src/tests/reference.rs`) or a stock charter's brief room (`temper-service`'s `the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds` and the start's 2,048-byte check) refuses the longer `goal` description, stop and ask with the figures. Don't shorten words the design binds.
- The help page's `PAGE_BYTES`: if the regenerated page no longer fits, stop and ask with the figures.

## Steps

1. Red first, in `temper/crates/temper-engine-smith/src/tests.rs`, beside `read_compare_too_large_names_bounds_and_paged_fallback`:

   ```rust
   /// agent.md, 4.3: a refused branch read says what to read instead; every
   /// other refusal keeps its one line.
   #[test]
   fn a_refused_branch_read_says_what_to_read_instead() {
       use skein_forge::api::Error;
       for (tool, error, expected) in [
           (b"read_branches".as_slice(), Error::Empty, BRANCHES_EMPTY),   // the whole text, first line and words
           (b"read_branch", Error::Missing, BRANCH_MISSING),
           (b"read_branch", Error::Empty, "Forge read failed: empty"),
           (b"read_branches", Error::Missing, "Forge read failed: missing"),
           (b"read_file", Error::Missing, "Forge read failed: missing"),
           (b"read_pull", Error::Empty, "Forge read failed: empty"),
       ] {
           let refused = engine::CallAnswer::ForgeRead(Box::new(Err(error)));
           let rendered = crate::answer_for_tool(&refused, tool, 2048);
           assert!(rendered.error());
           assert_eq!(rendered.text(), expected.as_bytes());
           for cap in [0, 32, 64] {
               // cut to fit, a prefix of the whole, still an error, as the compare's rows hold
           }
       }
   }

   /// agent.md, 4.3 and 4.3.2: a declaration says what a run calls instead.
   #[test]
   fn declarations_say_when_to_call_goal_and_what_prefix_to_give() {
       // goal's description == agent.md 4.3.2's words; delegate's still ends
       // "For a change to a repository, call change."; read_branches' prefix
       // member's description == "A nonempty prefix of the names, as temper/."
   }
   ```

   Read the declarations as `every_forge_read_declares_its_flat_members_as_json` does. `BRANCHES_EMPTY` and `BRANCH_MISSING` are test constants holding `agent.md`'s words. Commit alone ("temper-engine-smith tests: goal says when, refused branch reads say what to read"); both fail on the base.
2. The words: `goal`'s description and `BRANCHES_SCHEMA`'s prefix in `calls.rs`; the two recoveries in `answers.rs`'s `forge_read`. Bite: drop the `read_branch` arm once and see its row fail; drop "For a change to a repository, call change." once and see the declaration test fail.
3. Regenerate the page: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --lib -E 'test(=tests::commands::the_page_host_tools_is_what_the_declarations_render)'`, then run the same test without the variable. The diff of `host-tools.md` should be the `goal` and `read_branches` lines alone; say so in the report.
4. The status entries (acceptance 5).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-smith --all-targets -- -D warnings`; the runs of acceptance 6, naming the new tests' time. Commit ("temper: goal's declaration says when to call it, and a refused branch read says what to read instead"), push.

## Report

As the implementer's text says, and: the byte growth of the `goal` and `read_branches` declarations and the smallest stock brief room after it (as the start's check computes it), and the host-tools page's size against `PAGE_BYTES`.
