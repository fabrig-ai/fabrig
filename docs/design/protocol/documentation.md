# Documentation in the binary

Provisional, 2026-10-26. temper's user documentation: one source in the
repository, compiled into the binary, read by people at the command line
and by agents through a host tool, and by the site from the same files.
It builds the draft `../draft/configuring.md`, section 7.

## 1. In one page

- **One source, three readers.** The topics are Markdown files in
  `docs/guide/`. `temper help` prints them; every run reads them with
  the `read_docs` host tool; the site renders the same files at a
  release's tag. Nothing is written twice.
- **For people who run temper,** not in the design's register: walks
  through a setup (a box, a server, a forge), the concepts (a task, a
  goal, a chat, a charter, a template, a policy, a landing rule, the
  settings and their revisions), recipes ("review before landing", "a
  tester that cannot land", "a weekly budget") and the limits (what
  temper cannot do, what needs a restart).
- **Compiled in, as data.** `temper-docs` holds every topic as static
  text with its name, kind, title and summary, and the functions that
  list, find, page and search them. It decides nothing and has no state:
  the same binary always says the same thing.
- **An agent reads it as it reads a file:** a page of a topic from an
  offset, or the sections matching some words. The read is answered
  where the call is decoded, deciding nothing and recorded nowhere, so
  it costs a run no authority, no commit and no spend beyond its tokens.
- **Checked against the code it describes.** A step test holds the table
  to the directory, every topic to its bounds, and every `temper ctl`
  line in a topic's examples to ctl's parser, so a recipe that no longer
  parses fails the gate.

## 2. The source

- **A topic is one file** of `docs/guide/`, named by its stem in lower
  case, letters, digits and `-`: `docs/guide/landing-rules.md` is
  `landing-rules`. Its first line is its title, `# ` and at most 80
  bytes; its first paragraph after it is its summary, one sentence of
  at most 200 bytes; its sections are its `## ` headings, each at most
  80 bytes.
- **Four kinds,** said by a line under the title, `Kind: walk`,
  `Kind: concept`, `Kind: recipe` or `Kind: limits`, which `temper help`
  groups by. The walks are today's guides (`box`, `server`, `forgejo`,
  `github`); the others are written with the parts they describe and
  land with them, so that no topic describes what `main` does not do.
- **Bounds:** a topic at most 32,768 bytes of printable ASCII and line
  ends, so that its rendered size is known (section 3); a name at most
  32 bytes; at most 64 topics; a topic's examples, in fenced blocks, as
  a person types them.
- **Links** between topics are `temper help NAME` in the text, which
  the site renders as a link and the step test resolves.
- **Where it lives** is the repository, beside the design (the draft's
  default): a change of behaviour and the topic that tells it land in
  one change.

## 3. `temper-docs`

A crate of no side (`README.md`, section 9), `#![no_std]` with `alloc`,
that depends on nothing of temper's and that `temper` and
`temper-engine-smith` depend on.

- **The table:** `TOPICS: [Topic; N]`, each `Topic { name, kind, title,
  summary, text }` as `&'static str`, its text the file's whole bytes by
  `include_str!`, in the order the table lists them (walks, concepts,
  recipes, limits, each by name). The table is written by hand; the
  step test (section 6) keeps it equal to the directory.
- **`find(name) -> Option<&Topic>`**, by the exact name.
- **`page(topic, from, bytes) -> Page`:** the topic's text from byte
  `from`, cut at the last line end whose rendered size is within
  `bytes`, or, for a line longer than that, inside it; with `next`, the
  exact offset where the page stopped, or none at the end. Sizes are
  rendered sizes, as the text renders in a JSON string: 2 bytes for
  `"`, `\` and a line end, 1 for every other byte. A `from` past the
  end answers an empty page with no `next`.
- **`search(words, limit) -> Hits`:** the sections in which every word
  appears, a topic's head (its title, kind line and summary) being a
  section of its own, headed by its title, at offset 0, case folded on ASCII, in the table's order, then the
  section's: each hit the topic's name, the section's heading, the
  section's offset, and the section's first line holding the first
  word, cut to 120 bytes; at most `limit` hits, with whether more
  matched. Words are split on whitespace, at most 8 of at most 32 bytes
  each; more, or longer, is refused naming the bound.

## 4. `temper help`

- `temper help` prints the topics by kind, a line each: the name, the
  title and the summary.
- `temper help NAME` prints the topic whole, as its file holds it, and
  exits 0; a name that is no topic exits 1, printing `temper: no topic
  NAME; temper help lists them`.
- `temper help --search WORDS...` prints at most 20 hits, a line each:
  `NAME: HEADING: LINE`, then `and more; narrow the words` when more
  matched; none exits 1, printing `temper: nothing matches`.
- It reads no directory and no configuration: it works before `temper
  init`. `temper --help` stays the commands' usage, and `temper ctl help`
  the guide to ctl's tools (`../client/tools.md`, 11.5).

## 5. `read_docs`, for agents

- **Declared in every charter** by `temper-engine-smith`, beside the
  forge's reads (`../domain/agent.md`, 4.3): `read_docs` with either
  `{"topic": NAME, "from": OFFSET}`, `from` 0 when absent, or
  `{"search": "WORDS"}`, never both.
- **Answered at decode.** The engine's protocol layer decodes every
  admitted call (`../domain/root.md`, 5.2); a `read_docs` call it
  decodes is answered there, from `temper-docs`, as an undecodable call
  is answered, but as a success: the root relays the answer with no
  commit, no record and no check of authority. Documentation is the
  same for every run, so no authority covers or refuses it, and a call
  asked again is answered the same.
- **Its answer,** rendered by `temper-engine-smith` as every answer
  is, never past `DOC_ANSWER`, 2,048 bytes whole: for a topic, its name
  and title, a page of at most 1,536 rendered bytes, and either `next:
  OFFSET` or `end`; for a search, at most 8 hits, each its topic's name,
  heading, offset and line, with whether more matched; for a name that
  is no topic, `no topic NAME` and the hits of a search for NAME's
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
| `temper-docs`' step tests | the table equals `docs/guide/`'s files, in name and bytes; each topic's title, kind line, summary and headings within section 2's bounds; every `temper help NAME` in a topic names a topic; `page` over a topic's lines (a cut at a line end, a long line cut at a character, `from` past the end, `next`); `search` over a small fixture table (all words, case, order, the limit and more) |
| `temper`'s step tests | every fenced line of every topic that starts `temper ctl ` parses with ctl's argument parser to a tool; `temper help`, `help NAME`, an unknown name and `--search` print and exit as section 4 says |
| `temper-engine-smith`'s step tests | `read_docs`' schema; a topic's first and next page; a search; an unknown topic; both members, or neither, answered as an error naming what is wrong |
| the engine's domain world | a run's `read_docs` call answered with no commit, no record, and the run going on (one story) |

None is a story of the real loop or of the process tier, and none adds
to a suite's time beyond milliseconds.

## 7. Open questions

- **Translations.** Default: English only.
- **The site.** Default: it renders `docs/guide/` at a release's tag;
  temper serves no documentation page of its own until the user's pages
  ask for one.
- **A topic per release's changes,** read by the configuring agent when
  a release's revision changed a default (`../draft/configuring.md`,
  3.2). Default: the release notes are a topic, `releases`, newest
  first.
