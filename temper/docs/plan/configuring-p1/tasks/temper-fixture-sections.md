+++
id = "temper-fixture-sections"
part = "documents-jig"
kind = "migration"
needs = ["jig-documents-config"]
files = ["temper/crates/temper/tests/fixtures/cluster/engine.json", "temper/crates/temper/tests/support/github/engine.json", "temper/tests/real-loop/src/landing.rs"]
effort = "medium"
minutes = 30
+++

# temper: the engine documents that override `brief.sections` hold a charter's documents

## Goal

Once `jig-documents-config` lands, jig's configuration start refuses an engine document whose
`brief.sections` cannot hold the core's own six sections and `tasks.charter_documents`
(2 in Small), naming both (jig's `jig/docs/design/domain/engine.md`, 17.9;
`jig/docs/design/protocol/configuration.md`, section 6). temper's start runs that check over
the engine document (`temper-service/src/startup.rs`, `config::engine`). Three temper test
documents override `brief.sections` to 4, so they would no longer start: raise each to 8, the
profile's. This is a follow-up of `jig-documents-config`, whose spec named only jig's fixtures.

Acceptance:

- `"brief.sections": 8` in `temper/crates/temper/tests/fixtures/cluster/engine.json` and
  `temper/crates/temper/tests/support/github/engine.json`, and `("brief.sections", 8)` in
  `temper/tests/real-loop/src/landing.rs`'s overrides; nothing else of them changes, unless a
  memory or envelope figure of the same document then fails its start, which is raised by the
  least that starts it and listed with its old and new value.
- Each starts again: `heavy cargo nextest run -p temper-real-loop-world` (the real loop's focused
  tests: those using `landing.rs`'s document), and the `temper` crate's tests that load the two
  JSON documents (`grep` for `cluster/engine.json` and `github/engine.json` under
  `temper/crates/temper/tests/`; run their focused, non-ignored tests by name, never a live or
  ignored one), with their counts in the report. A test of these that cannot run in the focused
  profile is named in the report as not run, with why.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md`.
- `jig/docs/design/domain/engine.md`, 17.9 (a start whose brief cannot hold the documents is
  refused); `jig/docs/design/protocol/configuration.md`, section 6 (8 sections in both profiles).

## Owns and may change

- Owns the three files in the header, each only in its overrides.
- Not yours: `root.brief.sections` (temper's root, `root-document-binding`'s), jig's crates.
- Pre-approved: if another temper document or override list the start now refuses for the same
  reason turns up (a `grep` for `"brief.sections"` and `("brief.sections"` under `temper/`),
  raise it the same way and list it; `temper/tests/one-process/src/fixtures.rs` and
  `temper/tests/real-loop/src/green_light.rs` already set 8.

## Steps

1. The three overrides; the `grep` for any other.
2. The tests above, one command at a time.
3. `python3 -I -B temper/scripts/old_forms.py`; the citations check.
4. Commit (why: temper's test documents hold the core's sections and a charter's documents);
   push your branch.

## Report

As the implementer's text says, and also: each document raised, any figure moved, and the tests
run with their counts.
