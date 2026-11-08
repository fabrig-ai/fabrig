# look-ask-ways: plan

Step 4 of `temper/docs/design/draft/stock-ways.md`, section 7: the look
request `ask-temper-ways` (`98322b3565` on `origin/look-requests`),
landed as `temper/docs/design/draft/look-requests.md`, sections 3 and 5
ask, every state its message names, nothing drawn by hand
(`inputs/request.md`). The engine's side is on main (`ui-ask-m3a`): the
`ways` read, the paths and their cards; the client's domain already
watches `ways` and sends "Use this" and "Keep it". What is left is the
client's: the page's state, the drawing in the request's look, the
opening itself, the removal of what it replaces, the gallery's states,
the joined stories and the account.

The cut wrote the design first: `temper/docs/design/client/README.md`,
3.5 (*Ask temper's page*: the opening; *temper's own way*), 5.13 (*A
way's acts*: the page's state and the toast), 5.16 (*Drawn*, *Its
layout*, *Pressed*) and section 11 (the render check after landing);
`browser.md`, 2.5, 4.6's story 11 and 7.2's `ask-ways-*` states;
`journeys.md`, 9; `../domain/agent.md`, 8.1; their statuses, and the
drafts' (`stock-ways.status.md`, `own-way.status.md`).

## Parts and tasks

- **state:** `opening-state`, high, 110 minutes, no needs: the tab shown
  and the open stop on `StockWays`, the acts `WayTab` and `WayStop`, a
  way's act disabled while out, and the toast naming the revision
  (`notes/contracts.md`, section 1).
- **drawing:** `path-look`, high, 200, no needs: the path across and
  down the page, its bars as ends and goal branches, two tones of line,
  pressed stops with their halo and caret, and its rules (contracts,
  section 2).
- **opening:** `opening`, high, 260, needs both: the opening's tree
  (contracts, section 3), its words, tokens and rules, the drawing's act
  decoded, the old opening's parts removed from the page, the gallery's
  two reference states removed and the rest regenerated.
- **removal:** `starters-out`, medium, 80, needs `opening`: the domain's
  `Act::Starter`, `Act::ShowReference` and `Chats::reference`, the pane
  and their words, tokens and rules.
- **gallery:** `opening-gallery`, high, 120, needs `opening`: the ways
  delivered to `ask-first-run` and eight `ask-ways-*` states.
- **stories:** `opening-stories`, high, 150, needs `opening`: the joined
  world's table of the three ways over CI and both tiers and over
  neither, a stop's card, "Yours", the refusal in words, "Keep it" with
  its toast, a member; the browser tier's row written and compiled.
- **account:** `account`, high, 90, needs all: the request's account in
  its commit, and the statuses closed.

Slice 1 holds state and drawing; slice 2 opening, removal, gallery and
stories; slice 3 the account. No walk in the plan: the render check and
the walk follow landing (`README.md`, section 11; `stock-ways.md`, 7).

## Critical path

`path-look` (200), `opening` (260), `opening-stories` (150), `account`
(90): about 700 implementer-minutes. `opening-state` (110) runs beside
`path-look`; `starters-out` (80) and `opening-gallery` (120) beside the
stories.

## What was decided

- **The page's state is the domain's:** the tab shown and the open stop
  live on `StockWays`, switched by `WayTab` and `WayStop`, in no
  address, so that the view stays a function of the root.
- **One drawing, laid out by its width:** down the page under 560 pixels
  (or where the stops would stand closer than 48), where the request's
  page has two SVGs toggled by a media query; the scene knows its width,
  and one drawing keeps one fallback for a screen reader.
- **Stops pressed through skein's drawing acts** (`ui.md`, 11.3): a
  stop's group carries its act, the fallback's lines are buttons doing
  the same for the keyboard; the request's `role="button"` and
  `tabindex` on a `<g>` can't be drawn by a scene, which carries no ARIA.
- **The card's caret is the scene's:** a point drawn under the open stop
  at the scene's foot, exact at any width, where the request positions a
  `::before` by a percentage the tree cannot carry.
- **Two tones for the line,** where the request fades it, since a scene
  carries no gradient.
- **The reference loses its way in, and the client its copy:** with
  "See how it works" gone nothing reaches the pane, so the pane, its act
  and its mode go (`starters-out`); the page stays a help page Ask
  temper reads. Where else it is reached from is `own-way.status.md`'s
  **Open**.
- **"Keep it" and "Use this" toast the revision** (`Toasted::Way`), as
  the request asks, worded in 5.13.
- **Every commit carries the request's trailer,** as
  `look-requests.md`, sections 2 and 3, ask.

## Overlaps

Within the plan: `opening-state` adds the toast's words and arm to
`temper-client-view`'s `words.rs` and `frame.rs` while `path-look` owns
that crate's `paths`, `views.rs`, `look.rs` and `temper.css`: different
files. `starters-out` and `opening-gallery` run together; after
`opening` the gallery names neither removed act, so `starters-out`
should not touch it.

With drafts beside this one: `look-outside-pages` (integration, not yet
landed) changes `temper-client-view`'s `look.rs`, `words.rs`,
`tests/look.rs`, `frame.rs`, `temper.css` and the gallery. This plan
appends its tokens after the last (that plan reuses freed numbers below
them), numbers its words from 4300 (that plan reuses freed ones), puts
its rules under their markers, and regenerates the gallery's files on a
catch-up; `opening` and `opening-gallery` are the tasks that meet it.
