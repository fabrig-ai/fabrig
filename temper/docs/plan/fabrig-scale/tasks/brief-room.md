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

## Rebase (2026-11-04, step merge, foreman fabrig-scale/brief-room/foreman-1)

resolve the three merge conflicts while preserving whole-spec brief coverage and integration’s status updates.
Rebase `8b46110efc` onto the landing’s current integration tip.
- `temper/crates/temper-service/src/tests/stock_brief.rs` overlaps `046211ac6f` (“Derive B0 carriers and align init standard admission assertions”). Keep host/room/offer/T pins `400_135`, `822_476`, `200_068`, `16_803_810`; retain this task’s six exact charter measurements, tool-mask assertion, and `the_stock_brief_with_a_whole_spec_fits_every_charter_whole`. Resolve `measure_turn` to this task’s unconditional first-turn assertion.
- Preserve the derived room `362_900` and standard brief `360_299`, including the whole 131,072-byte spec and framing/margin checks.
- `temper/docs/design/domain/agent.status.md` overlaps main’s `4cec6f4389`, carried through `b2775ed115`. Preserve the expanded explanation of unavailable project-policy reads and the verdict follow-up gap; remove this task’s obsolete 4.3/4.6 host-room/control-family entry.
- `temper/docs/design/protocol/configuration.status.md` overlaps `8baca21f39`. Keep its November 4 date and removal of the obsolete init/default-sizing gap, then remove “its tools leave out the control family,” from the remaining stock entry. Preserve the separate host-family enforcement gap and other status text.
Keep the automatic merges in `limits.rs` and `tests/stock.rs`, preserving integration’s unrelated changes and this task’s limits and pins.
Run the spec’s focused stock tests first, then its remaining acceptance checks, through `heavy`.
