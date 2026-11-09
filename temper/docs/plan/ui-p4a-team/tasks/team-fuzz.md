+++
id = "team-fuzz"
part = "proof"
kind = "feature"
needs = ["team-surface"]
files = [
  "temper/tests/client/tests/suite/fuzzy/team.rs",
  "temper/tests/client/tests/suite/main.rs",
]
effort = "medium"
minutes = 70
+++

# Team's parts fuzzed against section 12's rules

## Goal

One fuzz test that holds Team's model to `temper/docs/design/draft/surfaces.md`,
section 12's rules, as `temper/docs/design/client/README.md`, 3.5, *Team*,
*Its parts*, makes them exact: over random project documents, readers and
waits, `pages::team::gather` places each person once and `parts` shows a
part exactly when its rule holds, and a move of the members' roles never
hides a person. Pure functions alone: no world, no root, so it stays well
under the fuzzy suite's budget.

Acceptance:

1. A new `temper/tests/client/tests/suite/fuzzy/team.rs`, `mod team;` in
   `main.rs`'s `mod fuzzy` block beside `client` and `public_address`,
   holding one test, `team_parts_follow_their_rules`, over seeds 1 to 256,
   a failure naming its seed, as `fuzzy/public_address.rs` names it.
2. **Its inputs, per seed** (skein's `Rng::new(seed)`, as
   `public_address.rs` uses it): a `c::Project`, `jig_client_kit::stories::project()`
   with its `members` replaced by 0 to 12 random members, each a party
   from 1 to 6 (so that repeats occur), a random `PartyKind`, a random
   `HeldRole` (`Policy { role }` among them), a name empty or not and a
   login; a random `cut`; a reader `you` from 1 to 7 (so that sometimes no
   member is the reader); `yours`, `None` or random `Waits`; `others`, 0 to
   8 pairs of a party from 1 to 7 and random `Waits`.
3. **Its oracles,** on `gather(&project, you, yours, &others)` and `parts`:
   - each party with a `Person` member appears in `people` exactly once,
     with its first such member's role and name (the login where the name
     is empty); no `Service`'s party appears unless a `Person` member has
     it too; nothing else appears;
   - the reader is first and `you` while a `Person` member has their
     party, and absent otherwise; the others follow in their first
     appearance's order;
   - each person's `waits` is `yours` for the reader, the first pair
     naming them for another, else `None`;
   - `cut` is the document's; `triager` and `standing_roles` are `None`;
   - `parts.people` exactly while `people` isn't empty (no part drawn
     empty); `parts.triager` false;
   - **a move never hides a person:** the same document with each
     member's role drawn again lists the same parties in the same order.
4. Its CPU time, in the report, under 0.3 s
   (`heavy cargo nextest run -p temper-client-world --test suite --profile
   fuzzy -E 'test(=fuzzy::team::team_parts_follow_their_rules)'`).

## Inputs

- `temper/docs/plan/ui-p4a-team/notes/common.md`;
  `temper/docs/plan/ui-p4a-team/notes/contracts.md`, T2 (`gather`'s and
  `parts`' rules).
- `temper/docs/design/client/README.md`, 3.5, *Team*, *People* and *Its
  parts*; `temper/docs/design/draft/surfaces.md`, 12 and 14 (*A fuzz
  test of section 12's rules*).
- The models: `temper/tests/client/tests/suite/fuzzy/public_address.rs`
  (seeds, `Rng`, the failure's words); jig's
  `jig/testing/jig-client-kit/src/stories.rs`, `project()`.

## Owns and may change

Owns the two files in the header; in `main.rs` the one `mod team;` line
alone. Not `fuzzy/rail_and_work.rs` or `fuzzy/density.rs` (other plans'),
not `src/fixtures.rs`: the test's helpers stay in its file.

Options, each decided:

- **`stories::project()` isn't reachable from the suite:** build the
  document in the test's file from `c::Project`'s fields, the members
  random and the rest as small as the type allows.
- **The oracle needs a second implementation of `gather`:** write it in
  the test as the plainest loop over the members, not by calling
  `gather`'s helpers.

## Steps

1. Read the Inputs. The test (1 to 3); bite: in a scratch copy of
   `pages/team.rs`, not committed, let `gather` keep a service, see the
   test fail with its seed, restore; and once more, list the reader where
   the document puts them.
2. Its run (4) with its count and CPU time; `heavy cargo nextest run -p
   temper-client-world --test suite` (focused: the fuzzy test is not in
   it); clippy on `temper-client-world`; then as `notes/common.md` says
   before the push. Commit, saying that Team's parts now hold section 12's
   rules under random members and moves, and push.

## Report

The implementer's report: the inputs' ranges, each oracle, the bites with
their failures and seeds, the run's count and CPU time, and what is left.
