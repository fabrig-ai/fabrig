+++
id = "rule-view"
part = "cards"
kind = "feature"
needs = ["asked-sentence"]
files = [
  "temper/crates/temper-engine-domain/src/route/ways/view.rs",
  "temper/crates/temper-engine-domain/src/route/ways/view/address.rs",
  "temper/crates/temper-engine-domain/src/route/ways/view/rule.rs",
  "temper/tests/engine/domain/tests/suite/ways.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 75
+++

# The view `rule K/N`: a landing rule's card

## Goal

The engine's half of the request's task 6 (`temper/docs/plan/look-round-24/inputs/request.md`): a card's rule link ("Open CI ›", "Open owner ›") is to open the rule, and no read draws one. The forge's read `View` now answers `rule K/N` with the rule's card, as it answers `charter N` with a role's; the client's page for it is `role-pages`', built to the same wire.

Acceptance, by `temper/docs/design/domain/forge.md`, 8.13, *A rule's card*, and `temper/docs/design/client/README.md`, 5.16, *The addresses*, as this cut wrote them, and `temper/docs/plan/look-round-24/notes/contracts.md`, *The view address*:

1. **The address** (`route/ways/view/address.rs`): `Address::Rule { kind: u32, number: u32 }`, parsed from `[b"rule", word]` where `word` is `K/N`, `K` 1 or 2 and `N` by `natural`; no `@revision` (a `rule` with one is `Unknown`).
2. **The drawing** (`route/ways/view/rule.rs`, a new module beside `charter.rs`): `pub(super) fn draw(domain: &Domain, project: u32, party: u64, kind: u32, number: u32) -> ways::View`, reading the landing rule as the forge holds it now: kind 1, the deployment's rule `number` (scope 0); kind 2, the project's, its rule 1 the stock's while it has none of its own (`forge.md`, 8.13, *Over a project*). It answers `ways::View::Rule { kind, number, card }`, the card as 8.13's *A rule's card* says, word for word:
   - `who` "a landing rule · holds a merge until it is met" where it enforces, "a landing rule · advice, enforcing nothing" where not; `name` "the deployment's rule N" or "the project's rule N";
   - its facts in order, each only where the rule has it: "Covers"; "Asks" ("CI", "up to date", joined by ", "); "Gates" (each "the reviewer, blocking" or "the reviewer, advice", ", at a clean head" where its freshness is `clean`, the name the review stop's card gives the reviews' charter, joined by "; "); "Approvals" (each as the approve stop's words say it: "you approve" where the reader holds the role, else the role's name, "an owner approves", "×N" past one, joined by "; ").
   - "Covers": a pattern beginning `branch` open with nothing, "every branch"; `branch` exact X, "the branch X"; `branch` open P, "branches beginning P"; one beginning `forge`, "OWNER/NAME's B" with B its branch words as above but lower case ("every branch", "branch X", "branches beginning P"). Reuse a function that already words a pattern if one exists (`rg -n 'GatePattern' temper/crates/temper-engine-domain*/src`); otherwise write these words here.
   - Every value through `cut_on_word(value, VALUE)` (from `asked-sentence`).
   - A rule there is none of: `not_drawn(&[b"no rule ", K, b"/", N, b"."])`.
3. **The dispatch** (`route/ways/view.rs`): `Address::Rule { kind, number } => rule::draw(...)`; the unknown view's words name the new view after `charter N`: "views: way NAME · path OWNER/NAME [BRANCH] · proposal tN/K · charter N · rule K/N."; `fact` there bounds its value with `cut_on_word(value, ways::VALUE)` in place of its character cut, so that the charter's and the rule's views cut as the stops' cards do.
4. **The rows, no new test function:** `views_draw_stored_ways_paths_charters_and_waiting_proposals` (`temper/tests/engine/domain/tests/suite/ways.rs`, about line 493) gains rows: `rule 2/1` over the world's project (its rule 1, as the world sets it, or the stock's), its whole card; `rule 1/1` where the deployment has no rule, "no rule 1/1."; `rule 3/1`, `rule 2/0` and `rule 2/1 @revision 2`, the unknown view's words; and the two unknown-view rows already there take the new list of views. Its `Picture` gains `Rule { kind, number, who, name, facts }` beside `Charter`, and `drawn` maps `ways::View::Rule` to it.
5. **Status:** delete the **Not yet** entry naming `rule-view` in `temper/docs/design/domain/forge.status.md` (8.13, *A rule's card*, at the file's end). No other design line.
6. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-engine-domain` and `-p temper-engine-domain-world`, `-p temper-service`; clippy on `temper-engine-domain` and `temper-engine-domain-world`.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`, `temper/docs/plan/look-round-24/notes/contracts.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, task 6.
- `temper/docs/design/domain/forge.md`, 8.13, *A rule's card*, *Over a project*, *A typical run* (5, the approvals' words); 8.9 (landing rules); `temper/docs/design/protocol/configuration.md`, 13.2 and 13.3 (a rule's pattern and how rules add).
- `temper/docs/design/client/README.md`, 5.16, *The addresses* and *Read when shown*.
- `route/ways/view/charter.rs`, the model for `rule.rs`.

## Owns and may change

Owns the files above. Reader files, call sites only: `mod rule;` in `view.rs`. Not the derivation (`temper-engine-domain-ways`), the wire or the service's mapping (`ways-wire` lands it), not `charter.rs` (`asked-sentence`'s), not Ask temper's instructions (`stock/ask.txt`), which name `path`, `way`, `charter` and `proposal` alone (`temper/docs/design/domain/agent.md`, 8.4): the view is the client's page's.

Options, each decided:

- **A help page lists the views** (`rg -n 'proposal tN/K' temper/docs/help`): add `rule K/N` there in the same words, within its bound, and run its tests (`temper/docs/help/**` is read by tests); name it.
- **The role names of approvals** come from the project's roles as the approve stop's words take them (`derive.rs`, about line 208): call the same words; if they are private to the derivation, compute them here the same way and say so.

## Steps

1. Read the Inputs, `address.rs`, `view.rs`, `charter.rs` and the world's table.
2. The address and its unknown rows (acceptance 1, 4); bite: accept `K` 3, see the `rule 3/1` row fail, restore.
3. The drawing, the dispatch and the card's rows (2 to 4); bite: drop "Asks", see the `rule 2/1` row fail, restore.
4. The status entry (5), the runs (6); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| A rule's card, the view `rule K/N` | landed (`temper-engine-domain`, `route/ways/view/rule.rs`) |
| The charter's and the rule's views cut on a word | landed (`view.rs`, `fact`) |
| A card's rule link opening it | `role-pages` and `role-links`, this plan's |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the address and the words as written, the rows and the bites with their failures, the runs' counts, and the account as committed.
