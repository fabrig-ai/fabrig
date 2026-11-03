+++
id = "temper-lines"
part = "temper"
kind = "docs"
needs = []
files = [
  "temper/docs/design/protocol/README.status.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/protocol/configuration.md",
]
effort = "high"
minutes = 45
+++

# temper: the backfill's design gaps in three status files, and a stale sentence

## Goal

The debt backfill found temper design gaps that the code shows and no status file holds
(`temper/docs/plan/status-lines/inputs/request.md`, temper 1 to 3), and a stale design sentence
(jig 7: `temper/docs/design/protocol/configuration.md` names `hello_bounds`, which no code has).
Write one entry for each gap, after checking the code, and correct the sentence. Docs only.

Acceptance:

- Each gap below that the code still shows has one entry in its file, in the form
  `notes/common.md` sets; each it no longer shows is under `## Dropped` in the report.
- `configuration.md`, 5's `hello_bounds` sentence says what the code does (step 3), and no
  other line of that document changes.
- No other line of the status files changes, but for their date lines (today's).
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new.
- `git diff --stat origin/main` shows only this task's files.

## Inputs

- `temper/docs/plan/status-lines/notes/common.md` (the standing rules: form, labels, dates).
- `temper/docs/plan/status-lines/inputs/request.md`, sections `temper` and `jig` (item 7).
- `DESIGN.md`, 2. `jig/docs/design/examples.status.md` (the code's-side form).
- `temper/docs/design/protocol/README.md`, 4 to 7; `temper/docs/design/protocol/setup.md`, 5
  and 11; `temper/docs/design/client/README.md`, 4.7; `jig/docs/design/protocol/service.md`, 7;
  `temper/docs/design/protocol/configuration.md`, 5, read only (but the sentence of step 3).
- `temper/docs/development/debt/D001-reference-service-copy.md`, `D006-forge-reconcile-dead.md`
  and `D008-pipeline-names-in-code.md`, their **What**, read only.

## Owns and may change

- The files of `files`, each only by the entries below, step 3's sentence and its date line.
- `temper/docs/design/domain/forge.status.md` only as step 2.4 allows.
- Not `temper/docs/design/STATUS.md`: its "have no gap" list names none of these documents.
  Not `jig/docs/design/protocol/service.status.md`: the lead placed temper 2 in temper's file
  (jig's code does not depart; temper's design is silent on its copy).

Pre-approved:

- **Overlap with another draft:** `fabrig-scale` changes `setup.status.md`'s first **Not
  yet:** line, and its open tasks (`init-standard`, `measure-box`) may change the file too.
  Append this task's entry to the end of the **Departs:** bullet and touch nothing in the
  **Not yet:** bullets. `fabrig-scale` also rewrites much of `configuration.md`, but not section
  5's sentence: change that sentence alone.
- If the code shows a gap in a different file or item than the request names, write it with
  what the code has now and say so in the report.

## Steps

1. Read the note; `date -u +%F`.
2. Write the entries:
   1. **`temper-forge-setup-shell/src/reconcile.rs` has no product caller** (temper 1). Check
      it: `rg -n 'reconcile' temper/crates/temper/src temper/crates/temper-service/src
      temper/crates/temper-forge-setup-shell/src/lib.rs` (D006's **What**: only the crate's
      own tests call it; the binary links the crate for `command`, `prepare`, `run`, `fetch`,
      `parent` and `process`). Two entries, the code's side of both
      `temper/docs/development/debt/D006-forge-reconcile-dead.md`:
      - `protocol/README.status.md`: labelled with `README.md`'s section that lists
        `temper-forge-setup-shell` (5, "Components and crates"): the crate keeps
        `reconcile.rs` (`reconcile`, `Settings`, `Running` and the forge files' translation)
        with no product caller, beside the `temper forge reconcile` verb the **Departs:**
        bullet's first entry already records; no reason is recorded. Append it to that
        bullet, after the verb's sentence.
      - `protocol/setup.status.md`: labelled with `setup.md`'s section on the crates (11) or on
        reconcile (5), as fits: the same fact, citing `README.status.md`'s entry by path rather
        than repeating it. Appended to the **Departs:** bullet.
   2. **temper-service is a copy of jig's reference service** (temper 2), in
      `protocol/README.status.md`, labelled 6, 7 (the engine's and the worker's services, "in
      the shape of jig's `protocol/service.md`, section 2"): `temper-service`'s `accounts.rs`,
      `program_sources.rs`, `server.rs`, `store.rs`, `worker_route.rs`, `clients.rs` and
      `workers.rs` are copies of `jig/testing/jig-test-service`'s, which jig's
      `protocol/service.md`, 7 writes "as step code, so that it can be copied", and neither
      document says temper's is such a copy or where it departs from it; no reason is
      recorded. The code's side is `temper/docs/development/debt/D001-reference-service-copy.md`
      (which waits on `jig/docs/development/debt/D005-accounts-host-copied.md`). Check the
      copy with D001's **Measure** (bash, at the repository's root) and put its value in your
      report.
   3. **temper-07's `not_installed`** (temper 3), in `client/README.status.md`, labelled 4.7:
      `client/README.md`, 4.7 words an outcome "not installed" ("temper's App is not
      installed on OWNER.", with "Install on another account"), but `temper-service/src/family.rs`
      projects `forge::AdoptionRefusal::NotInstalled` to the client wire's
      `ForgeReason::NoAccess`: the wire has no not-installed reason
      (`temper-service/src/tests/setup_github.rs` pins it as a known gap). A **Not yet**
      entry, appended to the file's **Not yet:** bullet; the code's side is
      `temper/docs/development/debt/D008-pipeline-names-in-code.md`.
   4. **temper-12's merge-queue answers** (temper 3): `temper-engine-smith/src/answers.rs`
      answers an agent's forge `Answer::Queued` with `unreachable!`. Read
      `temper/docs/design/domain/forge.status.md`'s `4.4, 9.1, 18` entry ("the forge's own
      merge queue beyond a queued merge kept pending") and `rg -n -i 'queued'
      temper/docs/design/domain/agent.md temper/docs/design/domain/forge.md`. If that entry
      covers an agent never hearing a queued merge, write nothing and report it as covered;
      if a design section says an agent hears it and no status entry holds it, append an
      entry to `domain/forge.status.md`'s bullet of that label, citing D008, and say so.
   5. **temper-13's crowded installations** (temper 3): the comment is in
      `temper-forge-setup-shell/src/reconcile.rs`, which step 2.1's entry records as having no
      product caller: no separate entry; report it under `## Dropped` with that reason.
3. **The stale sentence** (jig 7), `temper/docs/design/protocol/configuration.md`, 5 ("The
   worker must fit the configured charters"): "The worker's bounds pass through the same
   `hello_bounds` conversion as its hello and jig's `shortfall` selects the first unmet
   requirement." becomes "The worker's bounds are its agent configuration's `bounds`, which its
   hello carries unchanged, and jig's `shortfall` selects the first unmet requirement." Check
   both halves first: `temper-service/src/startup/checks.rs`, `admission` (`agent.bounds`
   handed to `fleet::shortfall`), and `temper-service/src/worker.rs` (the hello's
   `bounds: agent.bounds.clone()`); `rg -n hello_bounds temper jig --glob '*.rs'` finds
   nothing. If either half differs, write what the code does in the same sentence's place and
   say so in the report. Rewrap the paragraph's lines only as far as the sentence needs.
4. Set the date lines; `git diff` shows only the entries, the sentence and the dates.
5. No build or test: `rg -n 'README.status|setup.status|configuration.md' temper --glob '*.rs'`
   and the same for `client/README.status`: a test that reads one of your files is a test to
   run, reported (temper's tests read `docs/help/`, which this task does not touch).
6. Commit (one commit), then the citation check; fix your lines until it reports nothing new.
7. Push your branch.

## Report

As the implementer's text and the note say: each entry written (file, label, first words), each
dropped gap with what you found, D001's measure, the sentence as written, and the citation
check's output.
