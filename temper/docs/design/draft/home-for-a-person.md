# Home for a person

Draft, 2026-11-15, from a session with the user that day. Home, Work and
Needs you as a person reads them: the shape of things at a glance, and
the details on each thing's own page. A recording of fabrig's own Home on
its deployment that Sunday was 5,506 px tall at 1366 by 768, eight
screens, with Needs you starting on the fifth. Most of its height was
agents' text shown as written: each decision opened with its goal's whole
prompt, and each goal's title was its prompt. The user: "the dashboard is
basically unreadable by a human right now, this is mostly a charter
problem", and the "goal is to communicate things visually as much as
possible". The same page with its content written for a person is 3,279
px, and the user kept it. With milestones (`milestones.md`), its pages
are the mockups in `../client/web/home-for-a-person/`, which the user
approved as the target on 2026-11-15. This draft says what stands behind
each part: what the client draws from what is served today, what agents
write, and what is new. It adds to `surfaces.md` and to the pages'
designs, and wins where they differ until its plans fold it in.

## 1. In one page

- **Half of it is served already,** and Home draws less: the kicker, the
  counts, the landing line, each decision's facts, the goals' counts and
  states, and every finding's kind, severity and age reach Home's reads.
- **The client draws the rest of what is served** (section 3): Needs you
  first, severity as a glyph and a colour, ticks, bars, and shorter
  fixed words.
- **Agents write for a person** (section 4): a goal's title is a short
  first line, a decision's question one sentence, a report's headline
  says where things stand with no numbers. This is charters' work, with
  no new field.
- **The triager groups findings** and asks its person one question a
  group (section 5).
- **The brief reads findings** (section 6), and agents name people
  (section 7).
- **Milestones** group goals (section 8).

## 2. The target

- **The mockups:** `home.html`, `work.html`, `milestone.html` and
  `needs.html`, drawn with the client's own stylesheets from a checkout
  (`../client/web/home-for-a-person/README.md`). Their README says what
  was recorded and what is illustrative.
- **Visual first:** a person gets the page before reading it. Severity
  is drawn twice, so that colour isn't alone: a chip's ground and a glyph
  of three bars. Goals have bars, milestones strips, landed decisions a
  tick.
- **Kept for the thing's own page:** prompts, paths, commits, change and
  pull numbers, measurements, and the agent's reasoning.
- **Precedence:** where these mockups and the whole story's differ
  (`surfaces.md`, section 13), these are the target for Home, Work's
  milestone cards, a milestone's page and Needs you's from line.

## 3. What the client draws

What is served today, drawn on Home (`../client/pages/home.md`, 1), and
on Needs you where it says so.

- **Needs you before Decided for you,** in the order `surfaces.md`,
  section 3 gives them, where `../client/pages/home.md`, 1 puts Decided
  for you first: what waits for the person comes before what was done for
  them.
- **The headline** says "11 findings need you." where every open card is
  a finding or a question about findings (section 5), and "things"
  otherwise, in number words as today.
- **Needs you's head:** "Needs you · 11"; then the findings' severities
  counted, "2 high · 4 medium · 5 low", each with its glyph; then "All
  needs".
- **Severity, drawn:** a glyph of three bars, one filled for low, two for
  medium and three for high, in the faint grey, the amber and the
  signal's colour, its word its label and its tooltip. A chip of findings
  takes its highest severity's ground.
- **A finding's row,** on a group's card and on a finding's own: its
  glyph; its title, a link to the finding; its kind, only where the card
  mixes kinds; and its age, in the mono face at the right.
- **The cards' order,** on Home and on Needs you: a card about findings
  by its highest severity, high first; any other card with high's; then
  the oldest first, as today.
- **"One more":** with one open card past the three, "One more", then its
  glyph where it has findings and its short line, on one line; with
  more, "N more", in number words to "Nine".
