+++
id = "overturn"
part = "overturn"
kind = "feature"
needs = ["home-away"]
files = [
  "temper/crates/temper-client-domain/src/forge/decided.rs",
  "temper/crates/temper-client-domain/src/pages/away.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# Overturn

## Goal

The person takes a decision back while its task lives: `temper/docs/design/client/README.md`, 3.5 (*Away*, *Overturn*), 4.6 (`Overturn`).

Acceptance:

- **"Overturn"** (`.btn.small`, as the mock's), at a row's right where its task is `Live` and it isn't overturned, `Act::Away(away::Act::Overturn { project, number })`, disabled while any overturn is out.
- **`Overturn` answered `overturned`:** the client then sends `c::Ask::Message { project, task, words }`, keyed, from the row's own question and took, in `forge::decided::overturn_words(question, took)`:
  "I overturn your decision. QUESTION You took TOOK. Ask me again with message, as a question, and wait for my answer."
- **Toasts:** message sent, `Toasted::Overturned`, "Overturned: its goal asks you again, under Still needs you."; message refused, `OverturnEnded`, "It has ended; a new goal can undo it.", the row left marked; `overturn_refused`
  or unknown, `OverturnAlready`, "That was overturned already.", no message.
- **One request a row:** a second act while out does nothing, and a page left mid-way still sends its message: the state is `Pages`', not Home's, if Home's would drop it.
- **`home-away-overturned`** in the gallery, as `browser.md`, 7.2; and `home-away` now shows the button.
- `README.status.md`'s and `browser.status.md`'s lines lose what this closes.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 (*Away*), 4.6.
- The models: `pages/task.rs`' `send` (`c::Ask::Message`); `pages/home.rs`' `skip`; `pages/needs.rs`' `toast`.

## Owns and may change

Owns, as `files` lists them. May also change: `temper-client-view`'s `tests/fixtures.rs`; the terminal's goldens, regenerated.

Options, each pre-approved:

- **Where the state lives:** `pages/away.rs`, beside the switch's pending key, is the default, so that leaving Home can't lose a message.
- **The message's bound:** question and took fit a task's message at the deployment's smallest; if a limit's test says otherwise, `overturn_words` cuts the question, and you say so.
- **A task read as `Live` that has ended:** the message's refusal is the answer; nothing more.

## Steps

1. `overturn` and `overturn_words` in `forge/decided.rs`; the acts and their answers. `src/tests/pages/home.rs`: `overturn_sends_the_request_then_the_message`, `the_message_says_question_and_took`, `a_refused_message_leaves_the_row_and_toasts`,
   `a_refused_overturn_sends_no_message`, `a_second_act_waits`, `leaving_home_still_sends_it`.
2. The button and the toasts. `tests/home.rs`: `a_live_row_offers_overturn`, `an_overturned_row_offers_none`. `tests/toast.rs`: the three.
3. The gallery.
4. The check of the implementer's text, `-p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`. Push.

## Report

As `notes/common.md`, Your report.
