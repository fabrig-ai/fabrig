# The ctl bounds

This document holds ctl's bounds, split from [`README.md`](README.md).

## 1. The bounds

settings FILE (or stdin):1MiB, tools.ask_bytes; item/member/value
bounds are checked before the request is sent.

Four bounds decide how much of a document a call prints:

| Bound | Whose | It decides | Past it |
|---|---|---|---|
| a page's bytes | the engine's (jig's `client/domain/contract/documents.md`, 1) | whether a turn comes in a page by its bytes or by its size | the turn's line, its size and the call that reads it (`tools-reads.md`, 1) |
| one read's bytes | the engine's (the read of one turn, jig's `client/domain/contract/reads.md`, section 1) | whether `task turn` is answered whole | its size, and that it is past the limit |
| one document's bytes | the client's, `document_bytes` | whether the client takes a document whole | a turn: its size, and that it is past the limit; any other document: the client's `Limit`, exit 1 |
| the tools' window | the tools' | how much of a result prints | the rest by its cursor, or `--part` and `--from` (`results.md`, 4) |

- **A turn past either read bound** answers exit 0, with its size and
  "past the limit; report it": the agent reports it as a finding, and
  never works around it.
- **The tools' window is within the client's limit:** a step test fails
  when one of the tools' ceilings passes the client limit it reads from
  (`testing.md`, 1). A result's text is within one document's bytes and a page
  within a turn's. The client's text is the tools' preset's own, within
  a result's text, so any one of its words prints whole, and an act's
  words are checked against it before they are sent. A page, its
  position line and its end line fit a result's text with 512 bytes to
  spare (`results.md`, 4), and its JSON, six bytes for each source byte, a result's
  bytes.

The numbers are constants of `temper-client-tools`' `limits` module. The
client's are the tools' own preset of `jig-client`'s `Limits`, not the
terminal's, whose window shows two rows:

| What | Bound |
|---|---|
| rows: a list's page, the inbox's needs and updates together, the activity lines `task show` adds, a conversation's page; the client's rows per window | 32, the most jig's client wire takes in a limit of rows (its schemas' ceilings; jig's `protocol/client/wire.md`, section 1) |
| a result's text | 80 KiB: a page, and the 16 KiB a result keeps beside it |
| a page: a turn's part, and the source one `task text` page holds | 64 KiB, 65,536 source bytes |
| a result's bytes, as JSON | one document's bytes six times, and 1 KiB: JSON may spend six bytes on one source byte, so a page's JSON fits |
| the client's text, `jig-client`'s `text` and `turn_text`; an act's words and a model's or effort's name, checked before sending | 32 KiB. jig's wire takes at most 64 KiB of it (`task_text_words`), short of a page with its lines; `task text` joins the engine's chunks, so a page needs no more |
| one document, the client's `document_bytes` | 1 MiB |
| project memberships in `me` | 9, covering the Small and Standard deployment profiles |
| a path's segments; an authority's entries | 9; 12, as the terminal's (`../view.md`, 4) |
| the link's deadline: the engine unreachable, or a call's request or answer unheard | 30 s |
| a retry's span, a request's and a watch's: at least half of it, jittered | from 1 s, doubling to at most 2 s, the most a sign-in's link waits between tries (`command-line.md`, 1.2) |
| the lock's wait | 60 s, tried every 100 ms |
| a key's file, less a final newline (`areas.md`, area 5) | 4 KiB; past it, or empty, exit 1 with nothing sent; a key the engine refuses is its refusal, exit 2 |
| a moved record's file, whole (`areas.md`, area 5) | 128 KiB, jig's bound on a keyed request's value (jig's `protocol/client/README.md`, section 3); past it, or empty, exit 1 with nothing sent; a file past its form's bound at the engine is its refusal, a limit, exit 2 |
| a forge's pasted value: its file, and the bundle the tool makes of it (`areas.md`, area 6) | 4 KiB, jig's bound on a carried value; past it, or empty, exit 1 with nothing sent; a value the engine refuses, one past the deployment's `records.value_bytes` among them, is its refusal, exit 2 |
| a picker's page, `forge repositories` (`areas.md`, area 6) | 100 rows, the most `picker.page` takes, printed whole: temper's own family carries them, not jig's rows; a step test fails when 100 rows at the client's bound on a field's bytes pass a result's text (`testing.md`, 1) |
| a wait's timeout | 300 s by default, at most 3600 (`tools-waiting.md`, 1) |

The client's other limits are the terminal's (`../README.md`, section 2).
The mandatory `me` frame takes every membership the deployment may serve
(jig's `client/domain/contract/documents.md`, 1), including a project just made.
Its project list grows as it fills up to nine entries; the whole document
keeps its 1 MiB wire bound. Templates and models keep their per-project
bounds. Raising the membership count reserves no storage for absent
memberships.

## 2. Open questions

- **The browser's membership boundary** (`bounds.md`, section 1): check a third
  membership and the nine-membership bound through the browser after
  landing. The native decoder's table and one-process restart stories
  cover the bound below that tier; the web preset uses the same nine.

- **A turn's outcome past the client's text** (`bounds.md`, section 1): the client
  keeps at most its text, 32 KiB, of a call's outcome in a turn (jig's
  transcript bound `outcome`), so an outcome's part never fills a 64 KiB
  page. Keeping more raises the client's text, and jig's wire takes none
  past 64 KiB. Default: 32 KiB.
