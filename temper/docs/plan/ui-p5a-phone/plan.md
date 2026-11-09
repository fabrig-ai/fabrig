# ui-p5a-phone: plan

Milestone P5a of `temper/docs/design/draft/whole-story.execution.md`: the
phone, Ask temper on it, and Tell temper
(`temper/docs/design/draft/surfaces.md`, sections 9 and 10). The design
this cut wrote: `temper/docs/design/client/README.md`, 3.5, *One layout for
every chat*, *Ask temper's page* (*The composer*), *Needs you* (*Its head*,
*On a phone*) and *Tell temper*, and 5.3, *On a phone*;
`temper/docs/design/client/browser.md`, 2.5 (the phone's paragraph, *What a
phone presses*); their status files; and lags 4, 9 and 10 of
`temper/docs/design/draft/surfaces.status.md`, with two departures for 10.

## Parts

| Part | What it makes run |
|---|---|
| `tabs` | the tab bar from one pure rule, `rail::tabs`; "Needs" with its count; Ask temper's page and its chats keep the tabs on a phone, "Ask" current; the top bar's and the Ask tab's targets at 44 px |
| `pages` | Needs you's own top bar on a phone, "‹ Needs you" and "1 of 2", the card as `phone-need.html` draws it; Home's, Needs you's and Work's targets at 44 px |

## Tasks

| Task | Part | Needs | Crates | Minutes |
|---|---|---|---|---|
| `tabs-surface` | tabs | | `temper-client-domain` (`rail::Tabs`, `rail::tabs`), `temper-client-world` (rows of `fuzzy::rail_and_work`) | 60 |
| `phone-pages` | pages | | `temper-client-view` (Needs you's bar, the 44 px table test), `temper.css`, the gallery's rows | 170 |
| `phone-tabs` | tabs | `tabs-surface`, `phone-pages` | `temper-client-view` (the tab bar, `TABBAR_ASK`), `temper.css`, the gallery's rows, the journeys reprinted | 180 |

## Critical path

`phone-pages` (170) → `phone-tabs` (180), about 350 implementer-minutes;
`tabs-surface` (60) runs beside `phone-pages`. `phone-tabs` waits on
`phone-pages` only because both write `temper-client-view`'s `look.rs`,
`words.rs`, `tests/look.rs` and `temper.css`, and the gallery's tests: one
writer per crate.

## Decided

- **Tell temper is designed, not built.** The engine files no finding
  (`temper/docs/design/domain/findings.status.md`: sections 2 to 9,
  whole), `findings.md` names no person's word among its filers, and no
  project's settings name a triager (`landing.md`, 13.1: kind 4 is not yet
  a setting; P4a's Team draws no triager line for the same reason). Filing
  needs skein's label writes on both forges, a `Finding` family and its
  effects, the findings setting and the triager's wake: temper/migration's
  design, and more than the execution's first step, which "asks nothing new
  of the engine" (its section 4), may take. By the rule of every milestone
  (its section 2), Tell temper draws nothing until it can file: no `Tell`
  address, no region at Needs you's foot, no link on the Ask tab, no
  placeholder. Its client design is written whole (`README.md`, 3.5, *Tell
  temper*), its **Not yet** in `README.status.md`, and the question
  `tell-temper-findings` asks who builds the filing; no task waits on it.
  So the request's joint world for Tell temper's filing has nothing to
  join yet.
- **The phone's pages already stand.** P1 drew the top bar, the switcher
  with "All projects" and the Away chip; P2 drew Home's phone form (`all-home`,
  the cards' full-width act and "More", "Decided for you · N ›") and Needs
  you one card at a time. This plan draws what the mockups still differ in:
  the tab's "Needs", the Ask tab's tabs, Needs you's top bar, and the 44 px
  targets section 10 asks.
- **The tab bar gets a rule of its own,** `rail::tabs(&Parts) -> Tabs`,
  over the rail's parts, so that the fuzz test of the phone's parts has a
  pure function to hold: its rows join P1's sweep of the rail's parts and
  its settings moves (`fuzzy::rail_and_work`), a check per seed, not a new
  test. Home's and Needs you's parts are P2's `pages::parts`, the same at
  every width, which P2's `fuzzy::density` already sweeps.
- **The gallery's phone states are those it has:** a print is one page at
  every width, so `all-home` and `night-all-home` are `phone-home.html` by
  day and night, `needs-list` and `night-needs` are `phone-need.html`, and
  `ask`, `ask-chat` and `night-ask` the Ask tab. The gallery's tests are past
  their budget, so this plan adds rows that read those prints, no state.
- **"‹ Needs you" goes to Home,** where Done's "Home" goes, as the mockup's
  link does; it is named by its words.
- **Every target at 44 px, where the phone's four tabs go:** the frame,
  Home, Needs you, Work, Ask temper's page and its chats. A goal's, a
  task's, another chat's and How work runs' controls keep the greenfield
  mock's sizes, as the request `ui-phone` of 2026-11-05 left them: a
  departure in `surfaces.status.md`. A control the mockup draws smaller
  keeps its drawing and takes its target around it.
- **Team, How work runs and Deployment on a phone** stay in the switcher's
  menu, as P1 and P4a drew them: `phone-home.html` draws no project rows
  and no phone a person's menu, whose parts bind. A departure in
  `surfaces.status.md`.
- **The need's "1 of 2" moves to its bar;** the dots and the "Next:" line,
  which `phone-need.html` doesn't draw, leave the view (they were the
  browser's alone: the terminal's Needs you is its inbox).
- **The pair and team phone Homes** (`pair/phone-home.html`,
  `team/phone-home.html`) draw sign-offs (N2), a change's size (P7) and a
  clock in the top bar, which is the device's: none is this plan's.
- **Stored formats, protocols, public interfaces:** none changes.

## Who hears first

- **temper/tui (the TUI's owner):** `temper-client-domain` gains
  `pages::rail::Tabs` and `pages::rail::tabs(&Parts) -> Tabs`, the phone's
  tab bar's rule; nothing the TUI reads changes, and the terminal has no tab
  bar.
- **The owner of coding runs:** nothing in `temper-engine-domain` or
  `forge_route.rs` changes.
- **temper/migration:** the question `tell-temper-findings` (findings'
  filing, with a person's word among its filers, and a triager named by
  the settings).

## Overlaps not avoided

- **ui-reaches (N1, integration):** its `stop-asking` task writes
  `temper-client-view/src/pages/needs.rs`, `look.rs`, `words.rs`,
  `temper.css` and the gallery's tests, which `phone-pages` writes too. The
  why and "Stop asking" stay in the card under the phone's bar; the
  overlap is the files, not the meaning.
- **ui-p4a-team (ready):** `frame.rs`, `words.rs`, `look.rs` and the
  gallery's tests; its switcher row "Team for PROJECT" is untouched here.
- **look-round-25 and ui-p3e-words (integration):** `look.rs`, `words.rs`,
  `temper.css` and the gallery's tests; `chat-draft`'s journey, which
  `phone-tabs` reprints.
- **Every UI plan:** `look.rs`'s style tokens, `words.rs`'s `Word` and the
  gallery's `tests.rs` are appended by all of them.
