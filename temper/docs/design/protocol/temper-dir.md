# A repository's `.temper/`

Provisional, 2026-11-28, from `../draft/ways-temper-dir.md`, sections 1,
6 and 8 (`../draft/ways.execution.md`, W2). What temper reads of a
repository's `.temper/` and the one judgement it makes of it: its
entries, each file's form and format, names, links and home. The forge
makes it of the files it reads at one commit, both for the project's
repository revision and for the check before a change lands, as the
forge's design says. What a way's tasks and a charter's
settings say is read where their own designs say, and adds its checks
to this judgement (`../draft/ways-recipes.md`;
`../draft/ways-charters.md`).

## 1. In one page

- **Temper's method files** are `ways.toml`, `ways/` and `charters/`;
  with `hooks/` they are what temper reads of `.temper/` (2). Anything
  else there is neither read nor judged.
- **Each file has a form** (3): `ways.toml` and a way's file are TOML,
  read by `skein-toml` (`skein/docs/design/data/toml.md`); a charter's
  file is Markdown with optional front matter; `hooks/config.toml` is
  the hooks' own subset (`hooks.md`, 2.3).
- **Names** (4) are jig's: 1 to 32 bytes of lowercase letters, digits
  and `-`, starting with a letter. A way is `ways/NAME.toml`, a charter
  `.temper/charters/NAME.md`.
- **Links** (5): `offer` names files that exist and `[default]` names
  offered ways; a symbolic link stands only at a hook's name.
- **Home** (6): the method files sit in the project's home repository
  alone.
- **Formats** (7): each file's `format`, absent 1; every shipped format
  reads, and a newer one is refused by name.
- **A charter's file** (3.3): a front matter naming its `format` and
  its `settings` charter, then its instructions and its parts, words
  alone.
- **One judgement** (8): the files of one commit and the deployment's
  charters in, the tree's faults out, each `FILE:LINE: why`, in a fixed
  order.

## 2. What temper reads

| Entry | It must be | Read |
|---|---|---|
| `.temper/ways.toml` | a regular file | its bytes (3.1) |
| `.temper/ways/` | a directory of `NAME.toml` regular files | each file's bytes (3.2) |
| `.temper/charters/` | a directory of `.temper/charters/NAME.md` regular files | each file's bytes (3.3) |
| `.temper/hooks/` | a directory | its listing; `config.toml`'s bytes (3.4) |

- **A tree holds temper's method** where `.temper/` holds `ways.toml`,
  `ways/` or `charters/`, as any kind of entry: the judgement says so
  (8), and the forge's design says what follows from it.
- **`ways/` and `charters/` are temper's alone:** an entry there that
  isn't a regular file named `.temper/ways/NAME.toml` or `.temper/charters/NAME.md` is a fault, a
  directory, a link or a submodule among them.
- **The rest of `.temper/` is not judged:** `hooks/`' other entries,
  `pre-pr`, a `README.md`, any other file or directory. Hooks are
  checked where they run (`hooks.md`, 2.2 and 4.3).
- **A listing names kinds** (skein's `forge/vocabulary.md`, 1.3): a
  file, a link, a directory or a submodule. An entry of the table above
  of another kind than it must be is a fault, naming its path and what
  it is: ``.temper/ways.toml: a symbolic link; temper reads a regular
  file here``.

## 3. The files

### 3.1 `ways.toml`

TOML, at most 65,536 bytes, its keys at format 1:

| Key | Its value | Absent |
|---|---|---|
| `format` | an integer from 1 (7) | 1 |
| `offer` | an array of names (4), at least one, none twice | a fault |
| `default` | a table: each key a start command, `chat`, `change`, `goal` or `milestone`; each value a name `offer` holds | no defaults |

- **An unknown key is a fault,** at any depth, naming its line:
  ``.temper/ways.toml:4: unknown key `defaults` ``. A key a later design
  adds reads from the build that adds it (`../draft/ways.md`, 5.3), and
  an older build refuses it, as `configuration/file-formats.md`, 4 has
  it for every file.
- **A value of another type** is a fault naming its key and what it
  must be: ``.temper/ways.toml:2: `offer`: an array of way names``.

### 3.2 A way's file

`ways/NAME.toml`: TOML, at most 65,536 bytes. Its `format` is judged
(7); its other keys are its reader's (`../draft/ways-recipes.md`), which
judges them as it reads them. A way's file `offer` doesn't name is
judged as one it names, and offers nothing.

### 3.3 A charter's file

`.temper/charters/NAME.md`: UTF-8, at most 65,536 bytes: a front matter
or none, then the charter's instructions, then its parts, the project's
charter NAME whole (`../domain/agent/project-charters.md`, 3). One
reader serves every caller, `temper-charter-set`'s, called for one
file's text: this judgement, the root's set
(`../domain/agent/project-charters.md`, 4) and `temper ctl charter
copy` (`../client/ctl/tools-reads.md`, 1). A settings file's overlay
files are read by it too, taking `format` alone
(`../client/ctl/overlays.md`, 2).

- **The front matter:** from a first line `+++` to the next line `+++`,
  TOML read by `skein-toml`, at format 1:

  | Key | Its value | Absent |
  |---|---|---|
  | `format` | a whole number from 1 (7) | 1 |
  | `settings` | a name (4): the deployment's charter whose settings it runs by, its models, tools, hosts, waiting, resume and budget | the file's own name |

  Any other key is a fault naming its line: ``.temper/charters/careful.md:3:
  unknown key `model`: a charter's file holds words alone; models,
  tools, wakes and budgets are settings``. A first line `+++` with no
  closing one is ``FILE:1: front matter: no closing `+++` ``; a value of
  another type names its key and what it must be, as in `ways.toml`
  (3.1). An overlay's file naming `settings` is refused at apply,
  naming its line: ``…: `settings` is a project's charter file's key``.
- **The instructions:** the text after the front matter up to the first
  part's heading, or to the file's end where it has none.
- **The parts:** each from a line beginning `## part, on:` to the next
  such line or the file's end, other headings included; its reasons the
  rest of that line, split at commas, each trimmed of spaces, named as
  jig names a part's reasons (`jig/docs/design/domain/tasks/messages.md`,
  6). The heading line is no part of its text. A heading whose reasons
  are none, hold an empty name between commas, repeat one, or name one
  jig doesn't, is a fault at its line: ``.temper/charters/lead.md:40:
  `## part, on: frist`: `frist` is no reason a part is selected by``.
