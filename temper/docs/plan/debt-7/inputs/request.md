# Request: pay debt items, seventh round

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
  is exempted, not changed.

**Behaviour stays unchanged.** If a test's visible events or outcome change, stop and ask. Don't
retime a test or rewrite its assertion to make a refactor pass: debt-5 found that a "timing" change
split an atomic commit in two. Budgets, stored formats and breaking public interfaces stop and ask
too; additions to a public interface are fine.

**A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test
go red, restore it, and report what was broken and what failed.

**Keep new code in proportion.** These items exist to remove duplication: each part should end
with fewer lines than it started. A task's size cap is a limit: measure it, and stop and ask if it's
passed.

**All parts must be in the plan when the cut ends.**

## 1. jig D005 and temper D001: the reference service's copied parts

`jig/docs/development/debt/D005-accounts-host-copied.md` and
`temper/docs/development/debt/D001-reference-service-copy.md`. jig ships the parts temper copies as
product code that `jig-test-service` and temper's service both compose. The user has approved this
direction (the debt backfill's decisions, 2026-11-03). It changes `jig/docs/design/protocol/service.md`
section 7, which says the service is written so that it can be copied: jig's designer rewrites that.
The item is size L: if the whole doesn't fit one plan, cut the parts that do, starting with the
accounts host, and rewrite both items to say what's left. Keep temper's hidden `Debug` for
credential-holding types.

## 2. skein D029: four HTTP faces for the fake GitHub

`skein/docs/development/debt/D029-fake-github-http-faces.md`. Every test that serves the fake GitHub
over HTTP drives `skein_fake_forge_github::server`; the fixture App, accounts and repositories are
built once, in the fake.

## 3. smith: four small items

`smith/docs/development/debt/D009-startup-writers.md`, `D014-unused-dependencies.md`,
`D016-retyped-primitives.md` and `D019-channel-component-twins.md`, one task each.
