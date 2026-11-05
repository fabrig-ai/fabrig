+++
id = "server-walk"
part = "server-walk"
kind = "docs"
needs = []
files = [
  "temper/docs/help/server.md",
  "temper/crates/temper/src/tests/help.rs",
]
effort = "medium"
minutes = 25
+++

# The server walk's two small additions

## Goal

`temper help server` says two things a person met on a redeploy, each held by a help test. **This departs from the implementer's text:** `temper/docs/help/server.md` is this task's to change.

Acceptance:

- Step 5 says the agent's `environment` takes `[{"name": ..., "value": ...}]` entries, not a map, and points at `temper help box`.
- The walk says that with `application.sign_in.forges` on a new store, the first start's stderr says `temper serve: forge 1: no sign-in through temper: no such forge is kept`, and that the page's `temper-forge-sign-in` meta appears only once step 6's `forge connect` has kept the forge. It says that if a page fetched after that still lacks the meta, restart the engine once: a start makes it from the kept record.
- The page stays within the walk's bound, 12,288 bytes (`temper/crates/temper-docs/src/lib.rs`, `Kind::Walk`), and loses no fact.
- `temper/crates/temper/src/tests/help.rs` holds both, and each new test bites.

**The walk does not say that a restart is required.** The request has it that the meta appears only after `forge connect` and a restart, but `temper/docs/design/protocol/sign-in.md`, 9.7 makes the entry serve "when a forge is made", and these tests hold it with no restart: `a_kept_forgejo_at_its_redirect_gets_its_meta_and_the_page_a_new_etag` (`temper/crates/temper-service/src/tests/forge_sign_in.rs`) and `an_owned_forgejo_signs_its_member_in_through_temper` (`temper/crates/temper/tests/process.rs`). `a_restart_makes_the_meta_from_the_restored_record` holds the restart. The sentence above is true of all three. The cut's report carries the difference to the user; it is not this task's to settle.

## Inputs

- `temper/docs/plan/start-refuses/notes/common.md`.
- `temper/docs/plan/start-refuses/inputs/request.md`, "Two small additions".
- `temper/docs/design/protocol/sign-in.md`, 9.7.
- `temper/docs/help/box.md`, "Coding runs": the entries' form.

## Owns and may change

Owns the files in `files`.

May also change `temper/crates/temper/src/host.rs`, for one line alone: `fn note_line` becomes `pub(crate)`, if the test reads the line from it. Nothing else of `host.rs`.

Options, pre-approved:

- The page is 12,144 bytes, so the additions pass the bound. Condense the page to fit, losing no fact and changing no command, `Expect:` line or quoted output. If it can't be done that way, report it: raising the bound is the user's, and a test that reads a sentence you condensed gets that sentence alone changed, with the report naming it.
- Step 3's `Expect:` already gives the stderr line. Add "on a new store" there, and put the meta's sentence in step 6's `Expect:`, where the connect is. If the two read better together in step 3, do that and say so.
- If `smith_agent_shell::config::from_value` accepts a map for `environment`, the walk's "not a map" is false: stop and ask.

## Steps

1. Read `box_help_configures_coding_run_tools` in `help.rs`: it cuts a page's section, joins its whitespace and parses the page's JSON through `smith_agent_shell::config::from_value`, over `coding_run_agent_document()` with inert endpoints.
2. Edit `server.md`. In step 5, on the worker-alone bullet or beside it:

   ```text
   `agent.json`'s `environment` is a list of entries,
   `[{"name": "PATH", "value": "/usr/bin:/bin"}]`, not a map (temper help box).
   ```

3. Add the sign-in facts, as the acceptance gives them.
4. Add the tests beside the existing ones, sharing the inert-endpoints lines with `box_help_configures_coding_run_tools` if that is the smaller diff:

   ```rust
   #[test]
   fn server_walk_gives_the_agents_environment_as_entries() {
       // Step 5's section alone: its entries parse through the agent's configuration
       // and reach `command_entries`; the same names written as a map are refused.
   }

   #[test]
   fn server_walk_says_when_a_forges_sign_in_meta_appears() {
       // The page holds `note_line(Note::ForgeSignIn { number: 1, client, why: NotKept })`,
       // minus its newline, so the walk's line is the line temper prints; it names
       // `temper-forge-sign-in`, `forge connect` and the restart, in steps 3 and 6.
   }
   ```

5. Break each test once by hand (the entries written as a map in the page; the printed line reworded), see it fail for that reason, and restore.
6. Run, each through `heavy`:
   - `heavy cargo nextest run -p temper --lib -E 'test(help)'`
   - `heavy cargo nextest run -p temper-docs --lib`
   - `heavy cargo nextest run -p temper-client-tools --lib -E 'test(help)'`
7. `cargo fmt`, `heavy cargo clippy -p temper --all-targets -- -D warnings`. Commit and push.

## Report

The page's bytes before and after, what was condensed and that no fact went, each test's break and the failure seen.
