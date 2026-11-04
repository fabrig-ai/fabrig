# Request: pay four debt items, one per tree (fourth round)

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth:
each tree's `docs/development/debt/`). Four independent parts (temper's pays two linked items), one per tree, sized S to M. Each
part pays its item, so that the item's **Paid when** holds, and nothing else. A part
that needs a design change, a breaking public interface change or a stored-format change stops
and asks (additions to a public interface are fine).

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the
  command reads 0 (or the item's target) after;
- keeps behaviour unchanged unless the item says otherwise; tests that pinned removed code go
  with it, and the rest pass unchanged;
- **deletes the item's file and its row** in that tree's `docs/development/debt/README.md`, in
  the same branch (the README's rule: git keeps the history);
- updates any status entry the item names as closed by it, and any citation the deletion breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something
  else (for example a test that checks an old field is refused) is exempted, not changed.

**All four parts must be in the plan when the cut ends.** debt-2 lost a part left to a designer
still in flight when its cut ended, and the plan landed without it. If a part needs design, finish
that design inside the cut, or ask a question and leave the part in the plan as waiting on it.

Keep new code in proportion. debt-3's state machines added more lines than they removed; a state
enum replaces its flags, and the transitions it needs, not a layer around them.

## 1. skein D017: skein-kv's store keeps its lifecycle in flags

`skein/docs/development/debt/D017-kv-store-states.md`. `Store`'s phases become enum variants that
hold their data (`skein/docs/foundation/programming-model.md`, 5.4): matches are per state and event
with no catch-all arm, and `store.rs` has no module-wide `#![expect]`. tests/kv's crash and golden
tests pass unchanged, and the on-disk format is unchanged.

## 2. jig D028: the scripted engine re-implements the core's rules

`jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`. Each rule the scripted engine
copies (inbox order, phase classification, requester choice) is taken from jig's own code, or is
held to the engine by a shared story on the scripted and joint fronts. Prefer removing a copy to
pinning it.

## 3. smith D010: the benchmark runner knows each agent

`smith/docs/development/debt/D010-bench-runner-knows-agents.md`. `run.rs` and `process.rs` name no
agent: observation, archiving and source selection are methods of the agent adapters, and
`Capture::Codex` and `ProcessResult.observer` go.

## 4. temper D004 and D003: hand-written forms beside lib's helpers

`temper/docs/development/debt/D004-env-with.md` and
`temper/docs/development/debt/D003-widening-try-from.md`. Mechanical, across about 20 crates: both
measures read 0, with no behaviour change. Generated code stays as it is.