- **Texts** keep every byte as written but the line feeds that open and
  close the instructions and each part, as the stock's set joins them
  (`../domain/agent/charter.md`, 1). Empty instructions are none; a part
  is a part whatever its text.
- **Written back the same:** a charter written in this form, no front
  matter, its instructions, then for each part a blank line, its heading
  `## part, on: ` and its reasons' names joined by `, ` in jig's order,
  and its text, reads back to the same instructions and parts. A text
  holding a line that begins `## part, on:`, in its instructions or a
  part, can't be written so, and the writer refuses it, naming the
  part and the line.

### 3.4 `hooks/config.toml`

Read by the hooks' own reader (`hooks.md`, 2.3), each fault it names
this judgement's, at its line. Missing, it is silent.

## 4. Names

- **A name** is 1 to 32 bytes of lowercase ASCII letters, digits and
  `-`, starting with a letter: jig's `is_name`
  (`jig/docs/design/domain/engine/settings.md`, 1, *Names*). A way's
  name is its file's stem in `ways/`, a charter's in `charters/`.
- **Unique within its kind,** by its path: no two ways, and no two
  charters, share a name. A way and a charter may.
- **Free:** no deployment's name is reserved here (`../draft/ways.md`,
  section 9): a charter's file named as a deployment's charter is the
  project's charter of that name.
- **A fault names the entry:** ``.temper/ways/Hand-over.toml: `Hand-over`
  is no name: 1 to 32 lowercase letters, digits and `-`, from a
  letter``; in `offer` or `[default]`, its line.

## 5. Links

- **Between files:** each name in `offer` has its file,
  ``.temper/ways.toml:2: `offer` names `pair`, and `.temper/ways/pair.toml`
  is missing``; each `[default]` value is in `offer`,
  ``.temper/ways.toml:6: `default.change` names `pair`, which `offer`
  doesn't``.
- **Symbolic links:** a link stands only at a hook's name
  (`hooks.md`, 2.2); at any path of section 2's table, or inside
  `ways/` or `charters/`, it is a fault (2), and its bytes are never
  read. A hook's link is judged where the hook runs, since the forge's
  listing gives no link's target (skein's `forge/vocabulary.md`, 1.3).

## 6. Home

- **A project's method has one home:** its home repository, the one its
  path is drawn over (`../domain/forge/change/branch-path.md`, section
  1). In another of the project's repositories, each of `ways.toml`,
  `ways/` and `charters/` that `.temper/` holds is a fault:
  ``.temper/ways.toml: a project's ways live in its home repository,
  OWNER/NAME``, or, where the project has none, ``...: a project's ways
  live in its home repository, and it has none``.
- **Hooks are each repository's own:** `hooks/` is judged in every
  repository.
- **A context repository's `.temper/`** is never read.

## 7. Formats

| File | Its format's place | Newest |
|---|---|---|
| `ways.toml` | `format`, a key of its top table | 1 |
| a way's file | `format`, a key of its top table | 1 |
| a charter's file | `format = N`, its front matter's line | 1 |
| `hooks/config.toml` | `format = N`, before any header (`hooks.md`, 2.3) | 1 |

- **The rule is `configuration/file-formats.md`'s,** whole: absent is 1;
  a format moves only for what an older file can't say, with its moves'
  records; every format shipped keeps reading; temper writes `format`
  only above 1; nothing rewrites a person's file.
- **Judged before the file's keys:** a format that isn't a whole number
  from 1 is ``FILE:LINE: format: a whole number from 1``; one above the
  newest is ``FILE:LINE: format N; this temper-engine reads format 1:
  upgrade it, or write the file at format 1``, and neither judges the
  file further.
