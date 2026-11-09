# ask-temper: execution

2026-11-08. What of [ask-temper.md](ask-temper.md) gets built, and in what
order. It designs nothing: each milestone writes its own design from the
draft as it runs (`DESIGN.md`, section 3). A milestone that meets a question
the draft leaves open takes its section 12 default, or the nearest thing, and
says so in the draft's status.

## 1. In one page

- **Four milestones,** M1 to M4 (section 3): alone and small, reading,
  showing, remembering.
- **M1 first:** the chat takes the configuring chat's place, small, with what
  configuring needs, so that the user can try it from its landing.
- **M3a before M2:** the stock ways and the views drawn from settings
  (`stock-ways.md`), which Ask temper's opening needs; then M2, `see`'s
  remaining addresses, then M3b, the views drawn from M2's reads.
- **M3b after the whole story** (the user, 2026-11-08): nothing in
  `whole-story.md` needs its views, so it runs after that draft's stories.
- **M4 after M1,** beside the others: the person's preferences, which need
  jig's party scope first.
- **The instructions are whole from M1** but their Showing part, which joins
  at M3, and their Remembering part, at M4; an address not built yet answers
  so (`ask-temper.md`, section 8).
- **Then the user takes over:** they try it and say what changes.

## 2. How a milestone runs

- **Its entry here is its input,** with `ask-temper.md` its source.
- **Its design and its build together,** landed whole: the design documents'
  sections it needs (`ask-temper.status.md`'s lags), then the code and its
  focused and fuzzy tests (`ask-temper.md`, section 10).
- **Its walk** follows its landing and blocks nothing.
- **Its state** is section 3's table, kept current. When M3b, the last,
  has landed, `ask-temper.status.md` holds no lag, and this file goes with
  the draft (`DESIGN.md`, section 3).

## 3. The milestones

| | Milestone | Executes (`ask-temper.md`) | Needs | State |
|---|---|---|---|---|
| M1 | Alone and small | 1, 4, 5.1 in part, 5.2, 5.3, 8, 9 | | built: plan `ui-ask-m1`; charter 3, template 3, `Configure` and Settings' offer gone by `ui-ask-drop`; its walk follows landing |
| M2 | Reading | 2, 3, 5.1 | M1 | built: plan `ui-ask-m2`, its design in `../domain/agent.md`, 8.4.1 and jig's `domain/engine.md`, 7.3; its walk follows landing |
| M3a | Ways | `stock-ways.md`, sections 4 to 6; 6.2's `path`, `way`, `proposal` and `charter` | M1; `stock-ways.md`, section 7's step 2 | built: plan `ui-ask-m3a`, and the page's drawing of the ways by the look request `ask-temper-ways`, plan `look-ask-ways`; its walk follows landing |
| M3b | Showing the rest | 6; the rest of 8's Showing part | M2, M3a; after the whole story | not started |
| M4 | Remembering | 5.4, 7 | M1 | built: plan `ui-ask-m4`, its design in jig's `domain/engine.md`, 10.1, and temper's `../domain/agent.md`, 8.4.4 and `../client/README.md`, 5.17; its walk follows landing |

### M1. Alone and small

- **Executes:** `ask-temper.md`, section 9: charter 11 and template 5 in the
  stock, charter 11 in the rules' and policy 1's delegation, "Ask temper" in
  the rail and the tab bar, and charter 3, template 3, `Configure` and
  Settings' offer gone by plan `ui-ask-drop`;
  4.2: no repository and no workspace for its runs; 4.3: its tools declared
  alone, in the asking family; 4.4: Claude Code's prompt replaced for a run
  with no workspace; 5.1: `see` with the addresses configuring reads,
  `settings` and its members, `settings landing` and `settings gates`, `revisions`, `revision N`, `proposal`, `help`,
  `file` and `more`, every other address answering that it is not yet; 5.2:
  `propose_settings` with charter 3's mechanics moved into it; 5.3:
  `document`; `wait`; `ask-temper.md`, section 8, as `ask.txt`,
  without its Showing and Remembering parts.
- **Needs first:** a real subscription's login taking a request whose prompt
  is replaced (`ask-temper.md`, 4.4), checked once before the build relies on
  it.
- **Tests:** `ask-temper.md`, section 10: what temper gives within its bound on
  both shapes; the four tools and no workspace; Claude Code's arguments on the
  fake program; M1's addresses.
- **Walk:** on a fresh `temper box` with a ChatGPT account and then a Claude
  one, Ask temper opened from the rail: asked "how does a change land here?",
  then "every change gets one review from me before it lands", through to its
  proposal and an owner's Accept.
- **Done** when it lands.

### M2. Reading

- **Executes:** `see`'s other addresses, `me`, `project`, `goals`, `tasks`,
  `task N` with its tree, activity and turns, `change N`, `changes` and
  `inbox`, and `ask-temper.md`, 5.1's rendering for all of them, of the chat's
  project alone.
- **Design:** jig's read as a party for a run (`ask-temper.status.md`).
- **Tests:** each address's bound and refusal, the chat's project alone; the
  rendering as tables; a fuzzy test of addresses.
- **Walk:** on the playground, after "Add CSV export to widget" has landed, Ask
  temper asked "where are we at?" and "how did that change run?".
- **Done** when it lands.

### M3a. Ways

- **Executes:** `stock-ways.md`, sections 4 to 6: the stock ways as
  data, their paths, the forge's `ways` read, `see ways`, the `view`
  block and the views `path`, `way`, `proposal` and `charter`, drawn by
  the browser from the same derivation; in `ask-temper.md`, section 8,
  `ways` on the `see` line and the Showing part's first step.
- **Needs:** `stock-ways.md`, section 7's step 2 on main, so that "Use
  this" can set every way.
- **Tests:** `stock-ways.md`, section 8.
- **The look request `ask-temper-ways`** landed straight after it, by
  plan `look-ask-ways` (`stock-ways.md`, section 7).
- **Done** when it lands.

### M3b. Showing

- **Executes:** `ask-temper.md`, 6.2: the `view` block's other views,
  drawn by the browser and zoomed by the client; 6.3: flows with `:::code`,
  `:::agent` and `:::person` and their `-strong` forms; the rest of the Showing
  part of `ask-temper.md`, section 8. The TUI prints views instead of drawing
  them (`surfaces.md`, 11.5), which `whole-story.execution.md`'s T1 builds.
- **Needs:** M2's reads, which the views draw from; the look designed with the
  user, as look requests are (`look-requests.md`).
- **Tests:** each view's address parses and draws from a world's stored state;
  a flow's marks are read; an unknown mark keeps the block fenced code.
- **Walk:** on the playground, Ask temper asked "show me how a change lands
  here" and "show me how task N ran", each view opened a level down.
- **Done** when it lands.

### M4. Remembering

- **Executes:** jig's party scope and its kind, as far as a person's
  preferences need them (jig's `draft/notes-memory.md`, sections 3 and 7);
  `remember` and its bounds (`ask-temper.md`, 5.4 and section 7); the list in
  each brief of this charter's runs, shown in a turn's read to its person
  alone; the Remembering part of `ask-temper.md`, section 8; the page "What temper remembers
  about you".
- **Tests:** the bounds; the list read by its person's runs of this charter
  alone; a turn's preferences hidden from another role of the project; a line
  changed and deleted from the page.
- **Walk:** Ask temper told "remember that I want short answers", then a new
  Ask temper chat answering briefly, and the line on the page, deleted there.
- **Done** when it lands.
