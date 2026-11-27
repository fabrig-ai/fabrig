# The format of a file a person writes

Provisional, 2026-11-27, from `../../draft/formats.md`
(`../../draft/formats.execution.md`, V1). Every file a person writes for
temper names its format, a whole number; a format moves only when an
older file can't be read as the newer one; and a release reads every
format temper has shipped, converting an older file as it reads it and
refusing a newer one by name. This document holds the rule every such
file follows; each file's own document says its place, its words and
its tests: `file.md` for `temper.toml`, `../../client/ctl/tools-acts.md`,
1, for a settings file, and `../../client/ctl/overlays.md`, 2, for a
charter file. A file's format here is its number, not the rule for
members' JSON of `README.md`, section 8.

## 1. In one page

- **Every file a person writes names its format** (section 2), `format`
  at its top; a file without one is format 1, so every file written
  before this design reads as it did.
- **A format moves only for what an older file can't say** (section 3):
  a key renamed, reshaped, moved out of the file or gone. An addition
  with a default never moves it. Each move is a record of its file's
  table of moves, with its code named there.
- **An older format is converted as it is read** (section 4), key by
  key, each key a move touched said in one line on stderr, then checked
  and used as a file at the newest format. Nothing it says is dropped in
  silence, and an unknown key is refused at every format.
- **A newer format is refused by name** (section 5), before anything is
  read further or applied.
- **A person sees and rewrites** (section 6): help shows the newest
  format and each move, and `config upgrade` and `settings upgrade`
  print a file at the newest format. Nothing rewrites a person's file.
- **Frozen fixtures** (section 7): for each format shipped, example
  files kept in the tree and never edited, a check refusing an edit.
- **Nothing moves now** (section 9): every kind's newest format is 1.

## 2. A file's format

| Kind of file | Its format's place | Newest | Reader | Its document |
|---|---|---|---|---|
| `temper.toml`, each binary's | `format`, a key of its top table | 1 | every command that reads it | `file.md` |
| a settings file | `"format"`, a member beside `items` | 1 | `temper ctl settings apply`, `propose` and `upgrade` | `../../client/ctl/tools-acts.md`, 1 |
| a charter file a settings file names | `format = N`, the one line of its `+++` front matter | 1 | the same, with its settings file | `../../client/ctl/overlays.md`, 2 |
| `.temper/`'s files | `../../draft/ways.md`, 8.5 and 8.8 | — | the engine's and the worker's reads of `.temper/` | designed with them (`../../draft/ways.execution.md`, W2) |

- **A whole number from 1,** in decimal. Anything else in the place is
  refused alone, naming the file and `format`, since no other key can
  be judged without it: `temper.toml:3: format: a whole number from 1`.
- **Absent is 1.** No file written before this design needs a change.
- **One number per kind of file,** not per binary or per release:
  `temper.toml`'s formats are shared by the three binaries' shapes, each
  reading its own keys (`file.md`, section 1), and a charter file's
  format is its own, not its settings file's. A release that moves no
  format changes no file.
- **Written only above 1:** a renderer writes `format` only where the
  newest format is above 1, so every file printed by this release,
  `init`'s among them, names none, and a build that predates formats
  never meets a `format` its person didn't write.

## 3. When a format moves

- **Only for what an older file can't say as the newer one would:** a
  key renamed; a value's form or meaning changed; a group made a table
  or a table flattened; a key whose value moved out of the file, into
  settings or the store; a key whose feature is gone.
- **Never for an addition:** a new key with a default, or a new value
  of an existing key's form, keeps the format. An older file lacks it
  and takes the default.
- **One format per landing that moves it:** the moves one landing makes
  together are the next format, the newest plus 1, with their records,
  their code and that format's fixtures (section 7).
- **Each move is a record** of its file's table of moves, beside the
  table that declares its keys (`file.md`, section 5; for a settings
  file, `FIELDS`' shape and member, `../../client/ctl/tools-acts.md`,
  1): the format the move made, the key's name in the format before it,
  and one of:

  | Move | It records | An older file's key is | Its notice, after `KEY: ` |
  |---|---|---|---|
  | renamed | its new name | read at its new name | `format F; format N writes it as NEW` |
  | converted | its name at the new format, a conversion, and words | converted by the conversion, then read at that name | `format F; format N writes it as WORDS` |
  | moved out | its home, a seeding, and words saying where its value lives | handed to its home's seeding (section 4) | `moved: WORDS; this start seeded it`, or `ignored; moved: WORDS` |
  | gone | the release that took it away | ignored | `ignored; RELEASE took it away` |

  F is the file's format and N the newest. A key's format is the format
  of the move that made it, or 1: a key added with a default makes no
  move, so it records none.
- **A conversion and a seeding are named code** in the reader's crate,
  each written and tested in the landing that makes its move. One table
  still declares every key at the newest format; a reader keeps each
  move for as long as temper ships (section 10).

## 4. Reading an older format

In order, for every file of section 2:

1. **Parsed, its format read** (section 2). A newer format is refused
   (section 5).
2. **Moved to the newest format:** at an older format, each move of
   each later format applies in the order of its format, key by key,
   so a file at format 1 read by a release at format 3 takes format 2's
   moves, then format 3's.
3. **Checked and used** as a file at the newest format, by its table.
   An unknown key is still refused, at every format, naming its line: a
   misspelt key is no older key, and a name a move took away (renamed
   from, converted from, moved out, gone) is unknown in a file at that
   move's format or later.
