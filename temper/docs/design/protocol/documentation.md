# Documentation in the binary

Provisional, 2026-10-26, revised 2026-11-11. temper's documentation,
for agents: pages in the repository, compiled into the binary, read at
the command line by the agents that run and drive temper and by
temper's own runs through a host tool. It builds `../draft/agent-docs.md`
(the readers, the map, the kinds and the voice); this document is its
contract for the pages and their machinery, and `../client/ctl/README.md` for ctl's
help. People read the site, which is outside temper's design.

## 1. In one page

- **For agents.** The operator's agent runs the binary, the driver
  works through `temper ctl`, and temper's runs read with `read_docs`,
  except Ask temper, which reads with `see help`
  (`../domain/agent/agents.md`, 1). Every page is written for an agent
  about to act.
- **Pages in `docs/help/`, one table.** `temper-docs` holds a row per
  page: its name, kind, title, summary and text. The table is the map
  (2.4), and the directory equals it.
- **Six kinds, each with its template and bound:** `index`, `concept`,
  `walk`, `recipe`, `way` and `reference`, the last generated from the
  code's tables (2.2).
- **Compiled in, as data.** `temper-docs` holds every page as static
  text, and the functions that find, page and search them. It decides
  nothing and has no state: the same binary always says the same thing.
- **An agent reads a page as it reads a file:** a page of it from an
  offset, or the sections matching some words. A run's read is answered
  where the call is decoded, deciding nothing and recorded nowhere
  (section 5).
- **Checked against the code it describes.** Step tests hold the table
  to the directory, each page to its kind's template and bound, every
  command line in a page to its parser, every address to a page, every
  named setting field to its table, the placeholders to their set, and
  the generated pages to what their tables render (section 6).

| Reader | Who | Reaches the text by | Needs first |
|---|---|---|---|
| The operator's agent | sets up and keeps a deployment for a person: `init`, `box` or `serve`, a forge, accounts, an upgrade (`binaries.md`) | a shell, often before anything is configured: help works before `init` (section 4) | the index, a walk, the binary's reference |
| The driver | works through a deployment for a person: adopts, asks, follows, answers, decides, configures | `temper ctl`, a process per call | the page `ctl`, then each result's acts; a group's reference; a concept when the model surprises it |
| temper's own runs, except Ask temper | a chat, a coder | its charter's instructions, its host tools' descriptions, and `read_docs`; no shell into temper | the settings' concepts and reference, the recipes, the limits |
| Ask temper | configures how work runs for a person | its charter's instructions, its host tools' descriptions, and `see help`; no shell into temper | the settings inventory, then each setting the change touches, whole, by the addresses the inventory names; it explains from what it reads and from temper's pages (`../domain/agent/agents.md`, 1), its instructions naming no list of pages |

- **The moments:** first contact (the index, then a walk); every call
  (its result and acts); a failure (the error, its remedy, an address);
  a surprise in the model (the concept); configuring (the settings'
  concepts, their reference and the recipes).
- **QA reads what a user reads.** A pass is given a walk and its rung
  (`../../development/qa.md`, section 3), never a list of commands, so
  a rename reaches it through the walk. A step it finds only in the
  design or the source is a finding (`../../development/qa.md`,
  section 6).
- **People read the site,** the user's: it may render any page of
  `docs/help/` as it is; nothing here is written for it.
  `temper open`, a person's terminal, keeps its usage line alone.

## 2. The source

### 2.1 A page

- **One file** of `docs/help/`, the generated ones under
  `docs/help/reference/`, named by its stem in lower case, letters,
  digits and `-`, at most 32 bytes, unique across both: `landing-rule`.
- **Its head:** its first line is its title, `# ` and at most 80 bytes;
  then a line `Kind: K`; then its summary, one paragraph of one
  sentence of at most 200 bytes. Its sections are its `## ` headings,
  each at most 80 bytes.
- **UTF-8 with no control character but the line end,** so that its
  rendered size is known (section 3) and a walk's `Expect:` quotes
  ctl's output as it prints, its ` · ` among it. Its examples are
  fenced blocks, as an agent types them.
- **Links** between pages are `temper help NAME` in the text, NAME in
  lower case, which the step test resolves; a word of capitals after
  `temper help` is a placeholder, never a link.
