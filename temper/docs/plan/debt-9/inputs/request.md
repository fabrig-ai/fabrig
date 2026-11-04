# Request: pay debt items, ninth round

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth: each tree's
`docs/development/debt/`). Three independent parts. Each pays its items, so that each item's
**Paid when** holds, and nothing else.

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the command
  reads 0 (or the item's target) after;
- **deletes the item's file and its row** in that tree's `docs/development/debt/README.md`, in the
  same branch;
- updates any status entry the item names as closed by it, and any citation the deletion breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something else
  is exempted, not changed. Never pin an exact HEAD: fab merges the trunk in at each run's start.

**Behaviour stays unchanged, except where part 2 says otherwise.** If a test's visible events or
outcome change, stop and ask. Don't retime a test or rewrite an assertion to make a refactor pass.
Stored formats and breaking public interfaces stop and ask too; additions are fine.

**A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test
go red, restore it, and report what was broken and what failed.

**Keep new code in proportion.** Size caps in tasks are the cut's estimates, not user budgets. Compact
first. If a cap still can't be met without losing wanted behaviour, an answer call may raise it by up
to 25%, stating the measured size and why. Beyond that, ask. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.**

## 1. skein D016: the socket, TLS and HTTP transport is written four times

`skein/docs/development/debt/D016-transport-four-copies.md`. One concrete transport is used by the four
components, and each sends TLS `close_notify` on a graceful close. That close is the one behaviour
this item says changes: pin it with a test.

## 2. Two follow-ups the user decided on 2026-11-04

These are debt-7's plan questions `wrong-secret-kind` and `github-media-type`
(`temper/docs/plan/debt-7/plan.toml` on main, if still there).

- **temper's wrong-kind secret.** A restored secret of the wrong kind stops jig's engine, pinned by
  `a_restored_sources_wrong_kind_stops_the_engine`. By reading, temper's `Corrupt` event finds no owner,
  the accounts child is aborted and the engine runs on. First reproduce it in temper's lowest world
  that shows it. If the design says the engine stops, make temper stop, with a test that fails today:
  this is a bug fix, so the behaviour change is allowed here. If the design is silent or says otherwise,
  stop and ask.
- **The fake GitHub's media type.** Remove `Server`'s charset switch: github.com's spelling, with
  `; charset=utf-8`, everywhere. The one assertion in `skein/tests/fake-peers/tests/forge/github.rs` that
  reads bare `text/html` moves with it. The user approved that assertion moving; no other does.

## 3. smith: four small items

`smith/docs/development/debt/D005-block-walks.md`, `D006-backend-two-effect-paths.md`,
`D008-one-program-map.md` and `D013-names-left.md`, one task each.
