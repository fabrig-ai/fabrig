# Recipes

Provisional, 2026-11-12. A recipe says how a fresh fake box reaches a
state that QA's cases start from (`../draft/landed-qa.md`, 5): a
directory in the repository, its steps in the drive session's lines
(`drive.md`, 3), its last lines the state's check, walked by
`temper-client-journeys recipe walk` from a fresh box or from a box a
place restored from an ancestor's copy. The copy itself, its archive,
its place and when it is made again are the QA tool's
(`../draft/landed-qa.md`, 5 and 9); this document gives the hash that
names it. Section 2 is the directory, 3 the format, 4 the chain's hash,
5 the walk, 6 the code, 7 its tests, 8 what is open.

## 1. In one page

- **A directory per recipe,** `temper/docs/development/qa/setups/NAME/`:
  `recipe`, `scripts.json` where it has its own, `files/` where its
  `forge` lines commit files, and `README.md` (section 2).
- **Its format:** `recipe 1`; `after NAME`, its parent, where it has
  one; its steps, drive lines of the browser, the TUI, ctl and the forge; last,
  its check, `web see` and `tui see` lines (section 3).
- **A chain of parents,** followed by `after`: at most eight, no cycle
  (3.2).
- **Its hash** is SHA-256 over the chain from its root, each recipe's
  name, `recipe`, `scripts.json` and files, so it changes with any
  parent's (section 4).
- **Its walk** is one drive session: from a fresh box, every recipe of
  the chain in turn, or from an ancestor's restored copy, that
  ancestor's check and then the recipes after it. Each recipe's steps
  start on the box's root page with no TUI running, and each check is
  read there too, so a recipe walks the same from either (section 5).
- **Its first failed line stops the walk,** naming the recipe and the
  line; a place then judges it (`../draft/landed-qa.md`, 5).

## 2. The directory

- **`NAME`:** 1 to 64 bytes of lowercase ASCII letters, digits and `-`,
  starting with a letter; the directory
  `temper/docs/development/qa/setups/NAME/`, under `docs/` as the
  journeys' are, so that a recipe lands as documentation.
- **`recipe`:** section 3's text, at most 64 KiB.
- **`scripts.json`,** where it has its own: the fake model's scripts it
  adds to its parents', in the box's `--scripts` form
  (`../protocol/setup/box.md`, 5), an object of `cued`, `asked` and
  `otherwise`, each optional. The scripts cover its own steps and,
  where its cases go on past its state, theirs
  (`../draft/landed-qa.md`, 6).
- **`files/`,** where its `forge commit` lines commit any: each file
  they commit, at its repository path under `files/`, so that the
  recipe holds all it gives (`../draft/landed-qa.md`, 5): `files/PATH`
  for a line's PATH (`drive.md`, 3.5). Each PATH a `forge commit` line
  of the recipe names is a regular file there, at most 64 KiB, and each
  file there is named by one of its lines; a file missing, past the
  bound or named by none refuses the recipe, naming it. At most 64
  files. Only the recipe's own lines read them: a child's `forge`
  lines commit from the child's `files/`.
- **`README.md`:** one paragraph, the state it reaches and the pages and
  screens it serves. The walk does not read it; `recipe list` prints its
  first sentence (5.2).
- **Nothing else:** another file in the directory refuses the recipe
  when it is read, naming the file. A copy is kept beside the places,
  never here.

## 3. The format

### 3.1 Its records

Text, one record per line, as the journeys' file (`web/journeys.md`,
3.2): its kind first, then its fields, each after one space, a word
bare, bytes in double quotes.

| Record | Fields | Where |
|---|---|---|
| `recipe` | `1`, the format's version | the first line |
| `after` | a recipe's name | the second line, or absent: its parent, whose state it starts from; absent, it starts from a fresh box |
| a step | a drive line: `web`, `tui`, `ctl` or `forge`, then its record (`drive.md`, 3.2 to 3.5) | after those, in order, none or more |
| the check | `web see TEXT` or `tui see TEXT` | last: the run of `see` lines that ends the file, one at least |

- **A person's acts alone:** no look, no `end` and no empty line; each
  refuses the recipe, naming its line (`drive.md`, 4 keeps them out of
  the kept lines too). `tui stop` is the walk's (section 5) and refused
  in a recipe.
- **The check** is the trailing run of `web see` and `tui see` lines; a
  `see` before a step that isn't one is a step, which waits. A recipe
  whose last line is not a `see` is refused, naming it.
- **At most 1024 lines.**

### 3.2 Its chain

- **Followed by `after`,** each parent read from its own directory, as
  the recipe is, and refused as it would be.
- **At most eight parents:** a ninth refuses the recipe, naming the
  recipe whose `after` names it.
- **No cycle:** a recipe that its own chain reaches again refuses,
  naming the recipe that closes it.
- **A parent missing** refuses, naming the recipe whose `after` names
  it.

So a chain is the recipe and at most eight ancestors, root first.

## 4. The chain's hash

SHA-256 over the chain from its root to the recipe, each recipe in turn
as byte strings, each preceded by its length as 8 bytes, most
significant first: its name, its `recipe`'s bytes, its
`scripts.json`'s bytes, empty where it has none, then the number of
its files (none where it has no `files/`) as 8 bytes, and for each file
of `files/`, in the byte order of their paths, its path relative to
`files/` and its bytes. Written as 64 lowercase hexadecimal digits.

- **It changes with any parent's** recipe, scripts, files or name, so
  that a copy named by it is never restored for a chain that differs
  (`../draft/landed-qa.md`, 5 and 11). `README.md` is not hashed: a
  reworded paragraph makes no copy again.