- **Placeholders:** one set, defined once in the index, and no other in
  a written page: `DIR` (ctl's directory), `CONFIG`, `DATA` and `CACHE`
  (the three directories), `FILE`, `TEXT`, `KEY`, `TASK`, `PROJECT`,
  `FORGE`, `INSTALLATION`, `ACCOUNT`, `REVISION`, `PROPOSAL`, `PROVIDER`.
  A walk that needs another value names it in `Before you start`. In a
  fenced line that starts `temper `, a word of capitals is one of them,
  or one that a walk's `Before you start` declares, in a bullet that
  starts with it in a code span. Reference pages print the tables'
  argument names (`../client/ctl/tools.md`, 1).
- **Where it lives** is the repository: a change of behaviour and the
  page that tells it land in one change.

### 2.2 The kinds

| Kind | Bound | Headings, in order | Names |
|---|---|---|---|
| `index` | 6,144 bytes | free; it names every other page but the reference pages once, by `temper help NAME` | `index` alone |
| `concept` | 12,288 bytes | `What it is`; `How it changes` (optional); `What temper guarantees`; `Acting on it`; `Limits` | the map's nouns |
| `walk` | 12,288 bytes | `Before you start`; numbered steps, `## 1. …`; `Clean up`; `Next` | the map's walks |
| `recipe` | 3,072 bytes | `Goal`; `Settings`; `Enforced and followed`; `Checking it took` | the map's recipes |
| `way` | 8,192 bytes | numbered sections, `## 1. …`, from 1; free otherwise | a way of working temper ships: `how-temper-builds-temper` |
| `reference` | 32,768 bytes | a heading per row, its usage's words | generated (8.1 of the draft) |

- **A way** is read by a person as well as an agent: a person reads
  `how-temper-builds-temper` with `temper help`, and Ask temper with `see
  help` (`../domain/agent/agents.md`, 1); the client draws it nowhere
  (`../client/pages/README.md`, *temper's own way*). Its summary is the
  line under its title, of up to two sentences.
- **A walk's step** holds its commands, then a paragraph that starts
  `Expect:` with the output's first line verbatim, then one that starts
  `If not:` with each other outcome a sound temper gives and its act.
- **A concept** gives no verb's arguments or flags (`Acting on it` names
  its ctl tools and host tools), and its `Limits` link the page
  `configuration` for what needs a restart.
- **At most 96 pages,** `MAX_PAGES`.

### 2.3 The voice, checked

- **For an agent about to act:** the present of `main`; steps in the
  imperative; one fact a sentence.
- **temper's words,** each defined once, in its concept; never a word of
  the design's: no crate, no section number, no kit's name.
- **Exact:** commands as typed, values with their units and bounds,
  outputs verbatim, a limit as a fact.
- **Nothing twice:** a fact lives on one page and others give its
  address. The exit classes and the wait conditions are `ctl`'s; what
  needs a restart is `configuration`'s; a verb's syntax is its row's.
- **True of `main`:** no page describes what `main` does not do. A
  behaviour and the text that tells it land in one change.

Outside code spans and fenced blocks, a written page holds none of these
words of the design's: `jig`, `smith`, `skein`, `.md`, `temper-` (a
crate's prefix). The rest of the voice is the writer's and the review's.

### 2.4 The map

The map's single copy is `temper-docs`' table, a row per page; this
section is its design. It lists the end state: a page joins the table
with the part that makes it true of `main`.

| Noun | Concept | ctl groups | Covers |
|---|---|---|---|
| ctl | `ctl` | the client's: `sign-in`, `sign-out`, `pending`, `help` | a call's loop (read, act, wait), the wait conditions, keys and kept requests, acts and the revisions they name, the exit classes, JSON; the forge read with tools of the agent's choosing, and what temper owns there changed only through temper |
| deployment | `deployment` | `deployment` | the engine, the worker, a box, a server, the three directories, the configuration document |
| party | `party` | `me`, `party` | people and services, roles, pools, signing in to temper, credentials as files |
| project | `project` | `project` | projects, their home, their members |
| forge | `forge` | `forge` | kinds, modes (connected, managed, owned), GitHub's App and installations, the repositories a forge reaches |
| repository | `repository` | `repository` | adoption, its role, the home, the landing branch, CI and checks |
| account | `account` | `account`, `model` | LLM providers, accounts, sources, keys, a provider's sign-in, a program's login, a move between deployments, models |
| task | `task` | `task` | tasks and their parts (tree, activity, turns, run), delegates, phases, a task's own holds and their release, stop and cancel, amendments, a task's settings revision |
| hold | `hold` | `hold` | a placed hold: its owner, its mode (stop or drain), its scope (a task's subtree or a project), what lifts it; placed by `task hold` and `project hold` |
| chat | `chat` | `chat` | a conversation, its template, following it as a task, closing it |
| goal | `goal` | `goal` | tracked work, priority, plans, advisors |
| change | `change` | `change` | a change's way to landing: its pull request, CI, gates, queue, repairs, holds, merge; what the forge shows and temper does not |
| landing | `landing` | `landing` | a landing task: its members, each candidate with its base, its gates' receipts and runs, its proof, the turn; a repository's receipts (`repository receipts`) |
| finding | `finding` | `finding` | something found that is not its finder's to fix now: a labelled issue in the project's home, its kind, severity, area and state; triage (route, backlog, drop) |
| qa | `qa` | `qa` | QA after landing: the marks of what a user can now do, the chooser, the passes on the newest cleared `main` |
| inbox | `inbox` | `inbox` | what waits on the party: questions, proposals, escalations; updates |
| proposal | `proposal` | `proposal` | proposals of every kind, deciding, withdrawing |
| settings | `settings` | `settings` | the settings and their kinds, applying, proposing, running work |
| revision | `revision` | `revision` | revisions, the history, restoring, moving running work to the newest |
| charter | `charter` | (settings) | a charter's fields and defaults; the forge's charter |
| template | `template` | (settings) | a template's fields and defaults |
| policy | `policy` | (settings) | a policy's fields and defaults; the rules |
| landing rule | `landing-rule` | (settings) | a landing rule, the deployment's and a project's |
| budget | `budget` | (none) | what work spends, budgets, deadlines |
| playground | `playground` | `playground` | the demo, its fakes, starting it |

- **Walks:** `box` (a local box from nothing to a landed change, and
  back), `server` (a long-lived deployment), `forgejo` and `github`
  (connecting each, GitHub's shared App read-only among them),
  `configure` (changing how work runs), `upgrade`.
- **Recipes:** "review before landing", "a tester that cannot land", "a
  weekly budget". Each comes with the part that makes it true.
- **Reference pages:** `ctl-GROUP` for each group, `commands` for the
  binary's, `host-tools`, `settings-fields`, `configuration` (its
  members, and which need a restart) and `limits` (the bounds).
- **The rules:** every group names its concept; a concept may cover
  several groups, or none; a row's name, kind and noun are the table's,
  and the gate holds the directory to the table. A noun, walk or recipe
  the design does not name is no row.

## 3. `temper-docs`

A crate of no side (`../README.md`, section 9), `#![no_std]` with `alloc`,
that depends on nothing of temper's and that `temper-box-shell`,
`temper-client-tools`, `temper-engine-smith` and `temper-client-view` depend
on.

- **The table:** `PAGES: [Page; N]`, each `Page { name, kind, title,
  summary, text }` as `&'static str` (kind an enum of section 2.2's six),
  its text the file's whole bytes by `include_str!`, in this order:
  `index`, then the concepts, walks, recipes, ways and reference pages,
  each by name. The table is written by hand; the step test (section 6)
  keeps it equal to the directory.
- **`HOW_TEMPER_BUILDS_TEMPER`:** that page's row as a `const Page`, the
  table's entry its copy, so that the client's view takes this page alone
  and its bundle none of the others.
- **`find(name) -> Option<&Page>`**, by the exact name.
- **`page(page, from, bytes) -> Cut`:** the page's text from byte
  `from`, cut at the last line end whose rendered size is within
  `bytes`, or, for a line longer than that, inside it at a character's
  boundary; with `next`, the
  exact offset where the cut stopped, or none at the end. Sizes are
  rendered sizes, as the text renders in a JSON string: 2 bytes for
  `"`, `\` and a line end, 1 for every other byte, a character beyond
  ASCII its bytes. A `from` past the
  end answers an empty cut with no `next`.
- **`search(words, limit) -> Hits`:** the sections in which every word
  appears, a page's head (its title, kind line and summary) being a
  section of its own, headed by its title, at offset 0, case folded on
  ASCII, in the table's order, then the section's: each hit the page's
  name, the section's heading, the section's offset, and the section's
  first line holding the first word, cut to 120 bytes at a character's
  boundary; at most `limit`
  hits, with whether more matched. Words are split on whitespace, at
  most 8 of at most 32 bytes each; more, or longer, is refused naming
  the bound.

## 4. `temper help`

Every binary's `help`, `BINARY` its name (`../cli/README.md`, 5):

- `BINARY help pages` prints the page `index` whole.
- `BINARY help NAME` prints the page whole, as its file holds it, and
  exits 0; a name that is no page exits 1, printing `BINARY: no page
  NAME; BINARY help lists them`.
- `BINARY help --search WORDS...` prints at most 20 hits, a line each:
  `NAME: HEADING: LINE`, then `and more; narrow the words` when more
  matched; none exits 1, printing `BINARY: nothing matches`.
- `BINARY help commands` prints the page `commands` for the binary's own
  table (8).
- It reads no directory and no configuration: it works before
  `temper-box init`. `BINARY -h`, `--help` and `help` alone print the
  binary's page from its table, its public commands, flags and
  variables (`skein/docs/design/runtime/shell.md`, 14.2); `temper ctl
  help` prints the page `ctl`, then the tools
  (`../client/ctl/testing.md`, 5).

## 5. `read_docs`, for agents

- **Declared in every charter but an asking charter (asking, with settings
  at most; `../domain/agent/tools.md`),** whose
  runs read the same pages through `see help` (`../domain/agent/tools.md` and `../domain/agent/ask-temper.md`
  ), by `temper-engine-smith`, beside the
  forge's reads (`../domain/agent/tools.md`): `read_docs` with either
  `{"topic": NAME, "from": OFFSET}`, `from` 0 when absent, or
  `{"search": "WORDS"}`, never both. A topic is a page.
- **Answered at decode.** The engine's protocol layer decodes every
  admitted call (`../domain/routing.md`, 2); a `read_docs` call it
  decodes is answered there, from `temper-docs`, as an undecodable call
  is answered, but as a success: the root relays the answer with no
  commit, no record and no check of authority. Documentation is the
  same for every run, so no authority covers or refuses it, and a call
  asked again is answered the same.
- **Its answer,** rendered by `temper-engine-smith` as every answer
  is, never past `DOC_ANSWER`, 2,048 bytes whole: for a page, its name
  and title, a cut of at most 1,536 rendered bytes, and either `next:
  OFFSET` or `end`; for a search, at most 8 hits, each its page's name,
  heading, offset and line, with whether more matched; for a name that
  is no page, `no topic NAME` and the hits of a search for NAME's
  words. Each part's bounds (a name's 32 bytes, a heading's 80, a
  line's 120, a title's 80) make the whole fit `DOC_ANSWER`, which a
  step test proves at the bounds. The start refuses a configuration
  whose `core.call_answer_bytes` or local agent's `tool_payload` is
  below `DOC_ANSWER`, naming it, as it refuses other answers that
  cannot fit (`configuration/limits.md`); every shipped profile's
  are above it.
- **A read,** which smith may run beside others (smith's
  `domain/run.md`, 5.2).

## 6. Testing

| Where | What |
|---|---|
| `temper-docs`' step tests | the table equals `docs/help/`'s files, in name, kind and bytes; each page's head within 2.1's bounds; each page within its kind's bound, its headings its kind's (2.2), a walk's steps each with `Expect:`; `index` names every concept, walk and recipe once; every `temper help NAME` in a page names a page; placeholders (2.1) and the voice's words (2.3) in written pages; `page` over a page's lines (a cut at a line end, a long line cut at a character, `from` past the end, `next`); `search` over a small fixture table (all words, case, order, the limit and more) |
| `temper-box-shell`'s step tests | every fenced line of every page that starts `temper ctl ` parses with ctl's argument parser to a tool, cut at its first shell operator (`|`, `>`, `<`, `;`, `&&`) and each placeholder replaced by a value of its kind (`PROPOSAL` by `t1/1`, a number's by `1`), and every one that starts `temper-box ` and names another command parses with the binary's; each reference page equals what its table renders (`settings-fields` first, from ctl's `FIELDS`, `../client/ctl/tools.md` 3), printing the page as it should be; every setting field a written page names in a code span as a path from its kind (`charter.model.name`) resolves to a row of `FIELDS`; a page that says a ctl group or tool does not exist (`no ctl group `NAME``, `NAME (no tool)`) fails when the table has it, so a tool's landing finds the pages that deny it; `temper help`, `help NAME`, an unknown name and `--search` print and exit as `documentation.md`, section 4 says |
| `temper-engine-smith`'s step tests | `read_docs`' schema; a page's first and next cut; a search; an unknown page; both members, or neither, answered as an error naming what is wrong |
| the engine's domain world | a run's `read_docs` call answered with no commit, no record, and the run going on (one story) |

None is a story of the real loop or of the process tier, and none adds
to a suite's time beyond milliseconds.

## 7. Open questions

- **Translations.** Default: English only.
- **The site.** The user's: whatever it renders of `docs/help/`.
- **A page of a release's changes,** read by Ask temper when
  a release's revision changed a default (`../draft/configuring.md`,
  3.2). Default: a page `releases`, newest first, joining the map with
  the first release.

## 8. The reference pages

Each reference page is rendered from one table by the crate that holds
the table, checked in under `docs/help/reference/`, and held equal to
its rendering by its crate's step test (section 6), which prints the page
as it should be. A table's text (its lines and notes) is documentation,
in section 2.3's voice.

| Page | Its table | A row |
|---|---|---|
| `commands` | the binaries' tables, as the page is compiled (8.1; `temper_shell::table::commands_reference`), and each binary's own, as its `help commands` prints it (`temper_shell::table::commands_page`; `../cli/README.md`, 5) | a command, under each binary that runs it, its public ones first, then its internal ones: its words, forms, line, arguments and notes |
| `ctl-GROUP`, a page per group | ctl's tools (`temper-client-tools`; `../client/ctl/tools.md`, 1) | a tool: its usage, line, arguments, result line, keyedness and notes |
| `settings-fields` | `FIELDS`, beside the settings form's decoder (`temper-settings-form`; `../client/ctl/tools.md`, 3) | a member of a shape |
| `host-tools` | the host tools' declarations (`temper-engine-smith`; `../domain/agent/tools.md`) | a host tool: its name, its description, then each top-level member of its input's schema with its type, bound, whether it is required and its description (a nested member stays in the schema the run reads); in the one order the run's tool list has |
| `see` | `see`'s addresses (`temper-engine-smith`; `../domain/agent/ask-temper.md`, 1) | an address: its form, each form the instructions' line brackets a row of its own (`settings charter N`, `task N tree`), a line on what it answers, and the addresses one level down, its `see:` line |
| `limits` and `limits-engine` | the limits' names: `limits` temper's, those `application.limits.overrides` takes (`temper-service`'s, the setup's `setup.*` and the box's `box.*`), and `limits-engine` jig's, those `engine.limits.overrides` and a worker's `limits.overrides` take (`jig-protocol-config`) | a named limit: its dotted name, its unit (a count, bytes, milliseconds), the small profile's value, a line |
| `configuration` and `configuration-engine` | the configuration document's members by shape, beside each decoder: `configuration` temper's, the document's top level and `application`'s shapes, beside temper's decoders; `configuration-engine` jig's, `engine`'s and a worker's shapes, beside `jig-protocol-config`'s (`configuration/README.md`) | a member of a shape, the shape's section naming every path it is read at: its type and bound, whether it is required, what `temper-box init --box` writes, a line; the members that left the document for the store (`engine.charters`, `templates`, `policies` and `rules`; `application.charter` and `application.landing`'s rules: `configuration/settings.md`, 4) are one row each, saying that a start refuses it and where it went, as the refusal says, and sending the reader to `settings` |

- **`host-tools`' text is the run's:** the page renders the same
  description a run reads beside each tool, at most 160 bytes, and a
  member's, at most 120, each present, never empty: bounds and presence
  that `temper-engine-smith`'s step tests hold; so a run's tool and its
  page never differ, and a tool added without its words fails the gate.
- **`limits` and `limits-engine`:** each parser of overrides looks a
  name up in its names table before anything else, refusing a name the
  table lacks as unknown, so that a name the parser takes is a row; a
  step test applies every row's name at the small profile's value, so
  that a row is a name the parser takes. A row reads its value from the
  profile, never a copy of it. jig gives its own names as a list beside
  its parser (`jig-protocol-config`'s `names` module), each with its
  unit, its line and its value's reader, generic to jig's applications,
  which temper renders as `limits-engine`; temper's table, in
  `temper-service`'s `limits` module beside its parser, holds its own
  and the setup's and the box's, which their crates give as lists
  beside their parsers. A line is at most 120 bytes, so that each page
  stays within 32,768.
- **`configuration` and `configuration-engine`:** every member is read
  at a start, so a change to any of them needs a restart; each page says
  so once, and the concepts' `Limits` link `configuration`. Each shape's
  decoder reads its members through its table, a member it asks for
  that the table lacks failing every test that decodes it, so that
  every member the decoder takes is a row; a step test holds that every member
  `temper-box init --box` writes is a row, and that every row is a member the
  decoder takes or one of the members gone, which its decoder refuses
  naming where it went. Where the decoder is jig's, its members' rows
  are jig's, as `limits-engine`'s are.
- **`configuration`'s last section, `The merge command`,** gives the
  contract `merge_job`'s command is written to (`tooling.md`, 4.1) to
  whoever writes one, since a member's line holds at most 160 bytes: that
  member's line points to it. Its words sit beside the page's renderer,
  which reads the bounds and the `PATH` it names from the limits that hold
  them, so the section cannot drift from them.

### 8.1 The page `commands`

The compiled page names each command's binary
(`../draft/deployment-binaries.md`, section 9, item 6): it is rendered
from the deployment's tables and the box's, in this order, the
client's, `temper-engine`'s, `temper-worker`'s, `temper-agent`'s and
`temper-box`'s (`../cli/README.md`, 6 and 7).
`temper-box-shell`'s step test holds the checked-in page equal to that
rendering, since the box's crate links every table.

- **Its head:** the title `temper's commands`, `Kind: reference`, and
  one line: `Every command of temper's binaries, a section each: its
  usage, what it does, its arguments and its notes. A command two
  binaries share is written under the first and named under the
  others.`
- **A section per binary,** `## BINARY`, then the table's line, then
  each of its commands as a section of its own, `### BINARY WORDS` (`###
  BINARY` for the bare form, `### run as PROGRAM` for one run as
  another name), its public commands first, then its internal ones, in
  the table's order, as `commands_page` renders one.
- **A command an earlier table holds,** the same `skein_shell::Command`
  value, is named, not written again: its heading and its usage lines
  under this binary's name, then `As BINARY WORDS, above.`, naming the
  first binary's section. So `temper-box forge fetch` points to
  `temper-engine forge fetch`, and `temper-box program-login` to
  `temper-worker program-login`.
- **Bounded as every reference page** (32,768 bytes); each binary's
  `help commands` still prints its own table alone, by `commands_page`.

## 9. Errors

What an agent meets when something goes wrong is documentation at the
point of use (`../draft/agent-docs.md`, section 3), and the most read.

- **An error is one line, at most 240 bytes:** what is wrong, in the
  concept's words; the value or bound that made it so; and the act that
  mends it, as a call to copy, or the page that explains, as `temper
  help NAME`. `temper ctl: settings file item 2: charter.alternatives:
  at most 16; remove one` is one; `invalid form` is none.
- **Its areas,** each with its own renderer: ctl's own failures (exit
  1, `temper-client-tools` and `temper-client-shell-cli`); the
  engine's refusals as ctl renders them (exit 2, jig's and the forge's
  words, `temper-client-domain` and `temper-client-tools`); the
  binary's refusals at a command's start (`temper-box-shell`, `temper-service`'s
  startup, the box's and setup's); a run's host tools' error answers
  (`temper-engine-smith`).
- **Typed refusals are rendered from one table** per area, a row per
  refusal with its words: the engine's refusals are enums, so a step
  test holds that every variant has a row. A free-form error (a start's
  check of the document) keeps its words where it is made, under the
  same rule, and the area's golden tests hold its lines.
- **The words are documentation:** written in the pages' voice, never
  quoting a design's section; a remedy names the tool by its words
  (`settings show`), as `help` prints it.
- **An area is brought to the rule** by an inventory of its lines
  (each line, where it is made, what makes it fire), its words
  rewritten, and its golden tests updated, in one landing per area; a
  page that quotes a changed line changes with it.
