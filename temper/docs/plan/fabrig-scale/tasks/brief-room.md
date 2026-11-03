+++
id = "brief-room"
part = "room"
kind = "feature"
needs = ["init-standard"]
files = ["temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/tests/stock.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/crates/temper-service/src/tests/limits.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_tool.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 150
+++

# The stock's goal makes plans, and a run's brief holds a whole spec

## Goal

Two things the room at luna's window now allows (contracts K8; `init-standard` set the window):
the stock's tool families gain jig's control family, so a goal's `change` offers its plan's
members (`temper/docs/design/domain/agent.md`, 4.3.1; `protocol/configuration.md`, 13.4, policy
1 and the rules); and temper's `standard` brief carries a whole spec of jig's `standard`, so a
run reads the task it was given (`protocol/configuration.md`, section 7; `agent.md`, 4.1 and
4.3).

Acceptance:

- `temper-service/src/stock.rs`: `CHAT_TOOLS` gains `4` (jig's `ToolFamilies::Control`), beside
  the families it holds when you start (`1 | 2 | 512`, or `1 | 2 | 512 | 1024` if another plan's
  settings family has landed), its doc comment naming Control and why (a goal's `change` names
  its plan's members: `agent.md`, 4.3.1). Nothing else of the stock.
- temper's `standard` (`temper-service/src/limits.rs`, built by `forge-profile`) takes its own
  brief, small's `root.brief` but:
  - `budgets.task` 131,584: a whole spec of jig's `standard`, 131,072, with its contract's and
    requester's 512 bytes, as small's 4,608 is 4,096 and 512 (comment it so);
  - `budgets.pull` and `budgets.reviews` the task's, since they share its read bound (the
    comments beside small's say so); `read_bytes` the task's;
  - `brief_bytes`, and `root.journal.result_bytes` equal to it: the smaller of the three
    sections' sum with small's 768 bytes for other context (395,520) and the room the host part
    leaves the largest stock charter at init's agent, charter 1 with a Change contract and
    control, less 1,472 bytes for 64 sections' titles and framing and 1,129 of margin
    (*derived*: compute it in `stock_brief.rs` as its neighbours compute the room, and set the
    constant to it, a comment naming the derivation);
  - every limit the start's checks tie to these (the forge's `brief_bytes`, at least the root's
    `read_bytes`; a store bound of a journalled brief) the least that passes, listed in the
    report.
- `temper-service/src/tests/stock_brief.rs`: its figures at init's documents (host part, room,
  offer, T, each charter's non-brief host part and what it leaves) are the ones the code now
  derives, each asserted exactly as today; a new test,
  `the_stock_brief_with_a_whole_spec_fits_every_charter_whole`, builds the authorised brief at
  `standard`'s budgets, its task section a 131,072-byte spec, and asserts each stock charter's
  host part with it is within the bound, so fitting cuts nothing.
- The pins of `CHAT_TOOLS`: `src/tests/stock.rs` (near 88-92), `stock_brief.rs` (near 32),
  `temper/tests/engine/domain/tests/suite/forge_routes/goal_tool.rs` (near 219) name the new
  mask; a goal under the stock is offered `change`'s plan form (the engine domain world's
  goal-tool story asserts it, if it asserts the offered form: extend it to say so).
- Status: `agent.status.md`'s entry "4.3, 4.6: the host part's figures ... `CHAT_TOOLS`)."
  removed; `configuration.status.md`'s clause "its tools leave out the control family," removed.
- `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::stock/)'` passes; then
  `-p temper-service`, `-p temper-engine-smith`, the engine domain world's forge routes suite
  (`heavy cargo nextest run -p temper-engine-domain-world -E 'test(/forge_routes::goal_tool/)'`;
  find the package with `cargo metadata` if that name is wrong), and `heavy cargo nextest run -p
  temper --test suite -E 'test(/startup::/)'`.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K6 and K8.
- `temper/docs/design/domain/agent.md`, 4.1 (the brief's admission in `standard`), 4.3 ("The host
  part's budget", "The brief fits the room its charter leaves"), 4.3.1 ("Who may set them"),
  4.6 (the demo's room).
- `temper/docs/design/protocol/configuration.md`, section 7 (`standard`'s brief) and 13.4
  (policy 1, the rules, "Each charter's host families").

## Owns and may change

- The files of the header, for what acceptance names; in `stock.rs`, `CHAT_TOOLS` alone.
- `temper-engine-smith`'s table of each host family's declaration bytes (`src/calls.rs`) is
  read, not changed: control's plan form of `change` is already measured there.
- If a stock charter's room at init's agent cannot hold the whole brief at these budgets (the
  new test fails), stop and ask with the figures; never lower the task section below a whole
  spec.
- `temper/docs/help/` is not yours.

## Steps

1. `CHAT_TOOLS`; its pins; run the stock tests.
2. The brief's budgets in `standard`, the derivation in `stock_brief.rs`, the constant; the new
   test.
3. The packages of acceptance, one at a time.
4. The status lines; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-service --all-targets -- -D warnings`; the citations check; commit and push.

## Report

As the implementer's text asks, with the derived room and `brief_bytes`, and each coupled limit.
