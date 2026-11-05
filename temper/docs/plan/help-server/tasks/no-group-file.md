+++
id = "no-group-file"
part = "owned-by-hand"
kind = "feature"
needs = []
files = ["temper/crates/temper/src/forge.rs", "temper/crates/temper/src/commands.rs", "temper/crates/temper/src/tests/commands.rs", "temper/crates/temper/src/tests/box_kinds.rs", "temper/crates/temper/tests/suite/forge.rs", "temper/docs/help/reference/commands.md", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 60
+++

# `temper forge fetch`, `prepare` and `run` need no group's file outside a box

## Goal

`temper help server`'s step 3 fails at its first line: `temper forge fetch ... 1` answers `forge 1: DATA/box/groups/1: absent`. Decided: the commands need no file. A forge with none takes the machine's `owned` part, so the walk and a redeploy need no hand-written file. `temper forge remove` stays strict: it removes a directory by its kind, which it does not guess.

Acceptance:

- With no `box/groups/N`, `fetch`, `prepare` and `run` compose forge N from `application.machine.owned`, in `<data>/forge-N`, and write nothing under `box/`.
- With no file and no `owned` part, each refuses with exactly `temper forge VERB: forge N: PATH: absent, and application.machine names no owned Forgejo`.
- A file there is read as today: `fake` is fake, a bad kind refuses, a read that fails for another reason than absence refuses as today.
- `temper forge remove N` with no file refuses as today, naming `PATH: absent`, and removes nothing.
- The tests below pass; each is seen red first.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1 and 7.4, the passages this cut wrote; `temper/docs/design/protocol/README.md`'s forge commands.
- `temper/docs/plan/help-server/inputs/request.md`, finding 2.
- `temper/docs/plan/help-server/notes/common.md`.

## Owns and may change

Owns: `temper/crates/temper/src/forge.rs`, `entry` and `group_kind`.

May also change, and how:

- `temper/crates/temper/src/tests/box_kinds.rs` and `temper/crates/temper/tests/suite/forge.rs`: the tests below.
- `temper/crates/temper/src/commands.rs`: the `forge fetch`, `prepare` and `run` rows' `notes`, one sentence each. `temper/docs/help/reference/commands.md` is regenerated, never written by hand, by `temper/crates/temper/src/tests/commands.rs`'s test with `SKEIN_CODEGEN_WRITE=1`.
- `temper/crates/temper/src/tests/commands.rs`: `each_row_answers_its_help`'s hand-written `forge run` text, gaining the sentence that row's `notes` gains, and nothing else.
- `temper/docs/design/protocol/setup.status.md` and `README.status.md` (a departure from your text, which leaves `docs/design/` alone): remove the **Not yet** entry for 7.1 and 7.4, and README's for section 4 that names the group's file.

Options, each pre-approved:

- `skein_shell::read_file`'s absence may be matched as its `Unread` variant, or by a `symlink_metadata` first: take the variant if it has one, and keep a link refused as today.

## Steps

1. Confirm: a test of step 2's that runs `prepare` with no file fails with `absent`.

2. In `forge.rs`:

   ```rust
   /// Composes a forge's setup entry: its group's kind, or the machine's `owned` part with none
   /// (temper/docs/design/protocol/setup.md, 7.1 and 7.4).
   pub(crate) fn entry(directories: &super::Directories, bytes: &[u8], number: u16) -> Result<Entry, String>;
   ```

   `remove` keeps calling `group_kind`, which refuses an absent file. `entry` reads the same file, and an absent one alone means `BoxKind::Owned`, with the new refusal when the machine has no `owned`.

3. Tests:

   - `crates/temper/src/tests/box_kinds.rs`, beside the composing test: `a_forge_with_no_group_file_is_the_machines_owned`, `no_group_file_and_no_owned_part_is_refused_naming_both`, and, on `forge::start`, `remove_refuses_a_forge_with_no_group_file` with `forge-17` left in place.
   - `crates/temper/tests/suite/forge.rs`: `prepare` and `run` by the owned commands' fixture, the file removed after `owned::configuration`, on the fake programs that suite uses; and `forge_fetch_command`'s `present` row run once more with no file.
   - Nothing changes `owned::configuration` or `tier_configuration`: the box's own arrangement keeps its tests. `walk-story` takes the process tier's.

4. `cargo fmt`, clippy, `heavy cargo nextest run -p temper --lib -E 'test(~box_kinds) | test(~help)'` and `-p temper --test suite -E 'test(~forge)'`, push.

## Report

As the implementer's text says, and the rule each new test was broken by.

## Answers

2026-11-05. May this task update the hand-written `forge run` help text in `temper/crates/temper/src/tests/commands.rs`? Yes. The sentence this spec requires in that row's `notes` is what the test's literal copies, so its acceptance cannot hold otherwise, and the test keeps its meaning: the row renders as written. Append to the literal's notes line the same sentence, word for word, and change nothing else in the file; the file is now in `files`. Keep the pushed commit `709064f503`; run `tests::commands::each_row_answers_its_help` alone, then the unfiltered `heavy cargo nextest run -p temper --lib -E 'test(~box_kinds) | test(~help)'`, and push.
