# A charter's overlay in ctl

How `temper ctl settings apply` takes a charter's overlay from a settings
file, its text from directories of charter files, several files stacked
in one call, and how `settings show` prints one
(`../../draft/stock-charters.md`, section 4). The overlay itself, a
setting's value kept over a charter's default, is jig's
`domain/engine/overlays.md`; the settings file's other items are
`tools-acts.md`, 1; a set of charter files and its assembly,
`../../domain/agent/charter.md`, 1.

## 1. An overlay item

- **Its form:** `{"setting": {"kind": "charter", "number": N}, "overlay":
  OVERLAY}`, beside the file's other items (`tools-acts.md`, 1), at most one
  item per setting in a file. OVERLAY is an object of two kinds of
  member:
  - *the fields it replaces:* any member of a charter's VALUE
    (`tools-acts.md`, 1; `FIELDS`' `charter` rows) but `number`,
    `instructions`, `parts` and `wake`, each in the form that VALUE
    gives it and each optional: a member named replaces the default's
    whole, a member left out follows it (jig's
    `domain/engine/overlays.md`, 1). So `"shell": true` turns the shell
    on and `"documents": []` leaves none;
  - *the text it adds:* either `charters`, a nonempty list of
    directories, each a set of charter files (2), or the text itself,
    `instructions`, a string, and `parts`, a list in a charter's `parts`
    form (`tools-acts.md`, 1), each optional and empty when left out. An item
    naming `charters` beside `instructions` or `parts` is refused, naming
    the item: `settings file item 2: overlay: charters or instructions
    and parts, not both`.
- **An example,** the implementer's from a set beside the file:
  `{"setting": {"kind": "charter", "number": 2}, "overlay": {"shell":
  true, "agents": false, "charters": ["."]}}`.
- **Sent as one value:** the call's overlays of one charter, joined (3),
  become one item *set* to the overlay (jig's
  `domain/engine/overlays.md`, 2), its added text literal. The decoder's
  refusals are the file's (`tools-acts.md`, 1): members unknown or of the
  wrong type, a part's reasons, a part on a charter whose runs the
  overlay or its default resume where the client knows it; the charter
  the overlay makes is checked whole by the engine, whose refusal names
  the charter as for a charter whole.
- **`FIELDS`:** an `overlay` shape, its replaced fields resolving to the
  `charter` rows they name, and a row for `overlay.charters`; the
  `settings-fields` page renders it as it renders every shape
  (`tools-acts.md`, 1).

## 2. Its directories

- **Where a directory is:** relative to the directory of the file that
  names it; a directory may be absolute anywhere. A file read from
  standard input (`-`) has no directory of its own: a relative directory
  in it is refused, naming the item and the directory, and nothing is
  sent: `settings file item 1: overlay.charters[0]: "fabrig" is
  relative, and standard input has no directory; name it absolutely`.
- **What is read:** each Markdown file in the directory, by name, not
  below it; a file of another extension, the settings file among them,
  is no part of the set. A set's six names are `common`, `goal`,
  `design`, `designer`, `coordinator` and `implementer`, each optional in
  an overlay (`../../draft/stock-charters.md`, 2;
  `../../draft/any-project.md`, 4.2), so that a directory without
  `design.md` applies as it did before the set had one. A directory missing or
  unreadable, an unknown Markdown name, text in `coordinator` before its
  first section, and a section whose reasons are empty, repeated or
  unknown are refused, naming the item, the directory and the file, in
  the words of the assembly's refusals (`temper-charter-set`'s
  `Refusal`): `settings file item 1: overlay.charters[0]: notes.md:
  unknown Markdown file`.
- **A charter file's format**
  (`../../protocol/configuration/file-formats.md`): a file may open with
  front matter, three lines alone, `+++`, `format = N` and `+++`, spaces
  around `=` optional; without it, it is format 1. This client reads
  format 1 alone. The front matter is no part of the file's text. A
  newer format is refused before anything is sent, naming the item, the
  directory and the file: `settings file item 1: overlay.charters[0]:
  goal.md: format 2; this temper ctl reads format 1: upgrade it, or
  write the file at format 1`; front matter holding any other line, or
  not closed, `...: goal.md: front matter holds format = N alone`, and
  N not a whole number from 1, `...: goal.md: format: a whole number
  from 1`. Its lines are read by their form, in `temper-charter-set`,
  not by a TOML parser, so `temper-config-shell` stays the one crate
  that depends on the `toml` crate (`../../protocol/configuration/file.md`, 8). The
  stock's set (`../../domain/agent/charter.md`, 1) has none.
