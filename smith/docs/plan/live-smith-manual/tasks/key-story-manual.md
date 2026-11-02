+++
id = "key-story-manual"
part = "key-story-manual"
kind = "feature"
needs = []
files = ["smith/tests/end-to-end/live.rs", "smith/docs/design/testing.md"]
effort = "medium"
minutes = 25
+++
# The API-key live story is the owner's

## Goal

`smith::live an_anthropic_key_delivers_a_commit_in_place` needs an Anthropic API key, which none
of the logins the after-landing run borrows holds, so it is red on every run. Mark it ignored,
with a reason saying it needs an API key and is the owner's. Write in smith's testing design
(2.3) what runs after landing and how the owner runs the key's story. No other live test
changes.

Acceptance:

- `an_anthropic_key_delivers_a_commit_in_place` in `smith/tests/end-to-end/live.rs` carries
  exactly `#[ignore = "live: needs an Anthropic API key; run by the owner (smith/docs/design/testing.md, 2.3)"]`;
  no other line of the file changes.
- `heavy cargo nextest list --profile live-smith -p smith -p smith-agent-process-world` lists
  one test fewer than the same command at your base: every test it listed before except
  `an_anthropic_key_delivers_a_commit_in_place`. Your report gives both counts.
- The same list with `--run-ignored only -E 'test(=an_anthropic_key_delivers_a_commit_in_place)'`
  (the owner's command, below) lists exactly that one test.
- `smith/docs/design/testing.md`, 2.3 holds the two edits under Steps, and nothing else of the
  file changes.
- `cargo fmt` leaves nothing; `heavy cargo clippy -p smith --all-targets -- -D warnings` is green.

## Inputs

- `smith/docs/plan/live-smith-manual/notes/common.md`: the standing rules, among them that no
  live test runs.
- `smith/docs/plan/live-smith-manual/inputs/request.md`: the user's decision.
- `smith/docs/design/testing.md`, 2.3, from "**Live,** the real backends replace the fakes" to
  the end of its "**Stories,**" list.
- The precedent, read only: temper's owner-run live stories, `temper/crates/temper/tests/live.rs`
  (its `#[ignore = "live: owner-run only, ..."]` attributes).

## Owns and may change

- **Owns:** `smith/tests/end-to-end/live.rs` (the one attribute), `smith/docs/design/testing.md`
  (the two edits below).
- **Departures from the implementer text, stated:**
  - This task edits `smith/docs/design/testing.md`: the user asked for the design text with the
    change. Place the text given below; you may rewrap it to the file's width and nothing more.
  - The checks are the two `nextest list` runs and clippy, not a focused run of `smith`: the
    only code change is an attribute in the `live` binary, which no gate profile selects, and
    running it is forbidden (notes/common.md).
- **Leave as they are** (surveyed; each still serves the owner's run): the `anthropic-key` arm
  and the `KEY` message in `smith/tests/end-to-end/support/live.rs`, and
  `key_live_arguments_select_only_the_key` in `smith/tests/end-to-end/live_configuration.rs`.
  `.config/nextest.toml` (the `live-smith` filter) is not changed: the ignore is what leaves the
  story out. No other file names the story.
- **Pre-approved options:**
  - If the base's `live-smith` list already lacks the story, or lists other tests than the
    `smith` and `smith-agent-process-world` `live` binaries, change nothing more: report the
    lists as they are.
  - If clippy fails on code this task did not touch, report it under `## Left` with the lint and
    file, and do not fix it.
  - If `--run-ignored only` is refused by this nextest, use the spelling its `--help` gives, in
    the check and in the design's command alike, and say so.

## Steps

1. At your base, before any edit: `heavy cargo nextest list --profile live-smith -p smith -p
   smith-agent-process-world`; keep the count and the names.
2. `smith/tests/end-to-end/live.rs`, above `fn an_anthropic_key_delivers_a_commit_in_place`,
   under its `#[test]`:

   ```rust
   #[test]
   #[ignore = "live: needs an Anthropic API key; run by the owner (smith/docs/design/testing.md, 2.3)"]
   fn an_anthropic_key_delivers_a_commit_in_place() {
       commit("anthropic-key");
   }
   ```

3. `smith/docs/design/testing.md`, 2.3, inside the "**Live,**" list:
   - A new bullet right after the "**On smith's own accounts,**" bullet (that is, after its
     closing code block, the one ending `cargo nextest run --profile live-smith`) and before
     "**A missing setup fails by name,**", at the same indentation as its neighbours:

     ````markdown
       - **After landing, and the owner's.** After every landing, the
         pipeline runs `live-smith` on the logins it lends from its shelf:
         smith's ChatGPT record and Claude Code's login. No shelf holds an
         API key, so the story on a key is the owner's: it is ignored
         (`#[ignore]`, whose reason says so), the profile's run leaves it
         out, and the owner runs it alone, with `ANTHROPIC_API_KEY` set in
         the environment:

         ```sh
         cargo nextest run --profile live-smith --run-ignored only \
           -E 'test(=an_anthropic_key_delivers_a_commit_in_place)'
         ```
     ````

   - In the "**Stories,**" list, the key's line becomes:

     ```markdown
         - the same outcome on a key, the owner's story (above):
           `--account anthropic-key`, smith's own loop on Anthropic's model;
     ```

4. The two lists of the acceptance, after the edits; compare them with step 1's.
5. `cargo fmt`; `heavy cargo clippy -p smith --all-targets -- -D warnings`.
6. Commit (one commit is enough: the attribute and the design say the same thing) and push.

## Report

The first line as notes/common.md says. Then: the two counts of the `live-smith` list (before,
after) and the one test the owner's list selects; the diff's stat; clippy's result; the branch
and commit pushed; implementation minutes and check minutes apart.