- **A finding the person holds** keeps its own card, with its row's parts
  and its acts as on Needs you (`../client/pages/needs-you.md`, 1, *A
  finding that is the person's*), and a why line by Tell temper's rule
  (*Tell temper*, there): "Reached you as PROJECT's triager: findings
  that no area owns come to you.", or "as PROJECT's owner" where no
  triager stands.
- **A why line said once:** where every card of Home's Needs you has the
  same why line, it is said once, under the cards, in the plural:
  "These reached you because they start work on findings, which asks
  you unless you’re away.", with its Stop asking. Otherwise each card
  says its own (`../client/pages/needs-you.md`, 1, *Why it reached
  you*).
- **Decided for you's rows:** "Allowed: wording choices.", the object
  alone; "landed Sat 19:01" or "ended Sat 21:22", with no " · a new goal
  can undo it"; a green tick before "landed"; and the agent's why left to
  the task's page.
- **Goals' rows:** a bar of the goal's tasks, done in ink and the one
  running in grey, from its done and total. It is approximate, since a
  done task isn't always a landing, and the goal's page keeps the exact
  one (`../client/pages/goal.md`, section 2). The fallback headline in
  number words, "Five of six tasks done.". A title past 80 characters cut
  at a word, with "…".
- **Work and Needs you's milestone parts** are `milestones.md`, section 5.

## 4. What agents write

Charters' words, with no new field. The stock's charters
(`../../../crates/temper-engine-service-stock/src/stock/`) and whoever
starts goals by script write them so.

- **A goal's title** is its words' first line, at most 60 characters,
  naming the work for a person: "Store shape S7 · history in task logs".
  The chat's `goal` and `change` calls open their words with such a line
  (`../domain/agent/tools.md`, sections 1 and 2), and `temper ctl goal
  start`'s help says so.
- **A decision's words:** `decided`'s question in one sentence, the
  situation as a person reads it, and `took` in one line, in every
  charter that decides: the chat's goals, the standing role's, the
  triager's, the prover's and the porter's.
- **A goal's report headline:** one line for a person, saying where the
  goal stands with no task, change or pull numbers: "5 of its 7 changes
  landed; the other 2 are building."
- **The agent's own text stays** where it is read whole: the thing's
  page, and Overturn, which quotes the question back.
- **No second text:** every part of the page has its field already, and
  a short first line and short fields serve the agent as well as the
  person. Section 10 keeps the question open for a field that must stay
  long.

## 5. Findings grouped by the triager

- **The triager** (`../domain/areas.md`, section 6) is a standing role
  holding the findings no area owns, and triages them one short run an
  item (`../../../crates/temper-engine-service-stock/src/stock/triager.md`).
  It also groups what it keeps, by what they are and what one act would
  settle, and asks its person one question a group: "Three bugs in
  temper, each with a clear fix. Start a goal for each?"
- **The question's form:** the question line form
  (`../domain/agent/agents.md`, section 2) gains two lines, read as
  `Saw:` and `Choice:` are: `Findings: 2/2 2/8 2/10`, the findings it
  concerns, and `Group: bugs · in temper`, a plural noun and a phrase.
  The client draws a question with a `Findings:` line as a group's card:
  its chip "3 bugs" on its highest severity's ground, the phrase, and
  "the oldest 1 d" from its findings' ages; the question; a row for each
  finding (section 3); its context; its choices, the recommended one
  first; and "Tell it something else". A `Findings:` line that doesn't
  read leaves the card a plain question.
- **Its choices** are the triager's acts over the group: "A goal for
  each", "One goal for all three", "Backlog them", "Hand them to CI's
  owner". The triager carries the answer out with its `finding` tool and
  its goals, one act a finding (`../domain/findings.md`, section 6), so
  no act over several findings is added.
- **Counting:** the headline and Needs you's head count the findings the
  open cards concern, each once, with their severities.
- **Its why:** a reason kind of its own, starting work on findings,
  whose class in the stock asks: "Reached you because it starts work on
  findings, which asks you unless you’re away." Its Stop asking offers
  "Let the triager start goals for findings, then tell you."
  (`what-reaches-you.md`, section 5).
- **Where no triager role stands,** the person holds those findings, each
  its own card as section 3 says. Grouping is the triager's judgement,
  and code groups nothing.
- **A project's triager** is its kind 15 setting, which a way names
  (`../domain/areas.md`, section 6), changed by proposal like any.

## 6. The brief reads findings

- **The status agent's words** (`../domain/forge/attention.md`, section
  3) gain the project's open findings: how many of each kind and
  severity, and the titles of the high ones, cut to the spec's bound
  after the goals and before the roles' Nows.
- **A run starts** when the open findings have changed since the last
  brief, as well as on landings, by the same rule's other bounds: at
  most once a sitting, the brief over an hour old or none.
- **Its instructions** name findings by what they are about: "Seven of
  the 11 findings are about tests: four fail only beside other tests,
  and three CI runs failed with no fault in the change they checked."

## 7. People by name

- **Agents know a person by number:** a brief says "[Requested by person
  N]" (`../domain/agent/agents.md`, section 2). It says "[Requested by
  person N, Sam]", from the project's people, as the status agent's
  words already name signers, and the charters write the name:
  "Sam asked to reopen pull #1292".
- **A person with no name** keeps their login, as the status agent's
  words have it.

## 8. Milestones

`milestones.md`, whole: Home's milestone rows, Work's milestone cards, a
milestone's page, and Needs you's from line and filter. Three goals of
one draft running at once, which the recording showed as three rows,
share their milestone's row.

## 9. Out

- **An exact goal bar on Home:** Home's is approximate until a read gives
  a goal's landings (`../client/pages/work.md`, 2, *A goal row's count of
  landings*); the goal's page keeps the exact one.
- **A second, human-facing text** in every record (section 4).
- **Grouping findings by code,** and acts over several findings at once.

## 10. Open questions

- **A person's field beside an agent's,** if a field must stay long for
  the agents that read it. Default: none; the short field serves both.
- **The groups' order across projects** on Needs you unfiltered. Default:
  section 3's order over every project's cards.
- **Names across projects:** a person named in another project's brief.
  Default: each project's people alone.
- **Findings in a brief across projects** (`surfaces.md`, section 15's
  open question). Default: each project's brief reads its own.