- **The copy's name** is the QA tool's: this hash and the commit whose
  build made it (`../draft/landed-qa.md`, 5).

## 5. The walk

### 5.1 `recipe walk`

```text
temper-client-journeys recipe walk NAME --bin DIR --bundle DIR --scratch DIR [--from ANCESTOR] [--strict] [--ports LIST] [--setups DIR]
```

- **The chain read first:** a refusal of section 3 comes before
  anything starts, naming the recipe and its line.
- **`--setups DIR`:** the recipes' directory; by default
  `temper/docs/development/qa/setups` under the working directory, as a
  run from the repository's root finds it.
- **`--from ANCESTOR`:** NAME or one of its ancestors, whose copy the
  scratch directory's box holds, restored there by the place
  (`../draft/landed-qa.md`, 5). Without it, the scratch directory holds
  no box, no `config/` and no `data/`, or the walk refuses, naming them;
  with it, it holds them, or the walk refuses.
- **One drive session** (`drive.md`, 2) on `--bin`, `--bundle` and
  `--scratch`, its box given each `scripts.json` of the whole chain,
  root first, as `--scripts`, and `--strict` and `--ports` as given: on
  a pass's box `--strict` is (`../draft/landed-qa.md`, 6), and a place
  with ports gives them (`../testing/qa/places.md`, 2).
- **Its lines,** given to the session in this order, with the chain's
  recipes R1 to Rk, root first:
  - from a fresh box: for each Ri, its steps, then `web reload "/"` and
    `tui stop`, then its check;
  - from Rj's copy: Rj's check, then for each Ri after Rj, its steps,
    then `web reload "/"` and `tui stop`, then its check.

  The session starts on the root page with no TUI (`drive.md`, 2.3), so
  each recipe's steps start there, and each check is read there, as a
  restored copy shows it.
- **Each recipe's files:** a `forge commit` line reads its PATHs from
  the `files/` of the recipe that holds the line (section 2), never
  from another recipe's or a `--files` directory.
- **Its output:** each answer of the session, every line of it prefixed
  `NAME:N `, the recipe and the line of its file that gave it, or
  `walk:` for the walk's own `web reload "/"` and `tui stop`.
- **Stopped at the first failed line:** the session ended at once, its
  box stopped (`drive.md`, 2.4), the last stdout line `stopped at
  NAME:N: WHY`, exit 5. Whether the recipe is the one to follow or
  temper failed is the place's to judge (`../draft/landed-qa.md`, 5),
  from the line and what the session answered.
- **Walked:** every line `ok`, the box stopped, the last stdout line
  `walked NAME HASH`, exit 0, the scratch directory's box a copy of
  NAME's state, ready for the place to save.
- **The session's own statuses** pass through: 3 when the store is
  refused by name, 4 when the box failed, 1 for its own failures
  (`drive.md`, 2.5).

### 5.2 `recipe hash` and `recipe list`

- **`recipe hash NAME [--setups DIR]`** reads the chain as `walk` does
  and prints its hash (section 4), exit 0, or the refusal and exit 1.
- **`recipe list [--setups DIR]`** prints a line per recipe, in name
  order: `NAME`, then ` (after PARENT)` where it has one, then `: ` and
  its README's first sentence. It is the setups' catalogue that triage
  reads (`../draft/landed-qa.md`, 4). A recipe that does not read is
  listed as `NAME: refused: WHY`, and the list goes on.

## 6. Its code

- **`temper-client-journeys-tool`'s `recipe` module,** public, beside
  `drive` and sharing its `line` grammar (`drive.md`, 5), so that the
  QA tool links it (`../draft/landed-qa.md`, 2):
  - `recipe::Chain::read(setups: &Path, name: &str) -> Result<Chain,
    Refusal>`, the chain read and checked, root first;
  - `Chain::hash(&self) -> [u8; 32]`, section 4's;
  - `Chain::names(&self)`, the recipes' names, root first;
  - `Chain::lines(&self, from: Option<&str>)`, the walk's lines in
    5.1's order, each with the recipe and the file's line it came from.

  `Refusal` names the recipe, the line where there is one, and why.
- **Pure over the files:** the reading and the hash take the files'
  bytes from a small reader, so their tests need no directory; the walk
  is the drive session's logic given those lines.
- **SHA-256** is the workspace's own implementation, never a new
  dependency.

## 7. Its tests

Focused, with no browser and no box:

- **The chain read and walked:** a chain of three, read root first; its
  lines from a fresh box and from the middle recipe's copy, in 5.1's
  order.
- **Refused:** a cycle of two, a ninth parent, a missing parent, a
  recipe that ends on a step, a look, an `end`, a `tui stop`, a file
  beyond the four, a name out of bounds; a `forge commit` PATH missing
  from `files/`, and a file of `files/` no line names.
- **Its hash:** changed by a parent's `recipe`, by a parent's
  `scripts.json`, by a parent's file's bytes or path, and by a name,
  and unchanged by a `README.md`.
- **The walk's plan:** the box given each `scripts.json` of the chain,
  root first; each recipe's `forge commit` given its own `files/`; a
  failed line stopping the walk with exit 5, naming the recipe and its
  line.

## 8. Open questions

- **The catalogue as a file:** `recipe list` prints it from the recipes.
  Default: printed, with no file to keep in step.
- **The bound of eight:** a twin's chain (`../draft/landed-qa.md`, 6)
  may want more. Default: eight, raised when a twin's recipe needs it.
