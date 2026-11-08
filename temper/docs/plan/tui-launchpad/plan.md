# tui-launchpad: the plan

The user's look request `tui-launchpad` (`inputs/request.md`, its commit message
`inputs/request-commit.txt` and hand-made frames `inputs/launchpad-01.txt` to `-04.txt`):
after sign-in, the TUI opens on a launchpad for the person's project, whose box asks Ask
temper; `/` lists the commands; a question opens a chat, read as the browser's chat
terminal reads it, with an agent's question under the box as a numbered list. Every
commit carries `Look-request: tui-launchpad`; the last carries the account
(`temper/docs/design/draft/look-requests.md`, 5).

Contracts: `notes/contracts.md`; standing rules: `notes/common.md`.

**Interfaces that change.** skein: `skein-ui-screen`'s `Border::sides`, `Align`,
`StyleLook::{align, band}`, `Theme::active` and `Focus::{active, follow, ended, rows}`,
and its key reader's keys for a field an options menu follows
(`skein/docs/design/ui-screen.md`, 3, 5.2 to 5.4, 6.2). No skein crate but
`skein-ui-screen` and `skein-ui-screen-terminal` changes. temper: the TUI's own crates
throughout; `temper-client-domain`'s Home page (its composer asks Ask temper); a public
surface of `temper-client-view` that the TUI shares with the browser's chat terminal.
No compatibility is kept: none is owed before v0.1.0.

## Decisions

- **The launchpad is the client root at `Home { project }`,** the first of `me`'s
  projects: Home already reads the status agent's brief, what needs the person and the
  person's live chats, so the TUI draws them from there and derives nothing again.
- **The TUI's root holds the terminal's look** (`LookKept` with `terminal: true` at
  start), so the domain's own terminal machinery is the TUI's: commands, the lists they
  open, notes, a confirmation as a list, a chat's question read. `/theme` flips the
  root's look, which the TUI draws in; the browser's `KeepLook` and `Screen` requests are
  dropped.
- **The domain's one change** is Home's composer asking Ask temper, as `Ask { project }`
  does. The browser's Home sends neither words nor `StartChat`, so it is unmoved.
- **skein first:** the frames need a region framed by rules, a title's end part, lines
  aligned to their end, bands, an options menu from a field and a body that follows a
  log's end. Each is generic (the theme names it, or skein-ui already says it) and goes
  into skein's full-screen shell before the TUI draws with it.
- **Commands:** `/draft` and `/goal` are left out with drafting, `/font` is the
  browser's; `/new` opens the launchpad; `/exit` closes temper.
- **`sel`** joins the palette for the person's band and the active option, the one
  background the TUI paints, by this request.

## Overlaps

- `temper-client-domain` is shared with the web client's owner: the change is Home's
  composer, in `pages.rs`, `pages/choice.rs` and `pages/terminal.rs`; the last was
  changed by `look-round-19/night-keep` (merged into its trunk), so the two meet there in
  separate hunks. Ask temper's plans' files are left alone.
- `temper-client-view` is shared likewise: the TUI's shared surface keeps off `frame.rs`,
  `look.rs`, `words.rs` and `tests/rail.rs`, which `look-round-19/night-switch` changes.

## Parts and tasks

| Task | Needs | Crates and files | Wave | Minutes |
|---|---|---|---|---|
| `screen-looks` | | skein-ui-screen (theme, renderer, goldens) | 1 | 150 |
| `screen-lists` | screen-looks | skein-ui-screen (focus, renderer), skein-ui-screen-terminal (keys) | 2 | 210 |

The TUI's view, host and shared words, the domain's change, the joint story and the
journeys follow in later batches.

## Critical path

`screen-looks` → `screen-lists` → the TUI's view → its host → the journeys recorded
again, with the account in the last commit.
