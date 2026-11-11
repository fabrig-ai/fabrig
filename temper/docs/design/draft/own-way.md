# temper's own way, shown

Draft, 2026-11-05, from a session with the user that day, who asked how
a person new to temper, on "Configure temper", would learn that temper
runs work at scale, as fabrig builds temper with temper, and that this
is "just fabrig/temper's own 'way of doing'", whose parts "can be
tweaked to a good extent". The way itself is the reference
(`reference-code-production.md` and `archive/reference-landing.md`); this
draft is how a person meets it. The look request
`configure-temper-own-way` draws it (`../client/web/journeys.md`, 7).
Nothing here is adopted, and no plan takes it up until the user does.

## 1. In one page

- **A card on "Configure temper":** "At scale, as temper builds
  itself", its steps, "See how it works" and "Start from it", under the
  ways to begin (section 2).
- **A reference document,** "How temper builds temper": stock, shipped
  with temper, never written by a model; opened beside the page, it
  changes nothing (section 3).
- **Its table,** temper itself beside what is the person's to set: each
  row a setting the reference sets (`reference-code-production.md`,
  section 14), or marked as later (section 4).
- **"Start from it":** the configuring agent proposes the reference's
  settings, sized for the deployment (section 6).

## 2. On "Configure temper"

- **The card** follows the ways to begin: its title, "At scale, as
  temper builds itself"; its steps in a line, "Draft → The cut →
  Coding agents → Goal's branch → main"; its two acts. "Not sure yet?
  Ask what temper can do." moves into the section's head, so the page
  still shows whole on a laptop.
- **"See how it works"** opens the reference beside the page, as the
  configuring chat's document opens (Chat, Split and Draft; Draft on a
  phone). No chat is made.
- **"Start from it"** starts the configuring chat with its words as the
  first message, "Set Hello up the way temper builds itself, sized for
  this machine.", as a suggested request does (`../client/pages/README.md`
  ), the project's name in place of Hello.

## 3. The reference document

- **Stock, versioned with the release:** Markdown in the binary, read
  by skein-ui's reader (`skein/docs/design/ui/markdown.md`), its
  picture a flow (`skein/docs/design/draft/diagrams.md`); written from
  the reference drafts, in a person's words, and changed with them.
- **Its sections:** one picture, the flow from a design draft to main;
  a pipeline that keeps growing (the cut and early starts:
  `reference-code-production.md`, sections 5 and 7); in parallel, kept
  together (a change into the goal's branch, catching up with main,
  questions: `reference-code-production.md`, sections 6, 11 and 8);
  landing many (`archive/reference-landing.md`;
  QA after landing, `../domain/findings.md`, section 8); temper itself,
  and yours (section 4).
- **Read, never run:** opening it starts nothing; a question about it
  is the configuring chat's, beside it.

## 4. Its table

- **A row per part:** Plans, Models, Workers, Branches, Landing, QA and
  Questions. "temper itself" is the reference's value; "Yours to set"
  names the range, from where a new deployment starts to the reference.
- **Each row is a setting,** named where the reference sets it
  (`reference-code-production.md`, section 14): the goal template for
  drafts; each change's LLM choices; the profile's limits and the
  fleet's slots (`../protocol/configuration/README.md`, section 2); the
  landing rules (`../domain/landing.md`, section 13); QA after landing;
  questions. A row whose setting is not a project's yet says "later",
  never offers what cannot be set.

## 6. Starting from it

- **The configuring agent** (`configuring.md`) reads the reference
  with its instructions (`../domain/agent/agents.md`, 1) and proposes its settings sized for the
  deployment: the slots its workers have, the models its accounts reach,
  QA after landing once a tester is set; what the deployment cannot do
  yet, it says, and leaves out.
- **Its document,** "How Hello works", keeps temper's own beside
  Hello's, a column of the table, so that each difference is a sentence
  the person can change.

## 7. Open questions

- **When:** the user's; this draft keeps the idea, and no plan takes it
  up until then (2026-11-05).
- **Settings or the deployment's:** which rows a project sets and which
  are the deployment's (workers and their slots are the root's
  configuration: `../protocol/configuration/README.md`, section 2), and how
  the table says so.
- **Where the reference lives:** a stock file in the binary, as the
  stock templates are (`../protocol/configuration/stock.md`), or a
  document note the deployment seeds (`jig/docs/design/draft/notes-memory.md`,
  section 5).
- **The picture's parallel parts:** a flow is one path; the parts made
  in parallel are a caption until skein's flows branch
  (`skein/docs/design/draft/diagrams.md`, section 9).