- **Assembled for its charter** by the rule the stock's set is
  (`../../domain/agent/charter.md`, 1): charter 5 takes `common`, `goal`,
  `design`, `designer` and `coordinator`, its instructions `common`,
  `goal` then `design` and its parts `designer`, on `first`, then each
  section of `coordinator`; charter 2 takes `common` then `implementer` as its
  instructions, with no parts. A file its charter doesn't take is read
  and checked all the same, so one directory serves both charters. An
  overlay of any other charter naming `charters` is refused, naming the
  item: the sets are charters 5 and 2's.
- **Several directories** in one item are assembled each, then joined in
  order: their instructions, each not empty, with one blank line between
  them, every byte as written but the blank lines that open and close
  each; their parts one after another.
- **Read by the client:** the command line's input step reads each
  settings file and every directory its overlay items name before it
  opens the login's directory, as it reads the file today
  (`command-line.md`, section 1), and the tools assemble them; the engine
  receives only literal text. A file's Markdown counts toward that
  file's input bound.

## 3. Files stacked in one call

- **`settings apply FILE... --against REVISION [--running stays|moves]`**
  takes one or more files, `-` at most once, decided as one change
  (`tools-acts.md`, 1). `settings propose FILE [REASON]` takes overlay items
  too, from its one file.
- **Overlays of one charter are joined** into one: its text the files'
  in the order given, joined as directories are (2); its fields each
  file's over the earlier files', member by member, so a later file's
  `model` wins and an earlier file's `shell` stands where the later
  names none. So a deployment adds lines of its own over a public set
  without copying it: `settings apply fabrig/settings.json
  ours/settings.json` lays both sets' text over the stock's, fabrig's
  first.
- **Every other setting once:** a setting another file names too, or a
  set, removal or reset of a charter beside an overlay of it in any file
  of the call, is refused before anything is sent, naming both files
  and items: `settings apply: charter 5: set in b.json item 1 beside an
  overlay in a.json item 2`.
- **Refusals name the file:** with several files, a decoder's refusal
  names its file before its item, `settings file b.json item 2: ...`;
  with one, the words stay `tools-acts.md`, 1's. jig's refusal of the change
  numbers the joined items as sent, each overlay at its first file's
  place.

## 4. Showing an overlay

- **`settings show charter N`** prints the charter as its runs read it,
  the value and a line per part as `tools-acts.md`, 1 has them, and, where an
  overlay stands over it, marks what the overlay replaced and added: a
  line naming the members it replaces in `FIELDS`' order, `overlay
  replaces: model, shell, turns`, or `overlay replaces: nothing`; a line
  for its instructions, `overlay adds: 1840 bytes of instructions` or
  `overlay adds: no instructions`; and each of its parts' lines ending
  `; overlay`, `parts[4]: on first; 312 bytes; overlay`.
- **The inventory,** `settings show`, gives an overlaid charter's origin
  as `overlay`, beside `default` and `change`.
- **With `--json`:** `{"setting": SETTING, "revision": R, "value":
  CHARTER, "overlay": OVERLAY}`, CHARTER the charter as its runs read it
  and OVERLAY the overlay whole in its item's literal form: the members
  it replaces, `instructions` where it has some and `parts` where it has
  any, never `charters`, naming no directory. So `{"items": [{"setting":
  SETTING, "overlay": OVERLAY}]}` applies it back anywhere, as an
  overlay that takes a later default. A charter with no overlay prints
  no `overlay` member.
- **A proposal and a revision's items** print an overlay item in its
  item's literal form, as `--json` prints it.

## 5. Open questions

- **Stacking a proposal:** `settings propose` takes one file, its reason
  a positional argument; a deployment proposing two sets at once joins
  them by hand. Default: one file.
- **The browser's cards:** a proposal card shows an overlay item with no
  detail of its fields or text, the pages being outside this draft.
  Default: unchanged.
