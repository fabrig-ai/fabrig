+++
id = "wait-words"
part = "words"
kind = "feature"
needs = ["bounds-wait"]
files = [
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/tests/choice.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/model_check.rs",
  "temper/crates/temper-client-tools/src/jig/render/document/llm.rs",
  "temper/crates/temper-client-tools/src/tests/jig/reads.rs",
  "temper/crates/temper-client-tools/src/tests/printed.rs",
  "temper/crates/temper-client-tools/src/tests/fixtures.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/tools.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 120
+++

# A run waiting for a host says why, with a command that exists

## Goal

proof-2's goal said "no host has room, or none is connected: deployment show", a command ctl doesn't have, while its charter's 100,000 turns kept it off an 800-turn worker for good. `bounds-wait` carries that bound to the client (`TaskWaitRefused::Bounds`, a hello's `figures`); here the page, the terminal and ctl say it in a person's words, "No worker can take this: its charter asks 100,000 turns, and the largest worker takes 800.", for every bound, and every command a waiting line names exists, checked by a column of an existing table.

This task departs from the implementer's text in one way: three status entries are yours to remove and one summary line to cut (acceptance 7), because the design is already written.

Acceptance:

1. **The page's words** are `temper/docs/design/client/README.md`, 5.5, *No worker can ever take it*, exactly, with its table's bounds: `every_hello_refusal_keeps_its_cause_and_numbers` (`temper-client-view/src/tests/choice.rs`) keeps every row and gains a `Bounds` row for each of the ten bounds with figures, one with `why` none, one with figures none, one on the engine's own host, and a refused hello with figures (turns, charter 3); its old checks row reads "... See temper help settings for held revisions."
2. **The meeting point:** `every_chats_run_wait_says_why_above_the_composer_without_working_dots` (`tests/chat.rs`) gains the row `Refused(Bounds { kind: Worker, why: Some(Turns { charter: 1 }), figures: Some(Figures { required: 100000, offered: 800 }) })`, on a box, reading "No worker can take this: its charter asks 100,000 turns, and the largest worker takes 800." above the composer in both the terminal's and the browser's views.
3. **ctl's lines** are `temper/docs/design/client/tools.md`, 6.1's: the host wait's `no host has room, or none is connected; it starts when one does`; a bounds refusal's code line and its line in the client's form; a refused hello with figures' line in place of `its host refused this start: …`; turns, time and waiting naming their charter; the checks' `see temper help settings for held revisions`; `its account needs sign-in: account connect PROVIDER --key KEY`. `--json` is `notes/contracts.md`, *ctl's JSON*: the rows of `a_refused_next_run_names_its_cause_in_text_and_json` (`tests/jig/reads.rs`) gain `"figures":null` and rows for a bounds refusal with and without `why`, and a hello with figures.
4. **Every pointer is checked** (request, item 1): each row of `task_waits_explain_the_next_act_after_the_code_line` (`tests/jig/reads.rs`) gains a column, the commands its explanation names, and the test checks each: the explanation contains it; a ctl call parses through `crate::table::parse` once its placeholders are filled as `printed.rs`'s `substitute` fills them; a `temper help NAME` names a page `temper_docs` holds; the program login's command, the limits' `program_login`, is checked to be that prefix. Rows for a bounds refusal and a hello with figures name none; the checks' row names `temper help settings`. The empty account wait's lines, pinned by `empty_account_wait_names_claude_login_path`, get the same check over the commands they name (`account connect PROVIDER --key KEY`, `account add-key PROVIDER FILE --key KEY`, `account declare PROVIDER claude_code NAME --key KEY`, `account list`, `account verify ACCOUNT --source program`, the program login's). Put back `deployment show` by hand once: the row must fail on its group, as `no group deployment` does; restore.
5. **The page's waits name no ctl command:** the loop of acceptance 1's test and of acceptance 2's also checks that no expected text holds `temper ctl `, and that each `temper help NAME` in one names a page `temper_docs` holds.
6. A run waiting only for room keeps its words: on a box "Waiting for other work to finish. It starts by itself.", otherwise "No host has room for this, or none is connected. It starts when one does." (the rows already in acceptance 2's test, unchanged).
7. The entries are gone: `temper/docs/design/client/README.status.md`'s *Not yet: 3.5, 5.5: a run no worker can ever take reads...* and `temper/docs/design/client/tools.status.md`'s *Not yet: 5.2, 6.1: `task show`'s text names commands that aren't ctl's...*; and `temper/docs/design/STATUS.md`'s summary loses its clause "and `task show` names a `deployment show` ctl lacks".
8. Focused tests pass in `temper-client-view`, `temper-client-tools`, `temper-client-shell-terminal`, `temper-client-shell-cli` and `temper-docs`.

## Inputs

- `temper/docs/plan/ui-wait-words/notes/contracts.md`: the client contract's types, and *ctl's JSON*, which binds.
- `temper/docs/plan/ui-wait-words/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.5, *A run's wait*: *A host*, *No worker can ever take it* and *A command it names exists*; 3.5, the paragraph *A task's wait for its account and a refused next run*.
- `temper/docs/design/client/tools.md`, 6.1, *`next run:`*.

## Owns and may change

**Owned:** the `files` above, which `bounds-wait` left compiling with placeholder arms (`notes/contracts.md`, *What the `bounds` part leaves in temper*): replace each.

**Readers and neighbours:**

- `temper/crates/temper-client-tools/Cargo.toml`: no new dependency is expected; `temper-docs` is already its dependency.
- A golden or print the words change (the terminal shell's tests, `temper-client-tools`' `src/tests/golden/`): regenerate by its own tool and say so. None is expected: the survey found no print of these words outside the tests above.
- `temper/docs/help/task.md`: if it says what `next run:` prints for a host's wait, one line may follow the design; within its byte bound.

**Options, each with its default:**

- **Grouped digits:** `pages/session.rs`'s `over` groups a count with commas inline. Default: move that grouping to one `pub(crate)` helper in `words.rs` that both use, and say counts and spend through it. If moving it would touch more than `over`, copy nothing: call it from `session.rs` as it is, made `pub(crate)`, and say so.
- **Durations:** say a bound's milliseconds through the span words the page already has (`Words::time_span`'s "N min", "N h", "N h M min"; under a minute its words); a figure under a minute is said as that word.
- **The words' shape:** one `Word` per bound's clause, holes for the figures, plus the leads ("No worker can take this: ", "The engine's own host can't take this: ") and the two tails ("and the largest worker takes", "and the worker refused at HH:MM takes", "and it takes"); or one `Word` per bound and subject. Default: the first, joined as `sentences` joins today. Each new word fits the phrasebook's bounds; its byte test names them.
- **A row's commands** (acceptance 4): `&[&str]`, placeholders as printed (`PROVIDER`, `FILE`, `KEY`, `ACCOUNT`, `NAME`). Move `substitute` from `printed.rs` to `fixtures.rs` as `pub(crate)` so both tests fill placeholders one way.
- **`wait_why`** (`choice.rs`, the model check's first line) gives the same first sentence as `waiting` for a `Bounds` refusal; `model_check.rs`'s table keeps its rows and, if it lists refused waits, gains a `Bounds` row.
- **ctl's charters:** `agent_shortfall_text` prints `agent turns too few for charter 3` (and time, waiting), as checks already names its charter.

## Steps

1. The page: `choice.rs`'s `refused`, `hello_refusal`, `agent_shortfall` and `wait_why`, with the words in `words.rs` (English, and every other language or shell the phrasebook holds, as its siblings are).
2. ctl: `render/document/llm.rs`'s `text_wait`, `wait_explanation`, `refused_text`, `refused` (JSON), `agent_shortfall_text`, `source_explanation`'s sign-in line.
3. Tests: acceptance 1 to 6, rows of the tables named. For each new rule, break it once by hand (the largest for the first worker's figure; `deployment show` back; a page word naming `temper ctl `) and see its row fail for that reason; restore.
4. Remove the status entries and the summary clause (acceptance 7).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-tools --all-targets -- -D warnings`; the focused tests of acceptance 8. Commit, push.

## Report

As the implementer's text says, and: the words you chose for any bound the design's table left open (a sub-minute span, a figure without its unit), every command each row names, and any pointer outside a waiting line you saw broken (the survey saw `ForgeReplaceTerminal`'s `temper ctl forge replace N FILE` without `DIR` and `--key`, and `docs/help/reference/settings-fields.md`'s "settings help"): leave those, and list them under `## Left`.
