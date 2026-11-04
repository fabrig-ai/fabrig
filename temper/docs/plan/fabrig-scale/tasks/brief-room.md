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

## Answers

2026-11-04. Q: integration's `stock/chat.txt` grew 24 bytes, so charter 1's room is 362,876, not the
Rebase section's 362,900: rederive and lower the brief to 360,275, or have the owner shorten the
instructions?

Rederive. `brief_bytes` is *derived* (Goal; contracts K8), and the Rebase section's 362,900 and
360,299 were that derivation's result on an older base, not figures to hold: this answer replaces
them, and every other instruction of that section stands. `chat.txt` stays as integration has it.

- Keep the derivation: min(395,520, room - 1,472 - 1,129), at 1,129 of margin, with the task
  section 131,584 (the whole 131,072-byte spec and 512). At base `83b60d6e20` that is
  min(395,520, 362,876 - 1,472 - 1,129) = 360,275, and charter 1's non-brief bytes are 37,259.
- Set the figures from what the tests measure on the base you run on: `limits.rs`'s `brief_bytes`
  and its comment; `tests/limits.rs` (`brief_bytes`, `result_bytes`); `stock_brief.rs`'s room and
  all six charters' pairs (only charter 1's first pair has been measured; the others ran no
  assertion, so take them from the test, and a charter holding `chat.txt` moves by the same 24).
  The same applies if the base has moved again: follow the derivation, without asking.
- Only the aggregate falls: the reviews section gives up the bytes, the task and pull sections
  stay whole. Stop and ask only as "Owns and may change" says, when a whole spec no longer fits.

## Rebase (2026-11-04, step merge, foreman fabrig-scale/brief-room/foreman-3)

resolve the three merge conflicts and rederive the brief limit on the integration base.
Rebase `8b46110efc` onto the landing’s current integration tip (observed `e3a27e166f`).
- `temper/crates/temper-service/src/tests/stock_brief.rs` conflicts with `046211ac6f`. Keep host/room/offer/T pins `400_135`, `822_476`, `200_068`, `16_803_810`, this task’s tool-mask assertion, six exact charter measurements, and `the_stock_brief_with_a_whole_spec_fits_every_charter_whole`. Keep `measure_turn`’s unconditional first-turn assertion.
- Follow the Answers section when refreshing measurements: integration’s `3721f23e5c` grew `stock/chat.txt` by 24 bytes. Leave that file intact. Charter 1’s non-brief bytes become `37_259`, its room `362_876`, and the derived brief `360_275` on the observed base; verify all six pairs through the focused tests.
- Update `limits.rs`’s derivation comment and aggregate brief limit, `tests/limits.rs`’s brief/result pins, and `stock_brief.rs`’s room assertions together. Derive `min(395_520, room - 1_472 - 1_129)` again if the base moves.
- Preserve task, pull and reviews budgets of `131_584`, the whole `131_072`-byte spec, and the framing/margin checks. Reduce only the gathered reviews contribution when the aggregate falls.
- `temper/docs/design/domain/agent.status.md` conflicts with `4cec6f4389`, carried through `b2775ed115`. Preserve integration’s expanded project-policy-read explanation and verdict-follow-up gap; remove the obsolete 4.3/4.6 host-room/control-family entry.
- `temper/docs/design/protocol/configuration.status.md` conflicts with `8baca21f39`. Preserve November 4’s date, removal of the init/default-sizing gap, the separate host-family enforcement gap, and other status text. Remove “its tools leave out the control family,” from the stock entry.
- Preserve automatic merges in `limits.rs` and `tests/stock.rs`, including integration’s socket admission and fixture changes alongside this task’s limits and control-family pins.
Run the spec’s focused stock tests first through `heavy`, then its remaining acceptance checks.
