+++
id = "reference-widening"
part = "drafter-stock"
kind = "docs"
needs = []
files = [
  "temper/docs/examples/reference/README.md",
  "temper/crates/temper-docs/src/tests/rules.rs",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "medium"
minutes = 20
+++

# The reference's widening leaves out the drafter

## Goal

With the drafter's charter 12, the stock's delegation holds fifteen executors of
the box's sixteen places, and the reference's charters 7 and 8 would make
seventeen. The reference's work starts no design session, so its widening takes
charter 12 out of the delegation it applies, and the stock's other fourteen and
charters 7 and 8 fill the sixteen (`temper/docs/design/protocol/configuration.md`,
13.9, *The rules and policy 1*). Today's recipe keeps every executor it reads.
This builds on nothing: removing a kind that isn't there changes nothing.

Acceptance:

1. In `temper/docs/examples/reference/README.md`, the recipe's `reference-authority.jq`
   (about line 64): `widen`'s kinds line becomes
   `| .delegation.kinds |= (. + $required[0].kinds | unique | map(select(. != {"charter": 12})))`,
   and the line after the recipe ("Expect: widened rules and policy 1 in `FILE`,
   retaining executors. ...") says the widening adds designer charter 7 and takes
   out the drafter's charter 12, which the reference's work never starts, keeping
   each kind once. The JSON block `reference-authority.json` doesn't change: the
   stock's tests read it (`temper-service/src/tests/stock.rs`,
   `reference_requirements`).
2. A row in the existing test `the_reference_readme_grants_its_tools_and_note_scopes`
   (`temper/crates/temper-docs/src/tests/rules.rs`, about line 240), in its
   `command` list: `map(select(. != {"charter": 12}))`. Its `instruction` list
   keeps passing, `designer charter 7` among it.
3. The status: delete the bullet `**Not yet:** 13.9: the reference's widening ...`
   in `temper/docs/design/protocol/configuration.status.md` (`notes/common.md`,
   *Status files*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.9, *The rules and policy 1*;
  13.4, the row *the rules*.
- `temper/docs/plan/ui-drafter/notes/common.md`.

## Owns and may change

- Owns: the README's recipe line and its *Expect* sentence; the one row; the one
  status bullet.
- Not yours: the stock and its tests (`drafter-stock`, in parallel), including
  `stock.rs`'s `widen_reference` helper, which widens without the recipe's jq.
- If `jq` is on the machine, check the line once on a scratch file outside the
  repository (`{"delegation":{"kinds":[{"charter":11},{"charter":12}]}}` with
  `$required[0].kinds` `[{"charter":7}]` gives charters 7 and 11) and delete the
  scratch; if it isn't, say so.

## Steps

1. Read the inputs and the README's recipe (about lines 55 to 85).
2. Write the row, then the README; break it once (the old line) and see the row
   fail for it; restore.
3. Narrow test: `heavy cargo nextest run -p temper-docs --lib -E
   'test(=tests::rules::the_reference_readme_grants_its_tools_and_note_scopes)'`;
   then `heavy cargo nextest run -p temper-docs`.
4. `cargo fmt`; `heavy cargo clippy -p temper-docs --all-targets -- -D warnings`.
5. Commit (why, in the message) and push.

## Report

As the implementer's text says, and: the row broken and the failure seen; the jq
check's output, or that jq wasn't there.
