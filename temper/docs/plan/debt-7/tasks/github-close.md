+++
id = "github-close"
part = "fake-github"
kind = "docs"
needs = ["github-connection-face", "github-machine-face", "github-protocol-face", "github-joint-face", "github-fixture-readers"]
files = ["skein/docs/development/debt/D029-fake-github-http-faces.md", "skein/docs/development/debt/README.md", "skein/docs/development/debt/D030-forge-connection-harness-tests.md", "skein/docs/development/debt/D005-civil-calendars.md", "skein/docs/design/fake-forge.status.md"]
effort = "medium"
minutes = 15
+++

# fake-github: pay D029

## Goal

Every face is on the trunk. Check `skein/docs/development/debt/D029-fake-github-http-faces.md`'s **Paid when**, delete
the item and its row, close the design's status entry, and fix what the deletion and the faces leave stale.

Acceptance:

- The item's **Measure** reads 0: `rg -l 'Outcome::Admit' skein/crates/skein-forge-connection/src skein/tests/forge |
  wc -l` (4 at base).
- `rg -n 'service\.request\(|\.service\.admit\(|\.service\.finish\(' skein/crates/skein-forge-connection/src
  skein/tests/forge` prints nothing: nothing in those trees loops over the fake's `Outcome`.
- `rg -n 'fn fixture_(accounts|repositories|app)|\bRow \{|\bAccount \{'
  skein/crates/skein-forge-connection/src/tests/github_files skein/tests/forge/machine/src
  skein/tests/forge/protocol/src` prints nothing: the fixture is built once.
- `rg -n 'token_life:' skein -g '*.rs'` holds only `skein-fake-forge-github`'s `src/service.rs`, `src/fixture.rs` and
  the two overrides in `src/tests/git.rs`.
- The item's file and its row are gone. `skein/docs/design/fake-forge.status.md` has no 10.6 entry.
- `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`; `temper/docs/plan/debt-7/notes/fake-github.md`, 5.
- `skein/docs/development/debt/D029-fake-github-http-faces.md`.
- `DESIGN.md`, 2: a status file's form.

## Owns and may change

This departs from the implementer's text and the common note's "Not yours" for
`skein/docs/design/fake-forge.status.md` only: remove the sentence that begins `10.6:` from its **Not yet** line and
set the date line to today's. The rest of the line stays.

- `skein/docs/development/debt/D029-fake-github-http-faces.md`: deleted.
- `skein/docs/development/debt/README.md`: D029's row deleted; D005's row reads "written twice". Any count the file
  keeps of open items follows.
- `skein/docs/development/debt/D030-forge-connection-harness-tests.md`: "After D029, so the moved tests use the fake's
  own server." becomes "The moved tests use the fake's own server."
- `skein/docs/development/debt/D005-civil-calendars.md`: the machine world's `imf` is gone. Drop
  `skein/tests/forge/machine/src/github.rs` from **Where**, the bullet that names it, and the sentence "Then the world's
  `imf` uses skein-http's writer."; the title and "Three copies" say two.
- `skein/docs/development/debt/reviews/` is history: it stays.
- **If a search above finds code,** it is a face's leftover in a task that merged: stop and ask, with the lines. temper's
  `temper/crates/temper-fake/src/tests/github.rs` calls the service directly and speaks no HTTP: it is outside the
  **Measure** and stays, as do temper's own fixtures.

## Steps

1. The four searches. Paste their output in the report.
2. The edits above. `git diff --stat` against the base lists only the five files.
3. Citations. Nothing here is read by a test or built. Commit and push.

## Report

The common note's items; the searches' output; net lines for this task, and `git diff --shortstat` of the whole part if
the trunk's history lets you read it.