4. **Noted:** one line per key a move touched, in its file's form
   (`temper.toml:LINE: KEY: WORDS`; `settings file item N: PATH:
   WORDS`), WORDS section 3's notice, on stderr before the command goes
   on. The command goes on: a start starts, and a call is sent.

- **A key whose value moved out** is handed, with its value and line,
  to its home's seeding, which the start runs once the home can be
  read: where the home has no value yet, the key's value seeds it, and
  the start says so; at every later start the home has one, and the
  start says the key is ignored and where its value lives. The key's
  value never overrides its home's. A command that doesn't open the
  home, `config upgrade` among them, notes `moved: WORDS` alone.
- **A home's seeding comes with its first key:** what a home is (a
  setting, a record of the store), how a start reads whether it has a
  value and how it writes one are that home's, written in the landing
  that moves the first key there. This release moves no key, so it has
  none.
- **Nothing an older file says is dropped in silence:** each key is read
  at the newest format, moved with its notice, or refused.

## 5. A newer format

- **Refused by name,** with nothing read further or applied, since no
  release can know what a newer key means:
  `temper.toml: format 3; this temper-engine reads formats 1 to 2:
  upgrade it, or write the file at format 2`, the binary's own name;
  where a reader reads format 1 alone, `reads format 1`, and `write the
  file at format 1`. The binary doesn't start.
- **The same for every file:** a settings file at a newer format, or
  one naming a charter file at a newer format, is refused before
  anything is sent (`../../client/ctl/tools-acts.md`, 1;
  `../../client/ctl/overlays.md`, 2); a repository's `.temper/` at a
  newer format keeps its last good revision (`../../draft/ways.md`, 8.5
  and 8.8).

## 6. Printing a file at the newest format

- **`temper-box config upgrade`, `temper-engine config upgrade` and
  `temper-worker config upgrade`** (`file.md`, section 7), and **`temper
  ctl DIRECTORY settings upgrade FILE`**
  (`../../client/ctl/tools-acts.md`, 1), read a file as their file's
  commands read it, print its notices on stderr and the file at the
  newest format on stdout, and exit 0. A file the read refuses prints
  its refusal as the file's other commands print it, and exits 1.
- **Rendered from its values,** as `init`'s file is, `format` written
  only above 1 (section 2): a person's comments aren't kept, so a person
  carries them over, or edits the file in place by the notices, which
  say what changed.
- **None writes a file.** A person writes the printed one. A key moved
  out of the file isn't printed: its notice says where its value lives.
- **A repository's files** are rewritten by a change Ask temper
  proposes after an upgrade (`../../draft/ways.md`, section 11), never
  in place.

## 7. Frozen fixtures

- **Where:** `temper/docs/examples/formats/`, a directory per kind of
  file and format: `temper-toml/1/`, `settings/1/` and `charters/1/`.
  The landing that makes a format adds its directories.
- **What each format's holds:** for `temper.toml`, one file per shape,
  `box.toml`, `engine.toml` and `worker.toml`, each holding every public
  key its shape reads at that format (`file.md`, 3.1), internal and test
  keys being no person's; for a settings file, files that together hold
  an item of every kind a settings file sets, an edit `removed` and
  `reset`, and an overlay of each form, one naming the charter files of
  the same format as `../../charters/F`; for charter files, a set of the
  five names (`../../client/ctl/overlays.md`, 2). Format 1's name no
  `format`, as no file written before this design does.
- **Never edited after its release:** a focused check in
  `tests/boundary/` refuses a tree in which a file under
  `temper/docs/examples/formats/` that its base holds is changed or
  gone, comparing bytes; a new file or directory passes. Its base is the
  landing's, found as the host tools' contracts check finds it
  (`tests/boundary/tests/host_tools.rs`). A fixture found wrong after
  its release stays: the next format's fixtures show the right form.
- **Each is read by its reader's tests** (section 8).

## 8. Testing

Focused, with fuzzy where it says. Each file's document names its tests.

- **Frozen fixtures:** each fixture of each format reads, with its
  notices, to the same start's document, or the same settings items,
  as its rendering at the newest format does.
- **Moves:** each reader is also tested at a table of moves of its own,
  a format 2 over the production keys holding one move of each kind,
  since the production table holds none: each moved key prints its
  line; the name a move took is unknown at format 2; an unknown key at
  format 1 is refused.
- **Conversions, fuzzy:** for any file at format 1 of that test table
  within its bounds, converting then rendering at the newest format and
  reading that gives the same values.
- **A newer format** is refused by name, before any key is checked or
  anything is applied; `format` that isn't a whole number from 1 is
  refused alone.
- **Absent is 1:** every example file in the tree today, with no
  `format`, reads unchanged, and one naming `format = 1` reads as the
  same file without it.
- **The check** (section 7).

## 9. Upgrades

- **Nothing moves:** every kind's newest format is 1, the format of
  every file written before this design. No stored or wire format, and
  no version, moves.
- **Older builds:** a build that predates this design refuses `format`
  as an unknown key or member. Nothing this release writes names it, and
  its release notes say that a file naming `format = 1` is refused by a
  build before it.
- **The first move** of a format comes with the change that needs it:
  its records, their named code and the new format's fixtures, in one
  landing.

## 10. Open questions

- **smith's own `config.toml`** (smith's `shell/settings.md`): the same
  rule in smith's design, since a person writes it too, the default; or
  smith keeps its own (`../../draft/formats.md`, section 11).
- **How long a format keeps reading:** for as long as temper ships, the
  default, since a person's file may sit unread; or until a major
  release, which would say so a release ahead.
