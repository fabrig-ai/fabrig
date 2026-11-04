# retired-plans-out: contracts

Binding between the four audit tasks, which write the kept notes, and `keep-decisions`, which
writes their findings into the design. Read with `notes/common.md`.

## K1. The kept note

Each audit task writes one file, `temper/docs/plan/retired-plans-out/notes/kept-<group>.md`,
`<group>` being `goal`, `landing`, `configuring` or `playground` (its spec says which), and
nothing else. Its form:

```markdown
# kept: <group>

Read at `<the short hash of your base>`, <date -u +%F>. <n> directories, <m> findings.

## <tree>/docs/plan/<name>

- **Read:** plan.md; plan.toml, <q> questions; tasks' Answers, <a> files; notes/contracts.md;
  notes/common.md; <any other note the spec names>. (Say "absent" for what the directory lacks.)
- **Kept:** <group>-1, <group>-2 (or "none").
- **Held:** <item, a few words> (`<doc path>`, <section>); <item> (`<doc>`, <section>); ...
- **Superseded:** <item> (now: `<doc or code path>`, <section or item>); ... (or "none").
- **Not design:** <count> items, e.g. <one or two, a few words each>.

### <group>-1. <a title of a few words>

- **Kind:** decision | contract | open choice
- **Source:** `<path from the repository root>`, <heading, or "Answers, <date>">; <the date the
  source gives, YYYY-MM-DD>; decided by <the user | not stated>.
- **Checked:** `<doc>`, <section>: <what it says, or "silent">; <more, if searched>.
- **Target:** `<doc path>`, <section number>; <one of: a sentence or paragraph in the section |
  a **Not yet** entry | a **Departs** entry | a **Design lags** entry | a **Design departs**
  entry | an **Open** entry | an item of the document's Later | an item of its Open
  questions | none: `<AGENTS.md, DESIGN.md or skein/docs/foundation/...>`, the user's>.
- **Text:** <the text to add, ready to paste, in the target's words and form: a status entry
  as `<section>: <the gap>` with its reason; a design sentence as the document writes them>.
```

- Directories go in the order the spec lists them, one `##` section each, its findings after
  its five summary lines. Finding numbers run through the whole note from 1, never reused.
- **Text** follows `notes/common.md`, "What text added outside `docs/plan/` may say": no plan,
  no role of the lab, no name of the lab; dated. An open choice's **Text** states the choice
  and each of its options, and says it is open; a recommendation, if the source made one, is
  left out (the lab made it, not the user).
- **Held**, **Superseded** and **Not design** are evidence for the writer and the review, kept
  short: each held item names the section that holds it; "Not design" is a count with an
  example or two.
- Commit the note alone; the commit says how many directories and findings.

## K2. What is kept and what is not

For each item the request names (a task's `## Answers`, from that heading to the file's end; a
`plan.toml` `[[question]]`, its `text` and its `answer` if any; `plan.md`'s decisions; the
items of `notes/contracts.md` and `notes/common.md`; any other note the spec names), decide
one of four. Search the design for it first (`git grep -n -i '<term>' -- '*/docs/design/**'
DESIGN.md skein/docs/foundation tools/tiers/design.md`, then read the section a hit is in).

- **Held:** a design document, a status file, a draft, a draft's status, `DESIGN.md`,
  `skein/docs/foundation/` or `tools/tiers/design.md` states it, or a rule that covers it. The
  design held it if it holds it now: who wrote it there does not matter.
- **Superseded:** the design or the code now says otherwise (a later decision), or what it
  decided about is gone.
- **Not design:** how the work was carried out, which fab keeps in its own copies: a task's
  scope, files and order; which tests a run ran, its commands, its machine, its minutes; a
  run's budget or timeout as a run's setting; branch, merge, slot, gate and repair mechanics;
  what an implementer reports or asks. And a code shape the code now carries as decided
  (a type's fields, a signature, a module layout, a test's name or world), where no rule of
  behaviour, format or bound rides on it. These are counted, not listed.
- **Kept:** the rest, each a finding: a decision about the product (a behaviour, a format, a
  bound, a refusal, words a person reads, who owns what between crates or trees, the reason
  for a design choice), a contract that crosses a boundary (a wire or stored form, a protocol
  rule, a test world's rule that tests rely on), or an open choice (a question with options
  left open, or answered "later" or by a default taken while the user was away) whose subject
  is still open in the design and the code.

When unsure between Held and Kept, it is Kept, with **Checked** saying what was found; the
writer decides. When unsure between Not design and Kept, it is Kept if a person or another
crate would see it, Not design otherwise.

A finding whose subject belongs to `AGENTS.md`, `DESIGN.md` or `skein/docs/foundation/` takes
**Target** "none: `<file>`, the user's": `keep-decisions` writes it nowhere and reports it.

## K3. What `keep-decisions` does with a finding

Each finding ends in one of four, by its number, in the writer's report and commit:

- **written:** at its target, in the target's words (the writer may reword it, shorten it, or
  place it in a nearer section, and says so);
- **merged:** the same fact as another finding (another group's included), written once,
  naming both numbers;
- **skipped:** held or superseded at the writer's base after all, naming where;
- **left:** target none (the user's), or a target the writer may not change; reported under
  `## Left` with the finding's text.
