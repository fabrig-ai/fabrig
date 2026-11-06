+++
id = "setup-words"
part = "setup-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tools/src/forge.rs",
  "temper/crates/temper-client-tools/src/forges.rs",
  "temper/crates/temper-client-tools/src/tests/forges.rs",
  "temper/docs/design/client/tools.status.md",
]
effort = "medium"
minutes = 25
+++

# `forge setup` doesn't end on a line nobody can act on

## Goal

`forge setup` on a box's own Forgejo ends `temper's forge user may not read the branch protection: the forge shows it to repository administrators only.` This cut read why, and the line is true: the bot is a writer (`skein-forge-setup`'s `forgejo/access.rs`, `Grant::Write`), and Forgejo shows protection to admins alone, so adoption records `Protection::Unknown`. But a person can't act on it, on a repository temper has just made. It goes from `forge setup`'s ending, and stays where it can mean something.

Acceptance:

- `forge setup`'s printed adoption, and `playground`'s if it goes through the same path, has no such line, with `Protection::Unknown`.
- Its `--json` keeps `"protection":{"unknown":{}}`.
- `repository adopt` keeps the line: golden `adopt_adopted` doesn't change.
- The bot's grant doesn't change, and nothing outside `temper-client-tools` does. `Word::Protection` stays, for `repository adopt` and for `temper-client-view`.

## Inputs

- `temper/docs/design/client/tools.md`, 5.5, the `forge setup` row, and 14, `Protection on a forge temper owns`.
- `temper/docs/design/domain/forge.md`, 4, the bullet beginning `What temper may do is read at adoption`.
- `temper/docs/plan/ctl-words/notes/common.md`.

## Owns and may change

Owns the files in the header. Readers: `print` has one caller outside `forge.rs`, in `forges.rs`'s `adopted`. Nothing in a process-tier test or a help page reads this line (`rg 'may not read the branch'`); say so if you find otherwise. `temper-client-shell-terminal`'s golden `adopt_adopted.txt` and `temper-client-view`'s tests are another surface's, and don't change.

Departs from the implementer's text: `temper/docs/design/client/tools.status.md` is open for the one line in step 3.

Options, each pre-approved:

- Default shape, in `forge.rs`:

  ```rust
  /// Whether an adoption's text says its protection went unread (client/tools.md, 5.5).
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub(crate) enum Unread { Said, Unsaid }
  ```

  `print` takes it after `kept`, and hands it to where `Word::Protection` is phrased, about line 385. `repository adopt` passes `Said`, and `forges.rs`'s `adopted` passes `Unsaid`. If `print`'s text has a helper that makes this awkward, shape it otherwise, one value crossing and no second copy of the words, and say so.

## Steps

1. **Reproduce.** In `tests/forges.rs`, beside the story near line 578 that asserts `project: 1 · shop\nAdopted `, `setup_ends_on_its_adoption_without_the_protection_line`, its outcome `fixtures::adopted_with_unknown_protection()`: the stdout lacks `may not read the branch protection`, in text. A second call with `--json` holds `"protection":{"unknown":{}}`. See the text story fail.
2. **The fix** as above.
3. **Status.** In `temper/docs/design/client/tools.status.md`, delete the bullet beginning `**Not yet:** 5.5: `forge setup` ends with the line`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tools --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tools`; `heavy cargo nextest run -p temper-client-shell-cli`. Commit and push.

## Report

As the implementer's text has it. Say whether `playground` goes through this path.
