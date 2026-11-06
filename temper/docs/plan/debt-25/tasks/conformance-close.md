+++
id = "conformance-close"
part = "conformance"
kind = "docs"
needs = ["core-answers"]
files = ["jig/testing/jig-conformance/README.md", "jig/docs/development/debt/D030-conformance-not-drop-in.md"]
effort = "medium"
minutes = 15
+++

# D030 says what's left

## Goal

The trait is 19 functions, and it stops there, for the reasons in `notes/contracts.md`. An application still writes `deliver`, `poll`, `inbound`, `cold` and `observed`, which mix the core's arms with its own, so the **Paid when** doesn't hold. The item stays open, saying that. It doesn't read as paid.

Acceptance:

- `jig/testing/jig-conformance/README.md`'s list of what an application gives matches the trait: no restart rows, no recovery function, no heap bound.
- `jig/docs/development/debt/D030-conformance-not-drop-in.md` is rewritten as it stands:
  - **Where** names `jig/testing/jig-conformance/src/harness.rs`, `jig/testing/jig-core-kit/src/conformance.rs` and the three other adopters' paths from `notes/contracts.md`.
  - **Checked** gets today's date and your base. **Size** is what's left.
  - **Measure** keeps its command, with "(19 at this writing)".
  - **Paid when** stays as it is.
  - **What** lists what's left, with the measured impl sizes of the four adopters: what each of the five mixed functions still answers.
  - **What stands in the way** gives `notes/contracts.md`'s four reasons, and says a trait must remain, so the Measure has no target of 0.
- `python3 -I -B tools/citations/citations.py --all` reads 0 after you commit.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/contracts.md`
- `jig/docs/development/debt/D030-conformance-not-drop-in.md`
- `jig/docs/design/domain/testing.status.md`, 5

## Owns and may change

- **Owns:** the files in the header. The design's status already says this, written by this plan's cut.

## Steps

1. Re-measure, then write both. Commit, run the citations tool and push.

## Report

The common note's items; the Measure; across D030's tasks, the test lines at this plan's base and now.
