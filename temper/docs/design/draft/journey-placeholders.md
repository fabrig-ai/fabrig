# A recording changes only when its screens do

Draft, 2026-11-17, on the user's word. Every branch that records a journey again rewrites its files, even when no screen changed, so any two such branches conflict on each file. On 11-17, four landings conflicted on 64 or 65 recordings each, besides their real files. The recordings under `temper/docs/design/client/tui/journeys/` and `temper/docs/design/client/web/journeys/` embed what varies from run to run: the build's identity (`temper 0.0.0 (COMMIT-dirty)`, in 12 files) and loopback ports (`127.0.0.1:PORT`, the box's and its Forgejo's, in 64 files).

## 1. What happens

- **`add` redacts, then proves:** it replaces each secret with `x`s of its own length, wherever it appears and in every form, and then replays the redacted recording against its live prints (`temper/docs/design/client/web/journey-redaction.md`; `temper/docs/design/client/web/journeys.md`, 4.3; `temper/crates/temper-client-shell-web/src/journey/redact.rs`; `temper/crates/temper-client-shell-tui/src/journey/redact.rs`).
- **Nothing replaces a value that isn't a secret,** so the build, the ports and anything else a run draws stay in the recording, its marks and its prints.

## 2. Wanted

- **Values that vary are normalised:** at `add`, and so at `again` (8.1), every value that varies between two recordings of the same screens and isn't a secret is replaced by a fixed placeholder of its kind. That covers the build's identity (version, commit and dirty mark), every loopback port, and whatever else the recorder finds varying from run to run, such as a wall time or a generated id.
  - Each is sought in every field and form, as redaction's originals are (`temper/docs/design/client/web/journey-redaction.md`), so the recording, its marks and its prints agree.
  - A placeholder is the same bytes whatever the original's length, and a framed field's length follows it.
  - Redaction of secrets stays as it is, and a secret is never a placeholder's.
- **The proof still holds:** the first replay's proof (4.3) and the replay's check (4.2) pass on a normalised recording.
- **The same screens make the same bytes:** two recordings of one journey, made with different builds, ports and times, are byte-identical, prints included.
- **Normalised once, not recorded again:** the committed recordings are normalised by the same code, and their prints made again from them (5). No journey is recorded again, and no box is run for it.
- Revise `temper/docs/design/client/web/journey-redaction.md`, `journeys.md`, 4.3 and 8.1, and `temper/docs/design/client/tui/journeys.md`, with their status files.

## 3. Out

- What a journey shows, and how its screens look.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The journeys tool's tests (`temper/crates/temper-client-journeys-tool/src/tests.rs`): a recording with two builds' identities and two sets of ports normalises to the same bytes; each placeholder is found in a recording's fields, marks and prints; a secret is still `x`s; the proof passes on the normalised recording.
- The web and TUI redaction tests beside `redact.rs`: framed fields keep valid lengths when a placeholder's length differs from the original's.
- The committed journeys replay and print unchanged after their one normalisation, by name.

Run only the new and affected tests, by name.