- **Longer than the store's rule:** a repository's file lives where
  temper has no say, so a format reads for as long as temper ships
  (`../draft/ways-temper-dir.md`, 8).
- **Frozen fixtures** (`configuration/file-formats.md`, 7):
  `temper/docs/examples/formats/temper-dir/1/` holds a `.temper/` at
  format 1, with `ways.toml`, two ways' files, two charters' files and
  `hooks/config.toml`, none naming `format`. Its reader's test reads it
  to a tree with no fault.

## 8. The judgement

- **What it is given:** the listing of `.temper/` and of each directory
  of section 2 it holds, each entry a name and a kind; the bytes of each
  file section 2 reads, and whether its read was cut; whether the
  repository is the project's home, with the home's name; and the
  deployment's charters, as the root last gave them to the forge (8.1).
- **What it gives:** whether the tree holds temper's method (2), and its
  faults, in order, at most 16, the count of those past 16 beside them.
  No fault is a good tree.
- **A charter's file, judged whole** (3.3, 8.1): its front matter and its
  part headings; then its settings charter, `settings` or its own name,
  one the deployment's charters hold:
  ``.temper/charters/careful.md:2: `settings` names `careful-impl`,
  which this deployment's settings don't offer``, or, with no
  `settings`, ``.temper/charters/careful.md: no `settings`, and this
  deployment offers no charter `careful` ``; then its words within the
  room that charter leaves them: ``.temper/charters/implementer.md: its
  words take 70,112 bytes and 3 parts; charter `implementer` leaves
  them 52,000 bytes and 8 parts``. Past the files, the charters' count
  within the deployment's: ``.temper/charters/: 40 charters; this
  deployment's tasks hold at most 32``.
- **Its order** is the reader's: entries before files; paths in byte
  order; in a file, its format first (7), then its lines in order, then,
  for a charter's file, its settings charter and its room; the charters'
  count after the files; home last.
- **Bounds:** at most 64 files read, each within its section's bytes,
  at most 524,288 bytes in all. A listing or a file the forge cut is a
  fault, ``FILE: past what temper reads, 65,536 bytes``; past the files'
  or the bytes' count, ``.temper/: past what temper reads: 64 files``.
- **One judgement, two callers:** the forge's read and the landing
  check give it the same input for the same commit and the same
  deployment's charters, and get the same answer.

### 8.1 The deployment's charters

- **What they are:** for each charter the newest settings offer that has
  a name (`jig/docs/design/domain/engine/settings.md`, 1, *Names*), its
  name and the room it leaves a project's words: the most bytes and the
  most parts a project's charter run by its settings may hold, so that
  the charter jig composes (`jig/docs/design/domain/engine/project-charters.md`,
  5) passes jig's bounds, `core.run_bytes` and `tasks.charter_parts`,
  and the room temper gives a received charter
  (`../domain/agent/room.md`, 4), once that charter's own fields and its
  overlay's words are counted; the words counted as jig counts a
  charter's, its instructions and each part with what joins it. Beside
  them, `tasks.charters`, the most charters a project may keep.
- **Given by the root,** which holds the settings and the limits: at its
  start, before the forge's first read, and in each decision that makes a
  settings revision. The forge keeps the last in memory, never in the
  store, and judges every tree against it.
- **A settings change after a revision** judges nothing again: the
  revision stands, and the binding refuses what no longer fits
  (`jig/docs/design/domain/engine/project-charters.md`, 6) until the next
  read.

## 9. Testing

Focused, with fuzzy where it says, in the judgement's crate.

- **Every fault:** a table of trees, one per fault of sections 2 to 8,
  each with its words and line, given a deployment's charters.
- **A charter's file:** front matter with `format` and `settings` in
  either order, and with neither; parts with one and several reasons;
  instructions alone; and the writer's text of any instructions and
  parts reading back to them, fuzzy too.
- **The fixtures** of 7 read with no fault.
- **Order and bound:** a tree with more than 16 faults names the first
  16, in order, and the count past them.
- **Fuzzy:** any tree within the bounds, of any entries, kinds and
  bytes, is judged within the bounds and the same twice; a good tree
  with one entry changed to a fault's form is judged with that fault.

## 10. Upgrades

- **No version moves:** every `.temper/` file's newest format is 1, the
  form of every file written before this design, and `hooks/config.toml`
  without `format` reads as before.
- **Older builds** refuse `format` in `hooks/config.toml`, as a key
  before any header; nothing temper writes names it
  (`configuration/file-formats.md`, 9).
- **A charter's file at format 1** gains `settings` and its parts' form
  (3.3) with no format's move: a file written before them, with no front
  matter or `format` alone and no part heading, reads to the same text,
  its instructions. A build before them refuses a front matter naming
  `settings`, as 3.1 has a key a later design adds, so the last good
  revision holds on it.

## 11. Later

- **A way's tasks** add their checks to the judgement as their design
  is written (`../draft/ways.execution.md`, W4).
- **A hook link's target,** where a forge's read gives it, judged before
  it lands.
