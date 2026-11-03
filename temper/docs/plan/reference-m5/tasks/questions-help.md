+++
id = "questions-help"
part = "reference"
kind = "docs"
needs = ["relay-foreman", "questions-panel"]
files = ["temper/docs/help/task.md", "temper/docs/help/change.md", "temper/crates/temper-client-tools/src/tests/golden/**"]
effort = "medium"
minutes = 30
+++

# questions-help: the help pages say how a change's questions go

## Goal

temper's help says what the code now does (`domain/forge.md`, 8.11): a change asks no one
unless its maker names its questions, and then relays its agents' questions through a
foreman or straight up. (A stated departure from the implementer's text: this task edits
`docs/help/`.)

Acceptance:

- `temper/docs/help/task.md`: the sentence "A change asks no one, and neither do the agents
  it delegates: ..." becomes: "A change asks no one unless its maker names its questions:
  then its producer, repairs and resolutions ask it, and it has each question settled by a
  foreman of its own, which reads the change's words and the design and answers, tells the
  producer to finish, or passes the question up, or passes it straight to its requester; it
  answers the asker, holding an answer until its dependencies have landed, and past its
  bound of questions it holds. A question or words a change does not take are answered
  that it takes none, naming its requester; a party's words to it are taken unanswered."
- `temper/docs/help/change.md`: where it lists `change`'s members (`landing`, `after`, ...),
  `questions` (`foreman`, `up`, or `none`, the default) joins them, with "how its
  producer's questions go"; and where it lists what `change show` prints, its questions
  line (C8), as `docs/help/reference/ctl-change.md` already says.
- The help goldens that render these pages, if any, follow them.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.1 and 8.11; `temper/docs/design/domain/agent.md`,
  4.3.1 (`questions`).
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C6 and C8.
- `temper/AGENTS.md`'s rule for `docs/help/` (compiled into its help tests).

## Owns and may change

Owns the files in the header. If `temper-docs`' rules test refuses a word or a link the
sentences above use, keep their meaning in the page's own voice and say what changed.

## Steps

1. The two pages; any golden.
2. Narrow: `heavy cargo nextest run -p temper-docs` (check the count), then `heavy cargo
   nextest run -p temper-client-tools --lib -E 'test(/help/)'`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`. Commit ("Say how a change's
   questions go") and push.

## Report

As the implementer's text says.
