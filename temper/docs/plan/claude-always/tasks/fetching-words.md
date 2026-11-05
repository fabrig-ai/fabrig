+++
id = "fetching-words"
part = "words"
kind = "feature"
needs = ["claude-fetch"]
files = ["temper/crates/temper-client-view/src/accounts/words.rs", "temper/crates/temper-client-view/src/tests/connect_model.rs", "temper/tests/client/tests/suite/connect_model.rs", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/client/README.status.md"]
effort = "medium"
minutes = 20
+++

## Goal

The Claude way says it fetches Claude Code first, and the plan's three design status entries go, its code having landed.

Acceptance: `Word::ProgramNote` reads as `temper/docs/design/client/README.md`, 5.5 has it, in Details and in Ready; the view's and the client world's tests pass with the new words; the three **Not yet** entries below are gone and citations check clean.

## Inputs

- `temper/docs/plan/claude-always/notes/common.md`
- `temper/docs/design/client/README.md`, 5.5
- `DESIGN.md`, 2

## Owns and may change

**Departs from your text:** you edit three status files (`Not yours` otherwise holds), because this is the plan's last task and each entry's code has landed. Only the sentences below.

**Owns:** `temper/crates/temper-client-view/src/accounts/words.rs`, the one word.

**Readers you may change:** `temper/crates/temper-client-view/src/tests/connect_model.rs` (lines 320, 350) and `temper/tests/client/tests/suite/connect_model.rs` (line 87): the words alone. Any other test that pins the old sentence, the same way.

**Pre-approved option:** `look-round-4/playground-line` changes `tests/connect_model.rs` beside this. If your branch meets it, take theirs and redo the one line.

## Steps

1. `Word::ProgramNote`: `If this machine has no Claude Code, this fetches it first. Claude Code asks you to sign in. temper never reads its login.` Same word, so Details and Ready both take it. No new word, no new layout.
2. Update the tests' words. If the note now wraps a terminal golden, take the new golden; say so in the report.
3. Remove these sentences, each from its `**Not yet:**` list, leaving the rest as it is:
   - `temper/docs/design/protocol/setup.status.md`: `6.6: Claude Code's build: no pin, no launcher and no fetch of it, so a `"fetched"` program has nothing to run.` and `8.2, 8.3: init writes Anthropic's provider, program, endpoint and model only as the fake backends' rewrite adds them; with no `claude` on `PATH`, no program at all.`
   - `temper/docs/design/protocol/configuration.status.md`: `5: a program's `"fetched"` path, which smith refuses as a relative one.` and `12.1: the rewrite adds Anthropic's provider, endpoint and model where it should find init's and swap them.`
   - `temper/docs/design/client/README.status.md`: `5.5: a program's Details and Ready do not yet say that Claude Code is fetched first.`
4. `python3 -I -B tools/citations/citations.py --base origin/main`. `cargo fmt`, `heavy cargo clippy -p temper-client-view --all-targets -- -D warnings`, `heavy cargo nextest run -p temper-client-view -p temper-client-world`, with the client world's package name as its `Cargo.toml` has it. Commit, push.

## Report

As your text says, and any golden that moved.
