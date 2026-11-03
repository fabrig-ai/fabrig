+++
id = "skein-rest"
part = "skein"
kind = "docs"
needs = []
files = [
  "skein/docs/design/oauth.status.md",
  "skein/docs/design/llm.status.md",
  "skein/docs/design/llm-connection.status.md",
  "skein/docs/design/examples.status.md",
  "skein/docs/design/lib.status.md",
  "skein/docs/design/forge-setup.status.md",
  "skein/docs/design/testing.status.md",
  "skein/docs/design/shell.status.md",
  "skein/docs/design/fake-forge.status.md",
  "skein/docs/design/fake-llm.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "high"
minutes = 50
+++

# skein: the backfill's design gaps in ten status files, and the index

## Goal

The debt backfill found skein design gaps that the code shows and no status file holds
(`temper/docs/plan/status-lines/inputs/request.md`, skein 1, 2, 5, 6, 8 and 9). This task writes
those of ten status files, after checking the code, and brings `skein/docs/design/STATUS.md`'s
summary up to both skein tasks' entries; `skein-machines` writes kv's, io's, channel's,
forge-connection's and forge's. Docs only.

Acceptance:

- Each gap below that the code still shows has one entry in its file, in the form
  `notes/common.md` sets; each it no longer shows is under `## Dropped` in the report.
- A file that said `No gap.` holds its bullet in that line's place.
- `STATUS.md` changes as step 3 says, and in no other way.
- No other line of the eleven files changes, but for their date lines (today's).
- `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new.
- `git diff --stat origin/main` shows the eleven files alone.

## Inputs

- `temper/docs/plan/status-lines/notes/common.md` (the standing rules: form, labels, dates).
- `temper/docs/plan/status-lines/inputs/request.md`, section `skein`.
- `DESIGN.md`, 2 and 4 (`STATUS.md`'s summary). `jig/docs/design/examples.status.md` (the
  code's-side form).
- `skein/docs/foundation/programming-model.md`, 5.4 (state machines) and
  `skein/docs/foundation/testing-strategy.md`, 4 (fakes), read only.
- Each debt item named below, its **What** (what the code does), read only.

## Owns and may change

- The eleven files of `files`, each only by the entries below, step 3 and its date line.
- Not the five files `skein-machines` owns, nor the debt items.

Pre-approved:

- **The 5.4 entries are departures from the foundation**, which has no status file: each is
  labelled with its own document's section that describes the machine (`oauth.md`, 3;
  `examples.md`, 3.4; `lib.md`, 12; and so on), says what
  `skein/docs/foundation/programming-model.md`, 5.4 asks (one explicit state enum per
  lifecycle, each state holding exactly its data, no lifecycle in flags or `Option`s beside it,
  exhaustive matches, one handler per cell), names the part of it the code breaks, as the debt
  item's **What** says, and ends "no reason is recorded" and the code's side.
- If the debt item's **What** no longer matches the code (`rg` its named fields or types), drop
  that part, keep what still holds, and say so in the report.
- **Overlap with another draft:** `public-reading-build/public-status` may change `STATUS.md`
  too. Make step 3's two edits and nothing else in it, so the branches share as little as they
  can.

## Steps

1. Read the note; `date -u +%F`.
2. Write the entries:
   1. `oauth.status.md`: the 5.4 departure (skein 1): the oauth client and its accounts keep their
      lifecycles in flags and parallel lists (`skein-oauth/src/machine.rs`,
      `skein-oauth-accounts/src/component.rs`, `exchange.rs`); the code's side is
      `skein/docs/development/debt/D022-oauth-states.md`.
   2. `llm.status.md`:
      - the 5.4 departure (skein 1): `skein-llm/src/client.rs`'s states hold no data; the code's side is
        `skein/docs/development/debt/D023-llm-client-states.md`.
      - **Not re-checked** (skein 9): every Responses request sends `"verbosity":"low"`
        (`skein-llm/src/translate.rs`, the `verbosity` key). Read `llm.md`'s sections on the
        request's encoding (4.5, 4.8) and `rg -n verbosity skein/docs/design/llm.md`; if the
        design has no text on it, write a **Departs:** entry labelled with the section that
        lists the request's fields; if the design now says it, drop it. No debt item named.
      Append both to the file's **Departs:** bullet; its section 5 entry stays word for word
      (`skein/tests/llm/tests/live.rs`'s `#[ignore]` reasons cite it).
   3. `llm-connection.status.md`: the 5.4 departure (skein 1): `skein-llm-connection/src/call.rs`'s call
      uses data-less states; the code's side is D023 too.
   4. `examples.status.md`: the 5.4 departure (skein 1): the echo, which the template applications copy,
      keeps its stop in flags (`skein/examples/echo/protocol/src/layer.rs`, `conn.rs`),
      against `examples.md`, 3.4 and the foundation's 5.4; the code's side is
      `skein/docs/development/debt/D024-echo-template-flags.md`. The file has a **Not yet:**
      bullet only: the entry makes a new `- **Departs:**` bullet after it.
   5. `lib.status.md`: 12 (skein 1): `RequestTable` (`skein-lib/src/requests.rs`), as D025's
      **What** says it departs from 5.4; the code's side is
      `skein/docs/development/debt/D025-request-table-in-lib.md`. New `- **Departs:**` bullet
      after the **Not yet:** one.
   6. `forge-setup.status.md`: 7 (skein 2): 7 has the setup world play the host component;
      the code writes the world twice, and the Forgejo stories run on a plans-only `World`
      (`skein/tests/forge/setup/src/world.rs`, `host.rs`); the code's side is
      `skein/docs/development/debt/D031-forge-setup-two-worlds.md`.
   7. `testing.status.md`: 6 (skein 5): sibling `*_tests.rs` files, inline test modules and
      worlds used as kits, against 6's layout; the code's sides are
      `skein/docs/development/debt/D027-step-test-layouts.md` and
      `skein/docs/development/debt/D028-worlds-as-kits.md`. Append to the **Departs:**
      bullet, after its `tests/` directories sentence, which stays word for word.
   8. `shell.status.md`: 7.5 (skein 6): the keeper's syscalls are in skein-shell's public API
      (`skein-shell/src/process.rs`, `linux.rs`, `darwin.rs`), which 7.5 does not design; the
      code's side is `skein/docs/development/debt/D045-shell-keeper-syscalls.md`. New
      `- **Departs:**` bullet after the **Not yet:** one.
   9. `fake-forge.status.md` and `fake-llm.status.md` (skein 8): "no shared types",
      `skein/docs/foundation/testing-strategy.md`, 4, against fakes that share their
      product's types: fake-forge's server halves (`fake-forge.md`, 4 and `skein/docs/design/forge.md`, 4.1:
      the dialect holds "the server's side of each document, for the fake forge") in
      `fake-forge.status.md`, and fake-llm's shared codec (`fake-llm.md`, 4) in
      `fake-llm.status.md`; fake-forge's entry adds that the oauth fake, the fake channel and
      the fake CDP peer share theirs too (name each crate as `rg` finds it). Each entry says
      the backfill recommends amending `testing-strategy.md`, 4, settled when
      `skein/docs/development/debt/D043-browser-kit-json-and-tables.md` is planned. In
      `fake-forge.status.md`, a new `- **Departs:**` bullet after the **Not yet:** one.
3. `STATUS.md`, two edits:
   1. The sentence "`README.md`, `channel.md`, ... and `browser.md` have no gap." loses
      `channel.md`, `kv.md`, `fake-llm.md`, `forge-connection.md`, `forge-setup.md`,
      `llm-connection.md` and `oauth.md` (each now has an entry from this task or
      `skein-machines`), and keeps the others in their order.
   2. Directly before "The foundation's rules (`../foundation/`) have no status file", one
      sentence, in the summary's words, near: "The state machines of kv, io, channel,
      forge-connection, Forgejo's plans, oauth and its accounts, llm, llm-connection, the echo
      and lib's `RequestTable` depart from `../foundation/programming-model.md`, 5.4, and the
      fakes share their product's types against `../foundation/testing-strategy.md`, 4
      (`fake-forge.md`, `fake-llm.md`); single departures stand in `forge-setup.md`,
      `forge.md`, `forge-connection.md`, `io.md`, `shell.md` and `testing.md`." Drop from it
      a document whose entries both tasks dropped (`skein-machines`' are on its branch or the
      trunk; if not yet merged, keep its documents and say so in the report).
4. Set the date lines; `git diff` shows only the entries, step 3 and the dates.
5. No build or test: `rg -n 'oauth.status|llm.status|llm-connection.status|examples.status|lib.status|forge-setup.status|testing.status|shell.status|fake-forge.status|fake-llm.status|STATUS.md' skein --glob '*.rs'`
   finds only `skein/tests/llm/tests/live.rs`'s `#[ignore]` reasons (which read
   `llm.status.md`, 5 as text, not as a file); a test that reads a file is a test to run,
   reported.
6. Commit (one commit), then the citation check; fix your lines until it reports nothing new.
7. Push your branch.

## Report

As the implementer's text and the note say: each entry written (file, label, first words), each
dropped gap with what you found, the labels you chose, the `STATUS.md` sentences as written,
and the citation check's output.
