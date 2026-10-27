# Documentation in the binary

Provisional, 2026-10-26, revised 2026-10-27. temper's documentation,
for agents: pages in the repository, compiled into the binary, read at
the command line by the agents that run and drive temper and by
temper's own runs through a host tool. It builds `../draft/agent-docs.md`
(the readers, the map, the kinds and the voice), which takes over
`../draft/configuring.md` 7 for agents; this document is its contract
for the pages and their machinery, and `../client/tools.md` for ctl's
help. People read the site, which is outside temper's design.

## 1. In one page

- **For agents.** The operator's agent runs the binary, the driver
  works through `temper ctl`, and temper's runs read with `read_docs`
  (`../draft/agent-docs.md`, section 2). Every page is written for an
  agent about to act.
- **Pages in `docs/help/`, one table.** `temper-docs` holds a row per
  page: its name, kind, title, summary and text. The table is the map
  (`../draft/agent-docs.md`, section 4), and the directory equals it.
- **Five kinds, each with its template and bound:** `index`, `concept`,
  `walk`, `recipe` and `reference`, the last generated from the code's
  tables. Today's guides are a sixth, `guide`, transitional, for their
  eight names alone, until each is rewritten into the others (2.2).
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
- **Placeholders,** in written pages, are `../draft/agent-docs.md` 5's
  set alone: `DIR`, `CONFIG`, `DATA`, `CACHE`, `FILE`, `TEXT`, `KEY`,
  `TASK`, `PROJECT`, `FORGE`, `INSTALLATION`, `ACCOUNT`, `REVISION`,
  `PROPOSAL`, `PROVIDER`. In a fenced line that starts `temper `, a
  word of capitals is one of them, or one that a walk's `Before you
  start` declares, in a bullet that starts with it in a code span.
- **Where it lives** is the repository: a change of behaviour and the
  page that tells it land in one change.

### 2.2 The kinds

| Kind | Bound | Headings, in order | Names |
|---|---|---|---|
| `index` | 6,144 bytes | free; it names every other page but the reference and the guides once, by `temper help NAME` | `index` alone |
| `concept` | 8,192 bytes | `What it is`; `How it changes` (optional); `What temper guarantees`; `Acting on it`; `Limits` | the map's nouns |
| `walk` | 12,288 bytes | `Before you start`; numbered steps, `## 1. …`; `Clean up`; `Next` | the map's walks |
| `recipe` | 3,072 bytes | `Goal`; `Settings`; `Enforced and followed`; `Checking it took` | the map's recipes |
| `reference` | 32,768 bytes | a heading per row, its usage's words | generated (8.1 of the draft) |
| `guide` | 32,768 bytes | free | `box`, `budgets`, `charters`, `forgejo`, `github`, `server`, `settings`, `tasks`, while each is not yet rewritten |

- **A walk's step** holds its commands, then a paragraph that starts
  `Expect:` with the output's first line verbatim, then one that starts
  `If not:` with each other outcome a sound temper gives and its act.
- **A concept** gives no verb's arguments or flags (`Acting on it` names
  its ctl tools and host tools), and its `Limits` link the page
  `configuration` for what needs a restart once that page exists.
- **A rewritten guide** takes its place in the map under its new name
  and kind in the same change that removes its `guide` row: `tasks`
  becomes `task`, `chat` and `goal`; `charters` becomes `charter`,
  `template` and `policy`; `settings` becomes `settings`, `revision` and
  `proposal`; `budgets` becomes `budget`.
- **At most 96 pages,** `MAX_PAGES`.

### 2.3 The voice, checked

Outside code spans and fenced blocks, a written page holds none of these
words of the design's: `jig`, `smith`, `skein`, `.md`, `temper-` (a
crate's prefix). The rest of the voice (`../draft/agent-docs.md`,
section 6) is the writer's and the review's.

## 3. `temper-docs`

A crate of no side (`README.md`, section 9), `#![no_std]` with `alloc`,
that depends on nothing of temper's and that `temper`,
`temper-client-tools` and `temper-engine-smith` depend on.

