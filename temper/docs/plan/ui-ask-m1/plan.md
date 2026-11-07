# ui-ask-m1: Ask temper, alone and small

The Ask temper draft's first milestone, M1 (`temper/docs/design/draft/ask-temper.execution.md`,
3, M1): charter 11 and template 5 beside the configuring chat, no workspace, its own
tools alone in the asking family, Claude Code's prompt replaced, `see` with
configuring's addresses, `propose_settings` with charter 3's mechanics moved in,
`document`, and `ask.txt`. The shapes that cross tasks are `notes/contracts.md`; the
standing rules `notes/common.md`. The design each batch cites is pushed with it.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| smith-start | `claude-start` | | Claude Code for a run with no workspace: `--system-prompt-file`, `--tools ""`, `--disable-slash-commands`, an empty `work` directory; the fake program takes and records them |
| smith-start | `mechanics` | | smith's system text with no workspace has no `## Tools` or `## Checkout` |
| family | `family` | | jig's asking family, bit 2,048, `ToolKind::Ask` |
| client | `client` | | Settings' Ask temper row and `Ask { project }`'s page, starting template 5 |

Further batches (being written): the stock (charter 11, template 5, `ask.txt`, the
rules' and policy 1's tools and delegation), the declarations and no workspace for an
asking charter, `see`, `document`, `propose_settings`'s mechanics, and the bound on
what temper gives a first request.

## Critical path, as it stands

`family` (45 min) leads to the stock and the declarations; the rest of the path is
written with the next batch.

## Shared pieces, for the heads-ups

- **smith's Claude Code start (4.4):** a run with no workspace replaces Claude Code's
  prompt (`--system-prompt-file`), names no built-in tools, disables slash commands and
  runs in an empty `work` directory beside its own files
  (`smith/crates/smith-protocol-program`, `claude-start`); its system text drops
  `## Tools` and `## Checkout` (`smith/crates/smith-domain-run`, `mechanics`). Every
  run with no workspace changes, temper's status agent (charter 9) and trials
  (charter 10) among them.
- **jig's tool families:** a twelfth, asking, bit 2,048 (`jig-domain-core`, `family`).
- **`temper-client-domain`:** `Address::Ask { project }` and `ASKING_TEMPLATE` (`client`).

## Overlaps with drafts beside this one

- `tui-approval/temper-domain` and `tui-approval/temper-view` (merged to their trunk)
  list `temper/crates/temper-client-domain/**`, `temper/crates/temper-client-view/**`,
  `temper.css` and the gallery: `client` edits the Settings page, the configure page,
  `address.rs` and `chats.rs` and the gallery's settings samples, where that plan edits
  approvals; unavoidable, and textually apart but for the gallery's `names.txt` and
  `tests.rs`, which are regenerated on a catch-up.
- `ui-stop-coding-row/jig-stop-interrupts` lists `jig/docs/design/domain/engine.status.md`:
  this plan's jig status entry is in `authority.status.md` instead, and its design adds a
  row to `engine.md`, 7.3, alone.
- No draft beside this one lists `smith/**`.

## Decisions

- **Template 5 is known by number in the client,** as template 3 is; no fallback to a
  chat: with no template 5 in `me`, Settings draws no Ask temper row.
- **No compatibility:** the asking bit is new in every authority mask; no stored value
  holds it, so nothing migrates.
