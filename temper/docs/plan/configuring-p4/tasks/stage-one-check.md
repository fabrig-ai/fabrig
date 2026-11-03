+++
id = "stage-one-check"
part = "golden"
kind = "feature"
needs = ["stage-one-surface"]
files = ["temper/tools/stage-one/src/check.rs", "temper/tools/stage-one/src/tests/check.rs", "temper/tools/stage-one/src/tests/fixtures/profile/**"]
effort = "high"
minutes = 150
+++

## Goal

Build the check by code (`temper/docs/design/domain/testing.md`, 6.5.1 and 6.5.4): bind the
golden's roles to the evidence's settings, give each obligation its outcome, find what fails
beyond them, and print the report; `temper-stage-one check EVIDENCE [--json]`.

Acceptance:

- Evidence made from the reference's profile, as if the agent had proposed it, passes: every
  setting obligation met, judged or unchanged, every box obligation confirmed, nothing beyond.
- The same evidence with the profile's added charters renumbered passes, and the binding says the
  new numbers; each mutation below fails with the outcome named.
- `heavy cargo nextest run -p temper-stage-one` passes; clippy clean as the surface's task says.

## Inputs

- `temper/docs/design/domain/testing.md`, 6.5 (6.5.1, the roles and the tests; 6.5.4, the check).
- `temper/docs/design/client/tools.md`, 5.3, "Settings FILE is JSON" (a landing rule's pattern
  as hex segments).
- `temper/docs/plan/configuring-p4/notes/common.md`; `temper/docs/plan/configuring-p4/notes/contracts.md`
  (K2, K3, K4, K5).
- On the trunk from `stage-one-surface`: `golden.rs` (`reference()`, `profile()`, the types),
  `evidence.rs` (`read`), the stub `check.rs`, `src/tests/check.rs`.

## Owns and may change

- Owns: `src/check.rs` (the stub replaced whole: K4's types, `check`, `main`), `src/tests/check.rs`,
  `src/tests/fixtures/profile/**` (new).
- `src/golden.rs` and `src/evidence.rs`: none of their public names or shapes; a helper method on
  their types (a value's member by path, a setting key's display) may be added there if the check
  needs it, said in the report.
- Options, each pre-approved:
  - If K4's `Outcome` needs one more case to say what 6.5.4 says (a member present but of the
    wrong JSON type), add it and say so; never fewer.
  - If a test helper that builds the profile's evidence would serve `stage-one-judge` too, make it
    `pub(crate)` in `src/tests/check.rs` (it will).

## Steps

1. **Reading a subject.** A role binds to a setting key (below); a `{"setting": ...}` subject is
   that key, `"project": "box"` the evidence's project. Its value is `current`'s item for the key;
   none, the obligation is `Missing`. A member is a dotted path, a numeric word indexing a list,
   `""` the value whole; absent, `Missing`.
2. **The expected value:** the obligation's `value`, or the profile's item for the role's kind and
   `profile` number at the member. Then substituted: each `{"role": STRING}` by the role's bound
   number (unbound: the obligation is `Missing`); each string equal to the placeholder forge by
   the box's forge and to the placeholder repository by the box's. A landing rule's `pattern`,
   expected and found, is compared decoded: its `segments` and its `last`'s `exact` or `open` from
   hex to text, the placeholder repository's owner and name segments replaced by the box's.
3. **The binding** (6.5.1), in the golden's role order:
   - `{"resource": "goal-branch"}`: the current template whose `resources` hold
     `{"symbol": 1}` (any connector); several, the lowest number.
   - `{"executes": R}`: the charter R's bound template names in `executor.charter`.
   - `{"forge-charter": N}`: the charter number forge charter N's current value holds.
   - `{"number": N}`: that number, when `current` holds it.
   - `{"branch": ...}`: a current landing rule, the deployment's or the box's project's, whose
     decoded pattern's `last` is that exact or open text; several, the lowest number.
   - Null anchors: among the charters `items` sets that no anchor bound, try each assignment of
     the unanchored roles (each role to one charter or none, none twice); keep the one meeting
     the most obligations, ties to the lexicographically lowest numbers in role order.
   Each `Binding` says `Anchor`, `Assignment` or `Unbound`.
4. **The tests** (6.5.1): `equal` after step 2's substitution and decoding; `at-least` on
   integers; `bits` `found & expected == expected` on integers; `contains` each expected element
   equal to some found element; `set` present and not null; `judged` a non-empty string, else
   `Missing`; `unchanged` no item of `items` for the key and the key in `current`, else `Wrong`
   (found the item's edit, expected `"unchanged"`); `only` every current setting of the kind,
   both scopes, binds to a role, else `Wrong` (found the unbound keys).
   Box obligations: a confirmation present and true is `Confirmed`; false or absent, `Lacking`
   with what was seen (absent: `"not confirmed"`). Gap obligations: an alternative all of whose
   words appear case folded in `proposal.txt`, then in `chat.txt`, is `Named`; else `Unnamed`.
5. **Beyond the obligations:** each item of `items` whose key is no bound role's and no
   obligation's subject is an `Extra`, "no obligation asks for it".
6. **The report.** `text()`: the binding a line each (`coordinator: charter 9, by its anchor`),
   then each obligation a line (`ID  OUTCOME[: detail]`, `wrong: found X, expected Y` with
   compact JSON), the extras, and a last line `passed` or `failed: N of M`; `json()` the same as
   one object (`binding`, `verdicts`, `extras`, `passed`). `check::main`: `EVIDENCE [--json]` over
   `run(args) -> Result<bool, String>`; exit 0 passed, 1 failed, 2 with the error on stderr.
7. **Tests** in `src/tests/check.rs`, on `skein-scratch` directories:
   - `profile_evidence(dir, edit)`: writes evidence made from the profile: `items.json` the
     profile's eight items, the forge and repository placeholders replaced by `forge.box:3000`
     and `people/kit` (hex segments too), plus `rules`, `policy` 1 and `project-policy` items
     from `src/tests/fixtures/profile/{rules,policy,project-policy}.json` (hand-written: tools
     1671, the delegation kinds the golden asks for with the profile's numbers, tasks 1024,
     depth 4, in FIELDS' shapes); `current.json` those items plus the stock's that the golden
     reads (template 2, forge charters 1 and 2 at 2 and 4, charters 1, 3 and 4), from
     `src/tests/fixtures/profile/stock.json`; `box.json` with that forge and repository;
     `confirmed.json` every box obligation present; short `proposal.txt` and `chat.txt`;
     `revision.json` `{}`. `edit` mutates the items before writing.
   - `the_profile_as_proposed_passes`; `renumbered_charters_still_pass` (5, 7, 8 to 11, 12, 13,
     with template 4's executor and the delegation kinds following); `a_value_changed_is_wrong`
     (goal rule's `ci` true); `an_added_charter_is_an_extra`; `a_role_removed_is_missing`
     (charter 8 gone: the triage's obligations missing); `the_stock_rule_left_fails_only` (a
     third deployment rule on every branch); `the_small_goal_template_set_is_wrong`;
     `a_box_obligation_lacking_fails`; `a_gap_named_and_unnamed` (a golden of the test's own,
     decoded with `golden::decode`, with one gap: its words in the chat named, absent unnamed);
     `run_exits_by_outcome` (passed, failed, and an unreadable directory's error).
8. `cargo fmt`; old forms; clippy; `heavy cargo nextest run -p temper-stage-one`; commit ("temper-
   stage-one: the check by code") and push.

## Report

As the implementer's text says; and: the binding the profile's evidence got, each obligation
that needed a choice of yours (a member's path, a decoding), and the test count.
