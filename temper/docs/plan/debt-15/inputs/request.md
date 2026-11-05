# Request: carry out the user's answers

From fabrig/debt, 2026-11-05 (the user's continuous tech-debt process). The user took every open
recommendation on 2026-11-05. Three independent parts.

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the command
  reads 0 (or the item's target) after;
- deletes any item it pays, in the same branch; updates any status entry the item names as closed by it, and any citation that breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something else
  is exempted, not changed. Never pin an exact HEAD: fab merges the trunk in at each run's start.

**Behaviour stays unchanged, except where part 1 says otherwise.** If a test's visible events or
outcome change, stop and ask. Don't retime a test or rewrite an assertion to make a refactor pass.
Stored formats and breaking public interfaces stop and ask too; additions are fine.

**A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test
go red, restore it, and report what was broken and what failed.

**Keep new code in proportion.** Size caps in tasks are the cut's estimates, not user budgets. Compact
first. If a cap still can't be met without losing wanted behaviour, an answer call may raise it by up
to 25%, stating the measured size and why. Beyond that, ask. Budgets and timeouts stay the user's.

**All parts must be in the plan when the cut ends.**

## 1. The two tasks held out of debt-9

- **`inputs/held-temper-wrong-kind.md`, answered (a):** a restored secret of the wrong kind stops temper's
  engine, as jig's does. One sentence joins `temper/docs/design/domain/credentials.md`, 5 (find it by name if it
  has moved), and about three lines and a test join temper-service. Reproduce first: the test fails today.
  This behaviour change is the user's decision.
- **`inputs/held-github-media-type.md`, answered (a):** `Server`'s charset switch goes. `page.header(b"content-type")`
  and `conversion.header(b"content-type")` in `skein/tests/fake-peers/tests/forge/github.rs` read the long forms,
  and `the_peers_server_names_a_bare_media_type` is deleted, since `an_app_request_is_answered_whole` pins the one
  spelling. The user approved these three test changes; no other.

Their branches `debt-9/temper-wrong-kind` and `debt-9/github-media-type` on origin are for reading: they stand
on an older main.

## 2. Status changes the user decided

- `temper/docs/development/debt/D001-reference-service-copy.md`, `d001-bound` answered (a) with (d): status `kept`
  with the reason that the overlap left is composition `jig/docs/design/protocol/service.md`, section 11, assigns to
  each application, so a later review doesn't find it again. Take the survey's small moves too
  (`config::clients::invalid`, a link owner lookup, the sizing test), about -25 on its measure. See
  `temper/docs/plan/debt-10/plan.md`, or `git log` if it has gone.
- `hearth/docs/development/debt/D013-records-three-times.md`: status `kept`, reason "the layers' own vocabularies".
- `listener-off-loopback` answered (a): temper's check stands; record the gap as a Departs entry in
  `temper/docs/design/protocol/configuration.status.md`.

## 3. skein D011: the fake GitHub stands on the fake Forgejo

`skein/docs/development/debt/D011-fake-github-on-forgejo-fake.md`. The user approved renaming
`skein-fake-forge-protocol` for the Forgejo fake, in this plan, with temper's uses following. Don't touch
what debt-14 is changing in hearth or smith.
