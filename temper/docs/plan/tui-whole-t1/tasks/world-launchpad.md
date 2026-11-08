+++
id = "world-launchpad"
part = "world"
kind = "feature"
needs = ["shell-launchpad", "tui-walk"]
files = [
  "temper/tests/client/tests/suite/tui.rs",
  "temper/tests/client/src/tui.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 90
+++

# The joined story: a need the engine asks, walked from the launchpad and answered; a change asked with `/change`

## Goal

Prove the launchpad as drawn against the real engine, in the client's
joint world (`temper-client-world`): a question the engine asks the person
shows under Needs you, `n` walks it and the recommended choice answers it
through the engine; `/change WORDS` starts a chat of the project's chat
template, not Ask temper's, and the TUI opens it
(`temper/docs/design/client/README.md`, 6.14; section 10).

Acceptance:

1. **`a_need_the_engine_asks_is_walked_from_the_launchpad_and_answered`**
   (`tests/suite/tui.rs`, beside `a_tuis_launchpad_asks_temper_and_draws_the_chat`):
   a TUI signed in on Hello's launchpad; the engine asks the person a
   question with two choices, the second recommended (the world's
   `fixtures::question`, its words in the form the view's question parser
   reads); the frame shows "? Hello" and the question's words under Needs
   you and "1 needs you · n to walk"; `n` shows the walk's prompt with the
   recommended choice first and active; Enter answers it: the engine
   records the person's answer with that choice's words (read through the
   engine, not the frame); the row leaves Needs you and the TUI notes
   "Nothing else needs you.".
2. **`a_change_asked_from_the_launchpad_starts_a_chat`:** on Hello's
   launchpad, `/change Add a CSV export` and Enter: the engine holds a new
   task of the project's chat template (its charter the chat's, not Ask
   temper's), whose request is "Add a CSV export", and the TUI's frame is
   that chat's.
3. Helpers the stories share go in `src/tui.rs` (a key for `n`, reading
   the frame's rows); `src/fixtures.rs` only where `fixtures::question`
   lacks what the story needs, one function, named in the report.
4. **Status:** delete the **Not yet** bullet naming `world-launchpad` in
   `temper/docs/design/client/README.status.md`. No other design line.
5. **Runs,** through `heavy`, counts in the report: the two stories by name
   (`heavy cargo nextest run -p temper-client-world --test suite -E
   'test(/launchpad/)'`), then `cargo nextest run -p temper-client-world
   --test suite -E 'test(/tui::/)'` once. Their wall and CPU stay within
   the tier's budget; a story over it is red (`notes/common.md`).

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`.
- `temper/docs/design/client/README.md`, 6.14 (*Needs you*, *Walking what
  needs you*, *The TUI's own commands*, `/change`), section 10 (the TUI's
  joined stories).
- The code: `temper/tests/client/src/tui.rs` (`Tui::start`, `press`),
  `src/fixtures.rs` (`tui_asking_world`, `question`), `tests/suite/tui.rs`
  (`signed_in`, `a_tuis_launchpad_asks_temper_and_draws_the_chat`),
  `tests/suite/stories.rs`
  (`a_question_needing_the_person_is_shown_first_in_the_inbox`, how an
  answer is read back).

## Owns and may change

Owns the files listed. `temper/tests/client/src/fixtures.rs` is temper/ui's
too (`ui-p2-home`'s `joint-across`, `ui-p1-rail`'s `readers`): touch it
only as acceptance 3 allows. Not the TUI's crates: a fault found there is
reported under `## Left`, with the frame, not repaired here.

Options, each decided:

- **The question's choices:** if `fixtures::question` takes words alone,
  write the choices in the words as the view's parser reads them
  (`temper-client-domain`'s `question.rs`, `parse`); if it can't mark a
  recommended one, take the first as primary and say so.
- **The chat template's charter:** compare the new task's executor with
  the template the world's project sets for chats (`jig_client::shapes::CHAT`),
  as the world reads templates elsewhere.

## Steps

1. The two stories, failing on the trunk before `shell-launchpad` merged
   where that can be seen, else by breaking the rule by hand (`n` not
   intercepted; `/change` sent as words to Ask temper); then green.
2. The status edit; the runs of acceptance 5; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-world
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
3. Commit (why: the launchpad's walk and `/change` proven against the real
   engine); push.

## Report

As the implementer's text says, with each story's rule broken by hand and
its failure, the options taken, and each story's wall time.