- **The table:** `PAGES: [Page; N]`, each `Page { name, kind, title,
  summary, text }` as `&'static str` (kind an enum of section 2.2's six),
  its text the file's whole bytes by `include_str!`, in this order:
  `index`, then the concepts, walks, recipes, reference pages and guides,
  each by name. The table is written by hand; the step test (section 6)
  keeps it equal to the directory.
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

- `temper help` prints the page `index` whole.
- `temper help NAME` prints the page whole, as its file holds it, and
  exits 0; a name that is no page exits 1, printing `temper: no page
  NAME; temper help lists them`.
- `temper help --search WORDS...` prints at most 20 hits, a line each:
  `NAME: HEADING: LINE`, then `and more; narrow the words` when more
  matched; none exits 1, printing `temper: nothing matches`.
- It reads no directory and no configuration: it works before `temper
  init`. `temper --help` stays the commands' usage until the binary's
  table (`../draft/agent-docs.md`, 7.1); `temper ctl help` prints the
  page `ctl`, then the tools (`../client/tools.md`, 11.5).

## 5. `read_docs`, for agents

- **Declared in every charter** by `temper-engine-smith`, beside the
  forge's reads (`../domain/agent.md`, 4.3): `read_docs` with either
  `{"topic": NAME, "from": OFFSET}`, `from` 0 when absent, or
  `{"search": "WORDS"}`, never both. A topic is a page.
- **Answered at decode.** The engine's protocol layer decodes every
  admitted call (`../domain/root.md`, 5.2); a `read_docs` call it
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
  cannot fit (`configuration.md`, section 7); every shipped profile's
  are above it.
- **A read,** which smith may run beside others (smith's
  `domain/run.md`, 5.2).

## 6. Testing

| Where | What |
|---|---|
| `temper-docs`' step tests | the table equals `docs/help/`'s files, in name, kind and bytes; each page's head within 2.1's bounds; each page within its kind's bound, its headings its kind's (2.2), a walk's steps each with `Expect:`; the `guide` rows among the eight names alone; `index` names every concept, walk and recipe once; every `temper help NAME` in a page names a page; placeholders (2.1) and the voice's words (2.3) in written pages; `page` over a page's lines (a cut at a line end, a long line cut at a character, `from` past the end, `next`); `search` over a small fixture table (all words, case, order, the limit and more) |
| `temper`'s step tests | every fenced line of every page that starts `temper ctl ` parses with ctl's argument parser to a tool, cut at its first shell operator (`|`, `>`, `<`, `;`, `&&`) and each placeholder replaced by a value of its kind (`PROPOSAL` by `t1/1`, a number's by `1`), and every one that starts `temper ` and names another command parses with the binary's; each reference page equals what its table renders (`settings-fields` first, from ctl's `FIELDS`, `../client/tools.md` 5.3), printing the page as it should be; every setting field a written page names in a code span as a path from its kind (`charter.model.name`) resolves to a row of `FIELDS`; `temper help`, `help NAME`, an unknown name and `--search` print and exit as section 4 says |
| `temper-engine-smith`'s step tests | `read_docs`' schema; a page's first and next cut; a search; an unknown page; both members, or neither, answered as an error naming what is wrong |
| the engine's domain world | a run's `read_docs` call answered with no commit, no record, and the run going on (one story) |

None is a story of the real loop or of the process tier, and none adds
to a suite's time beyond milliseconds.

## 7. Open questions

- **Translations.** Default: English only.
- **The site.** The user's: whatever it renders of `docs/help/`.
- **A page of a release's changes,** read by the configuring agent when
  a release's revision changed a default (`../draft/configuring.md`,
  3.2). Default: a page `releases`, newest first, joining the map with
  the first release.

## 8. The reference pages

Each reference page is rendered from one table by the crate that holds
the table, checked in under `docs/help/reference/`, and held equal to
its rendering by `temper`'s step test (section 6), which prints the page
as it should be. A table's text (its lines and notes) is documentation,
in `../draft/agent-docs.md` 6's voice.

| Page | Its table | A row |
|---|---|---|
| `commands` | the binary's commands (`crates/temper`; `README.md`, 4) | a command: its words, usage, line, arguments and notes |
| `ctl-GROUP`, a page per group | ctl's tools (`temper-client-tools`; `../client/tools.md`, 5.1) | a tool: its usage, line, arguments, result line, keyedness and notes |
| `settings-fields` | `FIELDS`, beside the settings form's decoder (`temper-settings-form`; `../client/tools.md`, 5.3) | a member of a shape |
| `host-tools` | the host tools' declarations (`temper-engine-smith`; `../domain/agent.md`, 4.3) | a host tool: its name, its description, then each top-level member of its input's schema with its type, bound, whether it is required and its description (a nested member stays in the schema the run reads); in the one order the run's tool list has |
| `limits`, a page per owner prefix where one page would pass 32,768 bytes | the limits' names: temper's (`temper-service`, today the overrides' match) and jig's (`jig-protocol-config`) | a named limit: its dotted name, its unit (a count, bytes, milliseconds), the small profile's value, a line |
| `configuration` | the configuration document's members, beside the document's decoder (`configuration.md`) | a member: its path from the document's root, its type and bound, what `temper init` writes, a line |

- **`host-tools`' text is the run's:** the page renders the same
  description a run reads beside each tool, at most 160 bytes, and a
  member's, at most 120, bounds that `temper-engine-smith`'s step tests
  hold; so a run's tool and its page never differ.
- **`limits`:** the overrides' parser becomes a lookup in the names
  table, so that a name the parser takes is a row and a row a name it
  takes. jig gives its own names as a list beside its parser, each with
  its unit and line, generic to jig's applications, which temper's
  table renders with its own.
- **`configuration`:** every member is read at a start, so a change to
  any of them needs a restart; the page says so once, and the concepts'
  `Limits` link it. A step test holds the table to the decoder: every
  member `temper init` writes and every member the decoder takes is a
  row, and every row is one of them. Where the decoder is jig's, its
  members' rows are jig's, as `limits`' are.

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
  binary's refusals at a command's start (`temper`, `temper-service`'s
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
