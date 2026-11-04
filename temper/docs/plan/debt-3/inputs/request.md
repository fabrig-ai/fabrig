# Request: pay four debt items, one per tree (third round)

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth:
each tree's `docs/development/debt/`). Four independent parts (jig's pays two linked items), one per tree, sized S to M. Each
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

**All four parts must be in the plan when the cut ends.** debt-2's smith part was left to a designer
still in flight when its cut ended, and the plan landed without it. If a part needs design, finish
that design inside the cut, or ask a question and leave the part in the plan as waiting on it.

## 1. smith D002: three owners still hold lifecycles in flags and `Option`s

`smith/docs/development/debt/D002-lifecycles-in-flags.md`. `HostIo`, the agent service's `Stages`
and the local `Domain`'s delivery save each hold their lifecycle as a state enum with its own
transitions and `follow` (`skein/docs/foundation/programming-model.md`, 5.4). One task per owner is
fine. Behaviour and stored formats unchanged.

## 2. skein D001: whole-buffer JSON reads are hand-driven

`skein/docs/development/debt/D001-json-whole-buffer-reads.md`. skein-json gets one slice-fed driver
of its tokenizer; `parse` and `validate` both run on it; no step crate outside skein-json drives a
tokenizer over a whole buffer by hand; `skein-oauth`'s `Json` is built on `skein_json::parse` or
the driver. About 25 copies in three styles go.

## 3. temper D002: three crates pump the JSON tokenizer by hand

`temper/docs/development/debt/D002-json-pumps.md`. Each of the three calls `skein_json::parse` (it
exists today, so this part does not wait on part 2); the item's measure reads 0.

## 4. jig D022 and D023: callers of what skein D046 now gives

`jig/docs/development/debt/D022-ui-limits-literals.md` and
`jig/docs/development/debt/D023-view-tests-text-bytes.md`. skein-ui's new constructor and its exposed
node accounting landed in debt-2. jig's `Limits` literals move onto the constructor, and the
view's tests stop re-implementing the accounting.
