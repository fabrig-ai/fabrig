+++
id = "role-links"
part = "links"
kind = "feature"
needs = ["role-pages", "text-once"]
files = [
  "temper/crates/temper-client-view/src/pages/ask_page.rs",
  "temper/crates/temper-client-view/src/tests/ask.rs",
  "temper/tests/client/tests/suite/ways.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 40
+++

# A card's link opens its role or its rule; the terminal names each region once

## Goal

The request's task 6 and temper's half of its task 8 (`temper/docs/plan/look-round-24/inputs/request.md`). Every stop's card links "Open NAME ›" to `Settings { project }` (`temper-client-view/src/pages/ask_page.rs`, `card`, about line 214). A charter's link now opens its role's page, a gate's its reviews' charter's role (the wire's `OpensGate.charter`, from `ways-wire`), a rule's the rule's page (from `role-pages`). And Ask temper's terminal print, with `text-once`'s rule, says "Ways of working:" and "The way a change goes:" once each, which its test now holds.

Acceptance, by `temper/docs/design/client/README.md`, 3.5, *Ask temper's page*, *a stop's card* (as this cut wrote it), and `skein/docs/design/ui-text.md`, 5:

1. **The link's target** in `card`: `wire::Opens::Charter(c)` → `Address::Role { project, charter: c.charter() }`; `wire::Opens::Gate(g)` → `Address::Role { project, charter: g.charter() }`; `wire::Opens::Rule(r)` → `Address::Rule { project, kind: r.kind(), number: r.number() }`. Its words, its place and the stops without a link are as today.
2. **The view's rows** (`temper-client-view/src/tests/ask.rs`, `stop_presses_open_move_close_and_tabs_close_cards`, about lines 591 and 652): "Open the reviewer ›" has the href `/projects/1/settings/roles/N`, N the fixture's reviewer charter as its `OpensGate` gives it; "Open CI ›" `/projects/1/settings/rules/K/M`, K and M the fixture's rule; a charter's stop (produce) its producer's role. No new test function: where a fixture's `OpensGate` lacks the charter, it is `ways-wire`'s field, set there.
3. **The terminal's once each** (`terminal_prints_tabs_stops_card_legend_and_act_in_order`, about line 782): beside its ordered words, `text.matches("Ways of working:").count() == 1` and `text.matches("The way a change goes:").count() == 1`. It fails on the trunk before `text-once`, which this task needs.
4. **The meeting point** (`temper/tests/client/tests/suite/ways.rs`, `keep_the_current_way_completes_to_begin_and_is_not_offered_again`, about line 183): after it finds "Open the reviewer ›", the person follows it, and the page's main has the heading "The reviewer" and the term "Asked to", read through the engine's own `charter N` view; then the story goes back to Ask temper's page (`Act::Go { address: Ask { project } }` or the rail, as the world drives pages) and continues as today. No new test function.
5. **Status:** delete the **Not yet** entry naming `role-links` in `temper/docs/design/client/README.status.md` (3.5, near its end). No other design line.
6. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-view`, `-p temper-client-world`, `-p temper-client-gallery`, `-p temper-client-shell-web`; clippy on `temper-client-view` and `temper-client-world`.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`, `temper/docs/plan/look-round-24/notes/contracts.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, tasks 6 and 8.
- `temper/docs/design/client/README.md`, 3.5, *Ask temper's page* (*a stop's card*; the terminal's print) and *A role's or a rule's page*; 4.6, *A path* (`OpensGate`'s `charter`).
- `skein/docs/design/ui-text.md`, 5 (a repeated `NAME:` prints once).

## Owns and may change

Owns the link's target, the rows, the story's lines and the status entry. Not the pages (`role-pages`), not the view's other pages, not the gallery's samples (its `ask-ways-card` asserts the link's words, not its target: `temper/tools/gallery/src/tests.rs`, about line 2278).

Options, each decided:

- **The world can't follow a link** by an act it has (read how its other stories go to a page): drive `Act::Go { address: Role { project, charter } }` with the href the link carries, decoded by the codec, and say so.
- **The rule's page in the world** draws "no rule …" or the unknown view's words until `rule-view` lands: assert no rule's page here; the reviewer's role is the meeting point.

## Steps

1. Read the Inputs, `card` and the tests named above.
2. The targets (acceptance 1) and the rows (2); bite: send the gate to `Settings` again, see the reviewer's href row fail, restore.
3. The once-each assertion (3); bite: in a scratch edit of `ask_page.rs`, wrap the tabs in a list named "Other" so that their `NAME:` follows another line, see the count assertion fail on "Ways of working:", restore; the story (4).
4. The status entry (5), the runs (6); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| "Open the reviewer ›" opens the reviewer's charter | landed (`ask_page.rs`: the gate's charter's role page) |
| Every card's charter or rule link | landed: a charter's role, a rule's page |
| The terminal: "Ways of working:" and "The way a change goes:" once each | landed (`skein-ui-text`, `text-once`; this task's assertion) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the targets as written, the rows and the bites with their failures, the story's lines, the runs' counts, and the account as committed.
