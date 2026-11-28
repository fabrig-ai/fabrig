# Formats of the files a person writes

Draft, 2026-11-27, from a session with the user that day. On the ways draft's `.temper/` files, the user asked that their format be versioned, "so whatever we decide now can keep being supported even if we change things. And the same should be true of all our user-facing configs (I guess temper.toml & co), if it's not already the case". It isn't: `temper.toml` names no format, and its one move so far refused the old file by name (`../protocol/configuration/file.md`, section 9). This draft gives every file a person writes for temper a format, and keeps every format temper ships readable.

## 1. In one page

- **Every file a person writes names its format** (section 3): `format = N` in TOML, `"format": N` in JSON, in a Markdown file's front matter. A file without one is format 1, so every file written before this lands reads as it does today.
- **A format moves only when an older file can't be read as the newer one** (section 4): a key renamed, reshaped or moved elsewhere. Adding a key with a default never moves it.
- **Every format shipped keeps reading** (section 5): an older file is converted as it is read, and the start says what it would write today. Nothing an older file says is dropped in silence.
- **A newer format than the binary knows is refused,** naming the file, its format and the newest the binary reads (section 6).
- **A person sees and rewrites** (section 7): help shows the newest format and each move, and each binary prints a file at the newest format for a person to write. Nothing rewrites a person's file by itself.
- **The files** (section 8): each binary's `temper.toml`, the settings files `temper ctl settings apply` and `propose` read, with their charter files, and a repository's `.temper/`.

## 2. Today

- **`temper.toml`** is read strictly through one table in code (`../protocol/configuration/file.md`, sections 2 and 5): an unknown key refuses the start, naming its line. `temper.json`'s members that no key replaces are refused with words saying where they went (`file.md`, 3.3). The format has no version.
- **A settings file** is JSON, `{"items": [...]}`, each item a setting's value, an edit or a charter's overlay, its text given literally or by directories of charter files (`../client/ctl/tools-acts.md`, 1; `../client/ctl/overlays.md`). Its items have the store's shapes of the release that reads them, and no version.
- **The store and the wires** convert values at an older layout as they read them, and keep no code for a layout older than the deployment runs ([archive/layouts-both-ways.md](archive/layouts-both-ways.md)). A person's file isn't the store's: it may sit unread for a year, in a repository or a directory temper doesn't control.

## 3. A file's format

- **A whole number from 1,** at the file's top: `format = 2` as `temper.toml`'s first key, or anywhere in its top table; `"format": 2` as a settings file's member beside `items`; `format = 2` in a Markdown file's `+++` front matter.
- **Absent is 1.** No file written today needs a change.
- **One number per kind of file,** not per binary or per release: `temper.toml`'s formats are shared by the three binaries' shapes, each reading only its own keys (`file.md`, section 1). A release that moves no format changes no file.

## 4. When a format moves

- **Only for what an older file can't say as the newer one would:** a key renamed; a value's form or meaning changed; a group made a table or a table flattened; a key whose value moved out of the file, into settings or the store.
- **Never for an addition:** a new key with a default, or a new value of an existing key's form, keeps the format. An older file lacks it and takes the default.
- **Each move is in the table:** the key table of code (`file.md`, section 5), and its counterpart for settings items, record for each key the format it came in, and each later format's change to it: renamed from, reshaped by a named conversion, moved out with its words. One table still declares every key.

## 5. Reading an older format

- **Converted as it is read,** key by key, to the newest format, then checked and used as a newest file is. A conversion is code, named in the table, and tested (section 9).
- **Said at the start:** one line per converted key, `temper.toml:LINE: KEY: format 1; format 2 writes it as WORDS`, on stderr and in the log. The binary starts.
- **A key whose value moved out of the file** seeds its new home once, where that home has no value yet, and the start says so. Later starts say the key is now ignored and where its value lives, `moved: WORDS` as today's moved names do, and still start. Its value never silently overrides the new home's.
- **A key whose feature is gone** is ignored, and the start says so, naming the release that took it away.
- **An unknown key** is still refused, at every format, naming its line: a misspelt key is no older key.

