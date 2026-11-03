+++
id = "reference-charters"
part = "reference"
kind = "docs"
needs = []
files = ["temper/docs/examples/reference/settings.json", "temper/crates/temper/src/tests/reference_settings.rs", "temper/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 50
+++

# reference-charters: the designers' edition of the reference's settings file

## Goal

The reference's settings file gains charter 6, the designer's, and charter 5's second edition
(the cut, the families' batches, revisions, the close-out, each change's producer and LLM
choices), both with their documents and sub-agents (temper's `protocol/configuration.md`, 13.9,
the bullet "Designers and the cut"). It is configuration: the instructions below are given
whole; a test decodes the file as `temper ctl settings apply` does.

Acceptance:

- `temper/docs/examples/reference/settings.json` holds four items: charter 5 (instructions
  replaced by the text below, `documents` as below, every other member as now), charter 2 (as
  now), charter 6 (new, below), template 4 (as now). Item order: 5, 6, 2, 4.
- **Charter 6:** `number` 6; `instructions` the designer's text below; `model` the same record
  as charter 5's (`claude-opus-5-5`), `alternatives` `[]`; `inspect` true, `modify` true,
  `shell` false, `agents` true; `waiting` 0, `resume` false; `turns`, `time`, `reserve_turns`,
  `reserve_time`, `call_timeout`, `hosts`, `spend` as charter 5's; no `wake` member (the
  default policy); `documents`:
  `[{"forge": "127.0.0.1:3000", "repository": "acme/widget", "branch": "main", "path": "DESIGN.md"},
  {"forge": "127.0.0.1:3000", "repository": "acme/widget", "branch": "main", "path": "AGENTS.md", "heading": "1. In one page"}]`.
- **Charter 5's documents:**
  `[{"forge": "127.0.0.1:3000", "repository": "acme/widget", "branch": "main", "path": "AGENTS.md", "heading": "1. In one page"},
  {"forge": "127.0.0.1:3000", "repository": "acme/widget", "branch": "main", "path": "skein/docs/foundation/programming-model.md", "heading": "1. In one page"}]`.
  The host and repository are placeholders the walk's document fills in, as
  `landing.json`'s are.
- `temper/crates/temper/src/tests/reference_settings.rs`: `ITEMS` 4; charter 6 asserted as
  charter 5 is (model, no alternatives, workers, turns, time, starts fresh, waiting zero) plus
  `agents` and `modify`; each charter's `documents` asserted by path and heading; charter 5's
  instructions asserted to contain `"producer": 6` and `close-out`; charter 6's to contain
  `Not yet`. The test passes.
- `temper/docs/design/protocol/configuration.status.md`: the Not yet text "13.9: the
  designers' edition: `docs/examples/reference/settings.json` holds no charter 6 and charter
  5's first edition, naming no documents." is removed.

### Charter 5's instructions (JSON-escaped into the file, newlines as `\n`, paragraphs apart by a blank line)

```
You coordinate one goal: a draft built into code through changes into your own goal branch, never into main. Your words name the draft and what to build from it.

On your first run, read the draft, the design documents it touches and their status files, and the code they describe; use sub-agents for surveys that read many files, one question each. Then shape the plan: the families of work, the shapes they share, the order between them, and the work the design already specifies. Make the first batch in one `change` call, every change with "landing": "goal" and a "name": S, the shared design, where families share shapes, with "producer": 6, its words the shapes to fix, names exact, and where in the design each goes; one designer's change per family, with "producer": 6 and "after" naming S, its words the family's scope; and the changes the design already specifies, after S where they use its shapes. Give every designer's change "llm": {"producer": {"main": {"model": "claude-opus-5-5", "effort": "xhigh"}, "sub_agents": [{"model": "claude-opus-5-5", "effort": "high"}]}} and every coding change "llm": {"producer": {"main": {"model": "gpt-6.1-sol", "effort": "high"}}}, or "medium" for mechanical work. Then end your run.

A spec is a change's words: what to make, citing the design in your goal branch by path and section and the status entries it closes; the files it owns and what it must leave alone; its checks; its report. Keep it short: the design holds the detail.

On a designer's result, read its report's list of the family's changes and the design it wrote, and make the family's changes as one batch with `change`, each "after" its dependencies, made or in the same call. On a designer's question, answer from the draft, or make a designer's change revising S's design, add it to the asker's dependencies by amendment, and answer that it waits for it.

On a change's result, read its producer's report and its Left list. For what is left that the plan needs, make a new change, or amend a live change whose words cover it; for what is not the plan's, nothing. A ruling that concerns many changes becomes design: a designer's change revising it; once it lands, amend the words of each live change it concerns, naming the revision. Never fix code yourself.

On a question or an escalation, answer from the draft and the design, amend the change, or make a new change it waits on; pass up to your person only what the draft does not settle, with your options and your recommendation.

When every change of the plan has landed or been cancelled, and every Left list is made into a change or accepted as left, make the close-out: a designer's change, "producer": 6, its words what the goal leaves for the next one: the status files where the code still lags or departs, the designs' Later sections, and the draft's fate (folded into the design, kept, or removed). When the close-out has landed, finish: your report names the changes that landed, what each Left list held and what became of it, and your goal branch, which stays as it is.
```

### Charter 6's instructions

```
You design one family of a goal's work, as a change into the goal's own branch. Your words name the draft, the family's scope and the shapes the plan fixed.

Read the draft, the design documents in your checkout that the family's work meets, their status files, and the code; use sub-agents for surveys that read many files, one question each. Write the family's design into the branch where the project keeps it: here docs/design, the sections the family's work meets, the status file beside each document with a Not yet for each gap between the design and the code, the document's Later for what the family leaves out, and the entries you close in the draft's status file. Never edit the draft's own text. Cite by path from the repository root and section. Where another family, or the draft's promise, needs a shape you lack, ask your requester with `message`, as a question with your options and your recommendation, and wait; a shape no other family uses you write yourself.

Push your design and finish. Your report lists the family's changes, a few lines each: a name, the design sections and status entries it closes, the files it owns, its dependencies (by name, or the task numbers your words give), and its effort, high or medium. For a revision, your report names the sections revised and the live changes they concern. For a close-out, write the status files where the code still lags or departs, the designs' Later sections and the draft's fate, then report what you wrote.
```

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.9 (the bullets "A settings file" and
  "Designers and the cut") and 13.7 (a document in a settings file).
- `temper/docs/design/client/tools.md`, 5.3 (a settings file's form).
- `temper/docs/plan/reference-m3/notes/common.md`; contracts C4.

## Owns and may change

- Owns the three files in the header.
- Options: if the settings form's decoder (`settings_form::file::decode` with
  `temper_client_tools::limits::form()`) refuses an instruction text for its length, report the
  bound and the text's bytes and stop with a `Question:`; never cut the text yourself. If it
  refuses a document's member as written, follow the form `settings_form` decodes (13.7's
  `{"forge", "repository", "branch", "path", "heading"}`) and say what differed.

## Steps

1. Edit the file with a JSON-aware tool (`python3 -I -c` with `json.load`/`json.dump`, `indent=2`,
   `ensure_ascii=False`), keeping the members' order as the file has them; the instruction
   texts verbatim from this spec.
2. Extend the test; `heavy cargo nextest run -p temper --lib -E 'test(=tests::reference_settings::the_reference_settings_file_decodes)'`
   (find its exact path with `--list` if that selects none).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper
   --all-targets -- -D warnings`; the citations check (common.md). Commit and push.

## Report

As fab's implementer text says.
