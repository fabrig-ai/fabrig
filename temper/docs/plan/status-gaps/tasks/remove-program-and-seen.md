+++
id = "remove-program-and-seen"
part = "jig-compat"
kind = "migration"
needs = []
files = [
  "jig/crates/jig-domain-core/src/documents/task_wait.rs",
  "jig/crates/jig-domain-core/src/documents.rs",
  "jig/crates/jig-client-domain/**",
  "jig/crates/jig-client-protocol-wire/**",
  "jig/crates/jig-client-protocol-link/**",
  "jig/crates/jig-protocol-engine-clients/src/translate/task_wait.rs",
  "jig/crates/jig-protocol-engine-clients/src/tests/llm.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/values/llm.rs",
  "jig/crates/jig-client-view/src/words/**",
  "jig/crates/jig-client-view/src/tests/tree/cards_objects/task_contract.rs",
  "jig/crates/jig-client-view/golden/rendered/tree/cards_objects/task_contract/refused_chat_header/**",
  "jig/crates/jig-client/src/tests/**",
  "jig/tests/channel/tests/suite/program_runs.rs",
  "jig/crates/jig-domain-people/src/domain/start.rs",
  "jig/crates/jig-domain-people/src/tests.rs",
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/choice.rs",
  "temper/crates/temper-client-tools/src/jig/render/document/llm.rs",
  "temper/crates/temper-client-tools/src/tests/jig/reads.rs",
  "jig/docs/design/STATUS.md",
  "jig/docs/design/domain/hosts.md",
  "jig/docs/design/domain/hosts.status.md",
  "jig/docs/design/domain/people.status.md",
  "jig/docs/design/protocol/client.status.md",
  "jig/docs/design/protocol/browser.status.md",
  "jig/docs/design/protocol/configuration.status.md",
  "jig/docs/design/client/domain/contract.status.md",
  "jig/docs/design/client/domain/client.md",
  "temper/docs/design/client/README.md",
]
effort = "high"
minutes = 120
+++

# Remove `TaskWaitRefused::Program` and the kept `Seen` compatibility

## Goal

