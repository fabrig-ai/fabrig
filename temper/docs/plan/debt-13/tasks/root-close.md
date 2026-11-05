+++
id = "root-close"
part = "root-close"
kind = "docs"
needs = ["root-admission", "root-timers", "root-now"]
files = ["jig/docs/development/debt/D004-ops-root-jig-wiring.md", "jig/docs/design/domain/root.status.md", "jig/docs/design/examples.status.md"]
effort = "medium"
minutes = 25
+++

# D004 says what's left

## Goal

The plan paid what both roots copied word for word, and D004 is size L: the rest needs a decision. So the item stays
open, rewritten to say what's left and measured from here, and the status files stop naming what's built.

Acceptance:

- `D004`'s **Measure** gives this trunk's count, and the same command reads at most 3,640 (3,836 at this plan's base).
- The item's **Paid when** names only what's left, and each entry under `## What` is true of this trunk: check each by
  `rg` or by reading, and write what the check found.
- `root.status.md`'s **Not yet** clauses for 3 and 5 are gone, and its **Departs** keeps `batch` and drops `words`:
  `rg -n 'jig_protocol_tools::words' jig/examples/ops/domain/src` finds nothing. The sentence stays singular and true.
- `examples.status.md`'s **Departs** no longer names `admission.rs` or `Class`. It keeps the delegate JSON and the alert
  words, which stay.
- `python3 -I -B tools/citations/citations.py --all` reads 0, and `python3 -I -B tools/debt/debt.py jig` exits 0.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/inputs/request.md`, 2
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/domain/root.md`, 3, 5, 11 and 12
- `DESIGN.md`, 2

## Owns and may change

- **Owns:** `jig/docs/development/debt/D004-ops-root-jig-wiring.md`.
- **May change, and only so:** the two status files, as the acceptance says, each with today's date from `date -u`.
- **Not to touch:** any code. If a measure is over its cap, report the number: don't change code here.
- **Options:** if an entry below proves untrue on this trunk, it goes, and the report says why.

## Steps

1. Run the measures: the item's, `wc -l` of the test root and of `jig-test-service/src/tools.rs`.
2. Rewrite the item. Keep its ID, kind, `Seen` and `Where`. Add `Checked:` with today's date and the trunk's short
   hash. **Value** stays high; **Size** is yours to reassess, M or L. What's left, each checked:
   - The test root isn't in `ops`'s shape: `lib.rs` and `routes.rs`, `domain.now` beside the journal, released events run
     inside `release`, and `store_answers`, `busy` and `public_reads` kept between steps. Rebuilding it changes the
     order of release and of the backlog, so it needs a design decision on a store-answer queue and public reads, which
     `ops` lacks. Say it's the user's to decide.
   - Restart: `ops`'s `restart.rs` and the test root differ in where loads wait and when the journal is applied.
   - `Now::RunPrepared`, `Now::RelayBusy`, `Now::DropCall`, `Held::Relay` and the fleet's `Held` arms: the roots differ,
     and aligning them moves when an event runs.
   - The test root builds `LinkDown::Message` itself: the charter crate holds `message`, the test root doesn't link it,
     and `jig/tests/boundary/src/tools.rs` keeps `host` out of `jig-protocol-tools`.
   - The `document` loop in `client_documents.rs`: pulling `ApplicationDelivery` out of the queue can move it among held
     outputs.
   - `ops`'s fixed host refusals in `link.rs`'s `host_out`, which aren't copies, only a template's weight.
   - Temper's copies, which each task of this plan names under `## Left`: read their reports, or `rg`:
     `temper-engine-domain/src/link.rs`, `admission.rs`, `decision.rs`, `temper/tests/system/src/composed/link.rs`,
     `temper-service/src/assignment.rs`.
   - The review's aim of about 2,000 lines isn't reachable by removing copies: most of what's left in `route.rs`
     (`core_ask`, `effect_ask`, `connector_out`) is `ops`'s own routing. Say what the measure reads and that.
3. The status files, by the acceptance.
4. The citations and the debt index, commit, push.

## Report

What changed, the commit, the measures before and after, each entry checked and what it found, and anything dropped.
