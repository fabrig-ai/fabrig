+++
id = "skein-machines"
part = "skein"
kind = "docs"
needs = []
files = [
  "skein/docs/design/kv.status.md",
  "skein/docs/design/io.status.md",
  "skein/docs/design/channel.status.md",
  "skein/docs/design/forge-connection.status.md",
  "skein/docs/design/forge.status.md",
]
effort = "high"
minutes = 45
+++

# skein: the backfill's design gaps in kv, io, channel, forge-connection and forge

## Goal

The debt backfill found skein design gaps that the code shows and no status file holds
(`temper/docs/plan/status-lines/inputs/request.md`, skein 1, 3, 4, 7 and 9). This task writes
those of five status files, after checking the code; `skein-rest` writes the others and
`STATUS.md`. Docs only.

Acceptance:

- Each gap below that the code still shows has one entry in its file, in the form
  `notes/common.md` sets; each it no longer shows is under `## Dropped` in the report.
- A file that said `No gap.` holds its bullet in that line's place.
- No other line of the five files changes, but for their date lines (today's).
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new.
- `git diff --stat origin/main` shows the five files alone.

## Inputs

- `temper/docs/plan/status-lines/notes/common.md` (the standing rules: form, labels, dates).
- `temper/docs/plan/status-lines/inputs/request.md`, section `skein`.
- `DESIGN.md`, 2. `jig/docs/design/examples.status.md` (the code's-side form).
- `skein/docs/foundation/programming-model.md`, 5.4 (state machines), read only.
- Each debt item named below, its **What** (what the code does), read only.

## Owns and may change

- The five files of `files`, each only by the entries below and its date line.
- Not `skein/docs/design/STATUS.md`: `skein-rest` owns it and drops these documents from its
  "have no gap" list. Not the debt items.

Pre-approved:

- **The 5.4 entries are departures from the foundation**, which has no status file: each is
  labelled with its own document's section that describes the machine (`forge-connection.md`,
  8; `kv.md`'s section on the store's requests and commits; and so on), says what
  `skein/docs/foundation/programming-model.md`, 5.4 asks (one explicit state enum per
  lifecycle, each state holding exactly its data, no lifecycle in flags or `Option`s beside it,
  exhaustive matches, one handler per cell), names the part of it the code breaks, as the debt
  item's **What** says, and ends "no reason is recorded" and the code's side.
- If the debt item's **What** no longer matches the code (`rg` its named fields or types), drop
  that part, keep what still holds, and say so in the report.

## Steps

1. Read the note; `date -u +%F`.
2. Write the entries:
   1. `kv.status.md`:
      - the 5.4 departure (skein 1): `skein-kv/src/store.rs`'s store keeps its lifecycle in flags, with the
        subset switched off; the code's side is
        `skein/docs/development/debt/D017-kv-store-states.md`.
      - **Not re-checked** (skein 9): the kv world against `kv.md`, 9. Read section 9 and
        `skein/tests/kv` (it now has crash and model tests); write an entry labelled 9 only for
        what 9 asks that the world still lacks, and drop it if nothing is missing. No debt
        item named.
   2. `io.status.md`:
      - the 5.4 departure (skein 1): the pipe, signals and `FileIo` machines keep state in flags and
        literals (`skein-io/src/pipe.rs`, `signals.rs`, `file_layer.rs`); the code's side is
        `skein/docs/development/debt/D018-io-pipe-and-file-states.md`.
      - 5 (skein 3): no rule tells `Io`'s completions from `FileIo`'s: their operation tokens
        can collide, and hand bit tags work around it (D044's **Where**); the code's side is
        `skein/docs/development/debt/D044-io-token-tags.md`, which smith's
        `smith/docs/development/debt/D003-host-io-route-table.md` waits on. One entry serves
        both: smith's triage asks for the same line.
   3. `channel.status.md`: the 5.4 departure (skein 1): `skein-channel/src/machine.rs` keeps its lifecycle in
      eleven bools; the code's side is `skein/docs/development/debt/D019-channel-machine-flags.md`.
   4. `forge-connection.status.md`:
      - 8 (skein 1): the connections keep their state in flags beside a thin tag
        (`skein-forge-connection/src/connection.rs`, `hooks/connection.rs`,
        `installations.rs`), against 5.4; the code's side is
        `skein/docs/development/debt/D020-forge-connection-states.md`.
      - 8.1, 8.4 (skein 7): a forge connection sets its deadline in one place, and the webhook
        connection has four states, where 8.1 and 8.4 say otherwise (read both, and write
        what each says against what the code does); the code's side is D020 too.
      - **Not re-checked** (skein 9): Forgejo's own connection is drawn from the calls' pool,
        against 7.2. Read 7.2 and the pool's code (`rg -n 'pool' skein/crates/skein-forge-connection/src`);
        write an entry labelled 7.2 only if the code still draws it so and 7.2 says otherwise.
        No debt item named.
   5. `forge.status.md`:
      - the 5.4 departure (skein 1): Forgejo's plans keep a `Stage` beside bags of fields
        (`skein-forge/src/forgejo/plans`); the code's side is
        `skein/docs/development/debt/D021-forgejo-plan-states.md` (its measuring twice is
        debt, not a gap: leave it out).
      - 4.1 (skein 4): GitHub's dialect stands on Forgejo's modules (`skein-forge/src/github`
        uses `skein-forge/src/forgejo`'s; D010's **What** names which), against 4.1, where a
        dialect renders the vocabulary on one forge's API (read it, and name what 4.1 has a
        dialect hold that GitHub's borrows); the code's side is
        `skein/docs/development/debt/D010-github-on-forgejo-modules.md`.
      `forge.status.md` has a **Not yet:** bullet only: the two entries make a new
      `- **Departs:**` bullet after it.
3. Set the date lines; `git diff` shows only the entries and dates.
4. No build or test: `rg -n 'kv.status|io.status|channel.status|forge-connection.status|forge.status' skein --glob '*.rs'`
   must find nothing (if it finds a test, run it and report it).
5. Commit (one commit), then the citation check; fix your lines until it reports nothing new.
6. Push your branch.

## Report

As the implementer's text and the note say: each entry written (file, label, first words), each
dropped gap with what you found, the labels you chose, and the citation check's output.