The design dropped the program refusal of a task's wait (the user, 2026-11-02) and keeps no
stored `Seen` (`jig/docs/design/protocol/client.md`, 10: "A top-level `Seen` has no stored
record"; `README.md`, 7: nothing is translated before the baseline). The code still carries
both. Remove `TaskWaitRefused::Program` (domain, client contract, client wire, translations,
scripted engine, jig's and temper's words and renderers) and the restore of a stored `Seen`, in
the client's small store and in the people domain, with their tests and status entries. The wire
stays `client-v19`, changed in place (`protocol/README.md`, 7).

Acceptance:

- No `TaskWaitRefused::Program`, `OpenProgram`, `TaskWaitRefusedProgram`, `TaskWaitProgram` or
  `TaskWaitProgramSource` in jig or temper (`rg -n 'TaskWaitRefusedProgram|TaskWaitProgram|OpenProgram|TaskWaitRefused::Program' jig temper --glob '!**/docs/plan/**'`
  finds nothing); `client-v19.schema` keeps `hello` at tag 0 (its bytes unchanged), its
  generated `v19.rs` and goldens regenerated, the program goldens deleted by the drift test's
  write mode.
- A stored top-level `Seen` record is not restored: the client drops it, counts it in
  `Told::Dropped`, and erases its key at start (below). A test shows it.
- The people domain loads no stored `Seen` answer: a kept `Answer` whose ask is `MarkSeen` is
  skipped at load and erased at the sweep, as an expired answer is. A test shows it.
- The status entries and design sentences below are updated; clippy is clean; the tests in
  Steps pass.

## Inputs

- `temper/docs/plan/status-gaps/notes/common.md`.
- `jig/AGENTS.md` and `temper/AGENTS.md`.
- Status entries (the spec): `jig/docs/design/domain/hosts.status.md` (8.1),
  `protocol/client.status.md` (3 and 10), `client/domain/contract.status.md` (5.1),
  `domain/people.status.md` (5.1, the `Seen` answers), `protocol/browser.status.md` (10),
  `protocol/configuration.status.md` (6: the people child replays a kept `Seen` answer),
  `STATUS.md` (the clause "where the client wire keeps `TaskWaitRefused::Program` and stored
  `Seen` records and answers ...").
- Design: `jig/docs/design/client/domain/contract.md`, 5.1 (only `Hello` refuses a wait);
  `domain/hosts.md`, 8.1; `protocol/client.md`, 9 and 10; `client/domain/client.md`, 11 (the
  dropped records and `Dropped { count }`); `protocol/browser.md`, 10 (`Seen` alone does not
  warn); `domain/people.md`, 5.1.

## The lead's calls (no question needed)

- **The wire stays v19,** changed in place: jig's `protocol/README.md`, 7 ("Until the first
  release ... changes to its schema, generated code and golden bytes are made in place").
- **A stored top-level `Seen` in the client:** dropped and told as any record the client does not
  read (`client.md`, 11), and its key erased at start with the table's own erase
  (`down(out, Down::Erase { key })`, as a terminal erases), so the small store holds no `Seen`
  and `Seen` alone never warns (`browser.md`, 10). Do it in `Requests::restore_record`
  (`jig-client-domain/src/requests.rs`): an `Ask::Jig(Seen)` record is invalid, counted dropped,
  and its key erased. The link (`jig-client-protocol-link/src/translate/store.rs:144`) keeps
  decoding a `w::Ask::Seen` (the ask is live on the wire); it does not refuse it.
- **A stored `Seen` answer in the people domain:** `jig-domain-people/src/domain/start.rs` (load
  ~248-252, fit ~343-349) skips a `Stored::Answer` whose ask is `MarkSeen` and lets the start's
  sweep erase it, as it erases an expired answer (`expired()`, ~148-150, 192-197, 216-218). It
  never refuses the start. `AskMarkSeen` stays in `people-v14.schema`: it is a live ask.

## Owns and may change

jig (owned):

- `jig-domain-core`: `documents/task_wait.rs` (the `Program` variant, `OpenProgram`, their
  comments), `documents.rs` (the re-export).
- `jig-client-domain`: `contract/llm.rs` (the variant and `OpenProgram`); `requests.rs`
  (`restore_record`, as above); `tests/requests.rs` (`a_kept_seen_record_restores_without_resaving_then_erases`
  becomes `a_kept_seen_record_is_dropped_and_erased_at_start`, asserting `Dropped { count: 1 }`,
  a `Down::Erase` of its key, and no `Kept`).
- `jig-client-protocol-wire`: `schema/client-v19.schema` (`TaskWaitProgramSource`,
  `TaskWaitProgram`, `TaskWaitRefusedProgram` and the `program:` arm, ~676-679);
  `src/generated/v19.rs` and `golden/` regenerated, never by hand; `src/tests/task_refusal.rs`
  rewritten on the `hello` arm.
- `jig-client-protocol-link`: `translate/task_wait.rs` (the arm, ~15-20); its tests.
- `jig-protocol-engine-clients`: `translate/task_wait.rs` (the arm, ~22-35); `tests/llm.rs`:
  delete `program_refusals_survive_server_bytes_and_client_task_translation`.
- `jig-client-scripted-engine`: `codec/values/llm.rs` (the arm and `wire_task_wait_program`).
- `jig-client-view`: `words/words/status.rs` (~79-104), `words/phrases/jig.rs` (the program
  phrases, their ids, hole shapes, `ALL` and its length), `words/phrases/english.rs` (their
  English); `tests/tree/cards_objects/task_contract.rs` (the program cases) and its goldens under
  `golden/rendered/tree/cards_objects/task_contract/refused_chat_header/`: regenerate, then
  `git rm` the stale files the write mode leaves (it does not delete them).
- `jig-client` tests: `src/tests/facts.rs` (rows and
  `refusal_status_only_replaces_an_unplaced_starting_chat` take `Hello` as their cause);
  `src/tests/store.rs` (`a_kept_seen_record_decodes_restores_and_erases_without_resaving`
  becomes a test that the record is dropped, told `Dropped`, and erased);
  `src/tests/fixtures.rs` (`kept_seen`, if still used).
- `jig/tests/channel/tests/suite/program_runs.rs`: the assertion that `Program` never appears
  (~151-154) goes.
- `jig-domain-people`: `domain/start.rs` (as above); `src/tests.rs` (~272-326: the kept `Seen`
  answer is skipped and erased, not replayed).

temper (readers, edited only because jig's removal does not compile without it; delete only the
`Program` arms, words, renderers and test rows):

- `temper-client-view`: `src/choice.rs` (~728-737), `src/words.rs` (the program words, their
  `ALL`, ids and English; `PHRASES` and `SEGMENTS` regenerated by their test, never by hand),
  `src/tests/choice.rs` (`a_refused_next_run_says_why` keeps its `Hello` case alone).
- `temper-client-tools`: `src/jig/render/document/llm.rs` (the `program` text and JSON forms),
  `src/tests/jig/reads.rs` (`a_refused_next_run_names_its_cause_in_text_and_json` keeps its
  other causes).

Design and status (as `notes/common.md` says):

- Delete: `domain/hosts.status.md`'s 8.1 "Departs" item; `protocol/client.status.md`'s "3:" and
  "10:" sentences (17.4 stays); `client/domain/contract.status.md`'s 5.1 entry (3.2 stays);
  `domain/people.status.md`'s 5.1 sentence on `Seen` answers (the rest stays);
  `protocol/configuration.status.md`'s "6:" (5 stays); `protocol/browser.status.md`'s only
  entry, the file then saying `No gap.`; `STATUS.md`'s clause on the client wire's
  `TaskWaitRefused::Program` and stored `Seen` records and answers, the sentence mended.
- `jig/docs/design/client/domain/client.md`, 11: the sentence "A `Seen` record the small store
  holds is restored as any kept request, told `Kept`, never saved again, and erased at its
  terminal." becomes "A `Seen` record the small store holds is not read: it is dropped, counted
  in `Dropped`, and its key erased at start."
- `jig/docs/design/domain/hosts.md`, 8.1: the sentence's last clause ", and no run waits as
  `OpenProgram`" is deleted, the sentence ending "the binding revalidated exactly."
- `temper/docs/design/client/README.md`, 3.5 (~lines 936-941): delete the two program causes,
  "for the model, "This model runs through a program. ..."; for an account's source, "Account A
  would use its program source S. ..."", leaving the host's hello and the rest as they are.

Pre-approved options:

- If a test elsewhere in jig or temper builds or matches `TaskWaitRefused::Program`
  (`rg` above), delete that case and keep the test's other cases; list it.
- If `jig-domain-people`'s sweep cannot erase a skipped `Seen` answer without a new public item,
  skip it at load only, leave it stored, and say so in the report (people.status 5.1's sentence
  then becomes "a kept `Seen` answer an earlier build stored is skipped at load but not erased");
  this is the one design change allowed beyond the list.
- `jig/scripts/old-forms.txt` counts none of these forms; if a count falls, `--write` it.
- Ladder overlap: `v0.1.0-ladder/finding-4` changes `client-v19-part1.schema` and so
  `generated/v19.rs`, and `client/domain/contract.status.md`. If your base already holds it,
  nothing to do; the generated file is always regenerated, never merged by hand.

## Steps

1. Program: remove it bottom-up (domain core, client contract, schema, then the readers). Run
   the drift test in write mode, then without:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world --test suite -E 'test(=drift::codec_families_match_their_schemas_and_goldens)'`
   (check the name with `--list` first). `git diff --stat` on `golden/` shows only program
   goldens deleted.
2. The compile repair across both trees: `heavy cargo check -p jig-client-view -p jig-client -p jig-protocol-engine-clients -p jig-client-scripted-engine -p temper-client-view -p temper-client-tools --tests`
   while errors last.
3. Seen: `restore_record` and the people domain's load, with their tests.
4. Narrow tests, one at a time:
   - `heavy cargo nextest run -p jig-client-protocol-wire --lib`
   - `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-view --lib -E 'test(refused_chat_header_shows_the_supplied_cause_and_receipt_time)'`, `git rm` the stale goldens, then again without the variable
   - `heavy cargo nextest run -p jig-client-domain --lib`
   - `heavy cargo nextest run -p jig-client-protocol-link --lib`
   - `heavy cargo nextest run -p jig-client --lib`
   - `heavy cargo nextest run -p jig-protocol-engine-clients --lib`
   - `heavy cargo nextest run -p jig-domain-people --lib`
   - `heavy cargo nextest run -p jig-channel-world --test suite -E 'test(/program_runs/)'`
   - `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-client-view --lib -E 'test(english_phrase_declarations_match_the_vocabulary)'`, then `heavy cargo nextest run -p temper-client-view --lib`
   - `heavy cargo nextest run -p temper-client-tools --lib`
   Each count is not zero.
5. A change of meaning (a stored `Seen` is now dropped and erased): also run
   `heavy cargo nextest run -p jig-client-shell-web --lib` if it has tests of kept records, and
   `heavy cargo check -p jig-client-browser-world -p temper-client-browser-world --tests`.
6. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-client-domain -p jig-client-protocol-wire -p jig-client-protocol-link -p jig-protocol-engine-clients -p jig-client-scripted-engine -p jig-client-view -p jig-client -p jig-domain-people -p jig-channel-world -p temper-client-view -p temper-client-tools --all-targets -- -D warnings`.
7. The design and status edits; the acceptance's `rg`.
8. Commits: the program's removal (code, schema, goldens, both trees' readers); the `Seen`
   compatibility's removal; the design and status text. Push your branch.

## Report

As the implementer's text says, and also: the goldens deleted and regenerated (by `git diff
--stat`); how the people domain's sweep erases a skipped `Seen` answer (or the option taken);
each temper edit, by file.

## Answers

2026-11-02 (answer call, for run `remove-program-and-seen-1`):

- **May the task also edit `jig/docs/design/domain/hosts.md`, 8.1, to delete "and no run waits
  as `OpenProgram`"?** Yes. The acceptance's `rg` cannot hold without it, and the clause states
  only the removed form's absence, so deleting it is the rule of `notes/common.md` ("a sentence
  about something removed is deleted"). Delete ", and no run waits as `OpenProgram`", leaving
  "... and the binding revalidated exactly." Change nothing else in `hosts.md`; the file is now in
  `files` and in "Design and status". Do not exempt the sentence from the search.
