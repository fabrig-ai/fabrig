+++
id = "smith-lines"
part = "smith"
kind = "docs"
needs = []
files = [
  "smith/docs/design/domain/session.status.md",
  "smith/docs/design/testing.status.md",
  "smith/docs/design/benchmarks.status.md",
  "smith/docs/design/domain/host.status.md",
  "smith/docs/design/STATUS.md",
]
effort = "high"
minutes = 35
+++

# smith: the backfill's design gaps in four status files, and the index

## Goal

The debt backfill found four smith design gaps that the code shows and no status file holds
(`temper/docs/plan/status-lines/inputs/request.md`, smith 1 to 4). Write one entry for each, in
the file named, after checking the code, and bring `smith/docs/design/STATUS.md`'s summary up
to them. Docs only.

Acceptance:

- Each gap below that the code still shows has one entry in its file, in the form
  `notes/common.md` sets; each it no longer shows is under `## Dropped` in the report.
- `session.status.md`'s `No gap.` is replaced by its bullet.
- `STATUS.md` changes as step 3 says, and in no other way.
- No other line of the five files changes, but for their date lines (today's).
- `heavy cargo nextest run -p smith-bench` passes, its count reported.
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new.
- `git diff --stat origin/main` shows the five files alone.

## Inputs

- `temper/docs/plan/status-lines/notes/common.md` (the standing rules: form, labels, dates).
- `temper/docs/plan/status-lines/inputs/request.md`, section `smith`.
- `DESIGN.md`, 2 and 4. `jig/docs/design/examples.status.md` (the code's-side form).
- `smith/AGENTS.md`. The design sections each gap is against, below, read only.

## Owns and may change

- The five files of `files`, each only by the entries below, step 3 and its date line.
- Nothing else: not the debt items, not the design documents' text.

Pre-approved:

- If the code shows a gap in a different file or item than the request names (the session's
  `wait` match has moved, say), write it with what the code has now and say so in the report.
- smith's labels hold sub-bullets in `testing.status.md`, `benchmarks.status.md` and
  `host.status.md`: a new entry there is a sub-bullet, in section order; a label the file lacks
  is a new bullet in the file's form (`- **Departs:**`, then its sub-bullets).

## Steps

1. Read the note; `date -u +%F`.
2. Write the entries:
   1. `domain/session.status.md` (smith 1): the session matches the run's `b"wait"`
      (`smith-domain-session/src/session.rs`, `session/restore.rs`; `rg -n 'b"wait"'
      smith/crates/smith-domain-session/src`), against `session.md`, 1, where a session knows a
      conversation and "nothing of runs, charters, results or hosts"; the code's side is
      `smith/docs/development/debt/D004-session-knows-wait.md`. `No gap.` becomes a
      `- **Departs:**` bullet.
   2. `testing.status.md` (smith 2): `tests/session` and `tests/tools` call protocol crates
      (`tests/session/src/translate.rs`, `tests/tools/src/translate.rs`, and their
      `Cargo.toml`), against `testing.md`, 2: "Domain worlds use domains and fakes only, never
      protocol crates"; the code's side is
      `smith/docs/development/debt/D017-domain-worlds-on-protocols.md`. The file has a
      **Not yet:** bullet only: a new `- **Departs:**` bullet after it, holding this entry as
      its sub-bullet labelled 2.
   3. `benchmarks.status.md` (smith 3): the benchmark runner branches on each agent
      (`benchmarks/src/run.rs`, `process.rs`, `agent/`), against `benchmarks.md`, 6, where
      agents are adapters; the code's side is
      `smith/docs/development/debt/D010-bench-runner-knows-agents.md`. A sub-bullet of
      **Departs:** labelled 6, between the `5.3, 7` and the `7, 10` sub-bullets.
   4. `domain/host.status.md` (smith 4): the local domain composes person-facing text
      (`smith-local-domain/src/person.rs`, `domain.rs`; D018's **Where**), against
      `host.md`, 8; whether this departs is the designer's decision, and the entry says so in
      those words. The code's side is
      `smith/docs/development/debt/D018-domain-formats-text.md`. A new `- **Departs:**`
      bullet after the **Not yet:** one, holding this entry as its sub-bullet labelled 8.
3. `STATUS.md`, two edits:
   1. The sentence "`domain/session.md`, `protocol/charter.md`, `protocol/limits.md` and
      `protocol/transcript.md` have no gap." loses `domain/session.md` (if step 2.1 wrote its
      entry), and reads "`protocol/charter.md`, `protocol/limits.md` and
      `protocol/transcript.md` have no gap.".
   2. "the benchmarks' calibrations and arms, with three departures (`benchmarks.md`)" counts
      four departures if step 2.3 wrote its entry; and directly before the edited "have no
      gap" sentence, one sentence in the summary's words, near: "The session knows the run's
      `wait` (`domain/session.md`), two domain worlds call protocol crates (`testing.md`), and
      the local domain composes person-facing text, which the designer is to decide on
      (`domain/host.md`)." It names only the entries written.
4. Set the date lines; `git diff` shows only the entries, step 3 and the dates.
5. smith's design tree is read by `smith-bench`'s guard checks (`smith/benchmarks/src/check.rs`,
   `check_guard`, which reads design documents' headings, not status files): run
   `heavy cargo nextest run -p smith-bench` once and report its count. Also
   `rg -n 'session.status|testing.status|benchmarks.status|host.status|STATUS.md' smith --glob '*.rs'`:
   a test that reads one of your files is a test to run, reported.
6. Commit (one commit), then the citation check; fix your lines until it reports nothing new.
7. Push your branch.

## Report

As the implementer's text and the note say: each entry written (file, label, first words), each
dropped gap with what you found, the `STATUS.md` sentences as written, the `smith-bench` run's
count, and the citation check's output.