## 6. A newer format

- **Refused by name:** `temper.toml: format 3; this temper-engine reads formats 1 to 2: upgrade it, or write the file at format 2`. The binary doesn't start, since it can't know what a newer key means.
- **The same for every file:** a settings file at a newer format is refused before anything is applied, and a repository's `.temper/` at a newer format keeps its last good revision ([ways-temper-dir.md](ways-temper-dir.md), 5 and 8).

## 7. Seen and rewritten

- **Help and the reference page** show the newest format, rendered from the table as today (`file.md`, section 7), with a section listing each format's moves, key by key.
- **Printing a file at the newest format:** `temper-box config upgrade`, `temper-engine config upgrade` and `temper-worker config upgrade` read the binary's file and print it at the newest format, rendered as `init`'s file is, for a person to write. `temper ctl settings upgrade FILE` does the same for a settings file. None writes a file.
- **Comments aren't kept:** the printed file is rendered from values, so a person carries their comments over. The line notices of section 5 say what changed, so a person can edit in place instead.
- **A repository's files** are rewritten by a change Ask temper proposes after an upgrade ([ways.md](ways.md), section 11), never in place.

## 8. The files

| File | Its format's place | Reader |
|---|---|---|
| `temper.toml`, each binary's | `format`, its top table | the binary's start and every command that reads it (`file.md`, section 6) |
| a settings file | `"format"`, beside `items` | `temper ctl settings apply`, `propose` and `upgrade` |
| a charter file a settings file names | `format`, its front matter | the same, with its settings file |
| `.temper/ways.toml`, `ways/*.toml`, `hooks/config.toml` | `format`, its top table | the engine's read of `.temper/` ([ways-temper-dir.md](ways-temper-dir.md), 5 and 8); the worker for `hooks/config.toml` ([hooks.md](hooks.md), 3.4) |
| `.temper/charters/*.md` | `format`, its front matter | the engine's read of `.temper/` |

- **A settings file's items** are read at their format's shapes: each format names the kinds' shapes it was written for, and an item at an older shape is converted as the store converts a value it reads ([archive/layouts-both-ways.md](archive/layouts-both-ways.md)), from the shapes the release keeps. So a settings file kept in a repository, as fabrig's charter sets are, keeps applying after a release that reshapes a kind.
- **What isn't a person's file:** `temper ctl`'s directory, the store, the secrets, and what temper writes for its agents. Each has its own rule.

## 9. Testing

Focused, with fuzzy where it says.

- **Frozen fixtures:** for each format ever shipped, an example file of each shape and kind, kept in the tree and never edited after its release, a check refusing an edit. Each reads to the same start's document, or the same settings, as its rendering at the newest format.
- **Conversions, fuzzy:** for any file at an older format within its bounds, converting then rendering at the newest format and parsing that gives the same values.
- **Notices:** each converted key, moved key, seeded key and gone key prints its line; an unknown key at an older format is refused; a newer format is refused by name, before anything is read or applied.
- **Absent is 1:** every example file in the tree today, with no `format`, reads unchanged.

## 10. Upgrades

- **Nothing moves with this draft:** every file today is format 1, and this draft adds no format 2. The first move of a format comes with the change that needs it, its conversion and fixture in the same landing.
- **Older builds** never see a `format` key from a person who didn't write one. A person who writes `format = 1` for a build that predates this draft gets that build's unknown-key refusal, which the release notes say.

## 11. Open questions

- **smith's own `config.toml`** (smith's `shell/settings.md`): the same rule in smith's design, the default, since a person writes it too; or smith keeps its own.
- **How long a format keeps reading:** for as long as temper ships, the default, since a person's file may sit unread; or until a major release, which would say so a release ahead.
