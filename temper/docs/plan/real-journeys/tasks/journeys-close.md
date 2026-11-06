+++
id = "journeys-close"
part = "close"
kind = "docs"
needs = ["add-guard", "real-walk"]
files = ["temper/docs/design/client/journeys.status.md"]
effort = "medium"
minutes = 5
+++

# The status says what stands

## Goal

The redaction is built, so `journeys.status.md`'s **Not yet** goes. **This departs from the implementer's text:** this task edits this one file of `docs/design/`, and only this one.

Acceptance:

- `temper/docs/design/client/journeys.status.md` is dated today (`date -u`) and records only what stands: `No departures.` if nothing does; otherwise each gap the earlier tasks' reports or the code show, by section.
- `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing new.

## Inputs

- `DESIGN.md`, 2.
- `temper/docs/design/client/journeys.md`, 3.6 to 3.8, 4.3, 5.1 and 6.
- `temper/docs/plan/real-journeys/notes/common.md`.

## Owns and may change

Owns `temper/docs/design/client/journeys.status.md`. Nothing else: `temper/docs/design/draft/real-journeys.status.md` keeps what it says, its departures being the user's.

## Steps

Read `add` in `temper/tools/journeys/src/lib.rs` against 5.1 and the tests against 6 by `rg`, enough to say each **Not yet** entry is built. A gap you find goes in the file. No build, no test: `cargo fmt` isn't needed. Commit and push.

## Report

As the implementer's text says.
