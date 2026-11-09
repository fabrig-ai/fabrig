+++
id = "goal-path"
part = "path"
kind = "feature"
needs = ["goal-wire"]
files = [
  "temper/crates/temper-engine-domain-ways/src/path.rs",
  "temper/crates/temper-engine-domain-ways/src/path/derive.rs",
  "temper/crates/temper-engine-domain-ways/src/path/cards.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/derivation.rs",
  "temper/crates/temper-engine-domain-ways/src/tests/fuzzy.rs",
  "temper/crates/temper-engine-domain-ways/src/tests.rs",
  "temper/crates/temper-service/src/family/ways.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 90
+++

# The path draws "design" and "close-out" where a goal's template says so

## Goal

The one engine-side derivation draws a goal's coordinator as "design" where the
goal's template designs first, and "plan" where it doesn't, and draws a
"close-out" after the goal's landing where the template makes one. Every client
draws what it is given, so How work runs, the ways page, Ask temper's views and
the terminal's text follow with no change of theirs. The root gives both settings
false until `goal-read` reads them.

Acceptance, by `temper/docs/design/domain/forge.md`, 8.13 (the inputs, *A
typical run* steps 1 and 6, *The coordinator's cards*, *Drawn apart*, *No agent's
stop below the line*):

1. **The shapes,** exactly `temper/docs/plan/ui-p3f-design/notes/contracts.md`,
   section 5: `Start::Goal { charter, design, close_out }`, `Step::Design` and
   `Step::CloseOut` after `Step::Branch`, their comments citing 8.13.
2. **"design"** in place of "plan" with `design`: its words, actor, opening and
   card as the note has them, the fact `Its order` after the agent's facts;
   without `design`, the "plan" stop byte for byte as today.
3. **"close-out"** with `close_out`, on the landing branch's segment after a
   goal's alone: right after the merge (by queue) or the turn (by turn), before
   that segment's tiers' "verify" stops and its bar; its words, actor, opening
   and card as the note has them, the fact `Lands` after the agent's facts,
   BRANCH the landing branch. Not needed, no small words over it. Never on a
   goal's own segment, never for a chat.
4. **The service** (`temper/crates/temper-service/src/family/ways.rs`) sends
   `Step::Design` as `w::PathStep::Design` and `Step::CloseOut` as
   `w::PathStep::CloseOut`.
5. **The root's construction,** `temper/crates/temper-engine-domain/src/route/ways.rs`'
   `start`, gives `design: false, close_out: false`; every other reader of
   `Start::Goal` matches `{ charter, .. }` (`route/ways.rs`' `inputs`,
   `route/ways/view/roles.rs`); `route/ways/tests.rs`' expected starts take the
   two fields false. Those lines alone: `goal-value` gives `route/ways.rs`'
   `project_ways` an arm beside you, and `goal-read` fills `start` after you.
6. **Rows,** no new test function:
   - `tests/derivation.rs`' `paths_follow_the_stock_and_branch_rules_in_order`:
     the request's two rows, over Draft first's rules with CI and both tiers,
     and a merge job: a template that designs and closes out, its whole path
     stop for stop, Draft first's with "design" in place of "plan" and
     "close-out" right after the landing branch's "merge", before "verify"
     (as the "Yours" tab of `temper/docs/design/client/web/whole-story/how.html`
     orders them); and one that doesn't, Draft first's path as the table holds
     it now, "plan" and no close-out. A row by turn with `close_out`:
     "close-out" right after "turn";
   - `cards_open_the_setting_and_stay_within_the_bounds`: the "design" and the
     "close-out" stops' opening and cards, whole, as rows of its table; the
     length sweep with `design` and `close_out` set, so that both cards are held
     to the bounds;
   - `tests/fuzzy.rs`' `paths_end_at_a_bar_and_only_code_lands`: `design` and
     `close_out` drawn at random on a goal's start, and asserted on every seed:
     a `Design` stop iff the start is a goal with `design`, a `Plan` stop iff a
     goal without it; a `CloseOut` stop iff a goal with `close_out`, once, after
     the path's last `Merge` or `Turn` and before its last `Branch`, an agent's;
     the `needed` and small-words rules unchanged, `Design` and `CloseOut` among
     the steps that have neither. Its 64 seeds stay as they are.
7. **Bites:** draw "design" whatever the setting, see the fuzzy test fail on a
   seed without it; put "close-out" before the merge, see its row fail; restore
   each.
8. **Status:** delete the bullet naming `goal-path` in
   `temper/docs/design/domain/forge.status.md` (8.13) (`notes/common.md`,
   *Status files*).

## Inputs

- `temper/docs/design/domain/forge.md`, 8.13, whole: the derivation's rules and
  cards.
- `temper/docs/design/draft/stock-ways.md`, section 4, *What a goal's charter
  adds*.
- `temper/docs/plan/ui-p3f-design/notes/contracts.md`, sections 1 and 5.
- `temper/docs/plan/ui-p3f-design/notes/common.md`.
- The code: `temper-engine-domain-ways/src/path.rs` and `src/path/derive.rs`
  (`stops`, `Drawing::segment`, `land`, `tiers`, `agent`), `src/path/cards.rs`
  (`agent_card`); its tests in `src/tests/`, which `tests/suite.rs` builds.

## Owns and may change

- Owns: the ways crate's path, its two new steps and their stops, the service's
  two arms, the rows and the status bullet.
- Readers (acceptance 5): `temper-engine-domain/src/route/ways.rs` (`start`'s
  construction, `inputs`' match), `route/ways/view/roles.rs`,
  `route/ways/tests.rs`: the shapes' edits alone. Any other `match` on `Step`
  the compiler names takes the arm its nearest neighbour's case takes, named in
  the report.
- **A card with four facts:** the agent's card has three and the new fact makes
  four, `FACTS`; where the charter's agent is missing, `agent_card` gives one,
  and the new fact follows it. If a helper beside `agent_card` is cleaner than
  pushing onto its facts, write it in `cards.rs`.
- The service's `tests/family.rs` worst document: if its bound check meets the
  new steps, keep its bound and say so in the report; a raised bound is a
  question.

## Steps

1. Read the inputs.
2. The shapes and the readers' edits: `heavy cargo check --tests -p
   temper-engine-domain-ways -p temper-engine-domain -p temper-service -p
   temper-engine-smith`.
3. The two stops; their rows; the bites. `heavy cargo nextest run -p
   temper-engine-domain-ways`, and the fuzzy test by name in profile `fuzzy`.
4. Runs, counts in the report: `heavy cargo nextest run -p
   temper-engine-domain-ways -p temper-engine-domain -p temper-service`; the
   fuzzy profile's run of `-p temper-engine-domain-ways`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-engine-domain-ways -p temper-engine-domain -p temper-service
   --all-targets -- -D warnings`.
6. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the two stops' cards as drawn, the rows and
the fuzzy assertions, the bites and their failures, the runs' counts.
