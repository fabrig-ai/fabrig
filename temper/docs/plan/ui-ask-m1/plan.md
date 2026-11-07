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
| declare | `declare` | family | an asking charter (asking, and settings at most) is declared `see`, `document` and `propose_settings`'s asking form alone; the root's `Tool::See`, `Tool::Document`, decided by `ToolKind::Ask`, answered by stubs; host-part accounting |
| declare | `workspace` | declare | an asking charter's runs get no repository |
| stock | `stock` | declare | charter 11, template 5, `ask.txt`; the rules' and policy 1's tools 3,975 and delegation to charter 11 |
| stock | `bound` | stock, mechanics | the step test holding what temper gives charter 11's first request to 16,384 bytes on both shapes |
| proposal-read | `proposal-read` | | jig's core: a run's read of one proposal of settings, a task's of its project or a party's, decided by the settings family |
| see | `see` | declare, stock, document | `see`'s parser and routing; the settings addresses, `revisions`, `revision N`, `help`, `more`; the bound, `see:` lines, not-yet and unknown; the skeleton for the forge's and the proposal's addresses |
| propose | `propose` | declare, see | `propose_settings`' asking mechanics: a document's repository as `OWNER/NAME`, a large set in parts held by the engine, approvals judged at the call for the chat's project, the answer saying where the card waits and whose decision it is (both forms) |
| document | `document` | declare | `document` writes the chat's one note by section (write, append, replace), the root splicing it and writing through jig's note family; the client reads a `document` call as a `note` call |

Further batches (being written by the `see` designer): `see-forge` (`file`, `settings
landing`, `settings gates`) and `see-proposal`.

## Critical path, as it stands

`family` (45 min), `declare` (180), `document` (240), `see` (240), then `see-forge`,
`see-proposal` and `propose` beside each other: about 705 implementer-minutes to `see`.
`stock` (150) and `bound` (90) run beside `document`; `proposal-read` (150) from the start.

## Shared pieces, for the heads-ups

- **smith's Claude Code start (4.4):** a run with no workspace replaces Claude Code's
  prompt (`--system-prompt-file`), names no built-in tools, disables slash commands and
  runs in an empty `work` directory beside its own files
  (`smith/crates/smith-protocol-program`, `claude-start`); its system text drops
  `## Tools` and `## Checkout` (`smith/crates/smith-domain-run`, `mechanics`). Every
  run with no workspace changes, temper's status agent (charter 9) and trials
  (charter 10) among them.
- **jig's tool families:** a twelfth, asking, bit 2,048 (`jig-domain-core`, `family`).
- **jig's run reads:** `NamedAction::ReadProposal` and `Now::ProposalRead`, a run's read
  of one proposal of settings (`jig-domain-core`, `proposal-read`).
- **The new host tools, in `temper-engine-domain` and `forge_route.rs`:** `Tool::See`,
  `Tool::Document` and `DocumentAction` in the root's vocabulary, `ToolKind::Ask`, the
  stubs `route/see.rs` and `route/document.rs` (`declare`); `run_workspace` gives an
  asking charter's runs no repository (`workspace`); `temper-engine-smith` declares
  `see`, `document` and `propose_settings`' asking form to asking charters alone, and
  every other charter's offer is unchanged (`declare`).
- **`temper-engine-domain`'s root answers:** `CallAnswer::Document`, `FlightRender::Document`
  and `Read::Document` (`document`); the client's chat page reads a `document` call
  as a `note` call (`temper-client-domain/src/pages/chat.rs`,
  `temper-client-view/src/pages/document.rs`).
- **The stock:** charter 11 and template 5, and the rules' and policy 1's tools gain
  the asking bit (3,975) and charter 11's delegation (`stock`); a project made before
  keeps its own policy until its owner accepts a proposal.
- **`temper-client-domain`:** `Address::Ask { project }` and `ASKING_TEMPLATE` (`client`).

## Overlaps with drafts beside this one

- `ui-stop-coding-row/jig-stop-interrupts` lists `jig/docs/design/domain/engine.status.md`:
  `proposal-read`'s design adds its **Not yet** as that file's last bullet, and the task
  deletes it there; textually apart from that plan's lines.

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

- **A large set's parts are held in the engine's memory,** by task, not in a `work:`
  note: a proposing call then consumes them whatever comes of it, and no party can
  correct them; a restart loses them, and a run live across it is refused its next
  proposing call once (`agent.md`, 8.4.2 and section 10).
- **Approvals at the call for the chat's own project alone,** another project's left to
  acceptance, so that nothing of another project reaches this chat (a departure the
  draft's status records, decided by this cut).
- **The answer's words change for charter 3 too:** one decoder and one answer serve both
  forms of `propose_settings`.

- **One writer per file in the engine's crates:** `document`, then `see`, then
  `propose`, since each writes `route/host_route.rs` or `temper-engine-smith/src/settings.rs`
  and the crates' `mod` lines; `see` writes the whole skeleton for `see-forge` and
  `see-proposal`, which write their own modules alone.
- **`help` through the root:** `see help` is routed and decided by the asking family,
  not answered at decode as `read_docs` is, so that its part may be 6,144 bytes
  (`agent.md`, 8.4.1).

- **An asking charter** is one whose `tools` hold the asking bit and nothing but it and
  settings, so that a mask of every family (`RunPolicy::EVERY_FAMILY`, a record with no
  mask) never turns a charter into one.
- **`propose_settings` has two forms, one decoder:** charter 3 keeps its declaration byte
  for byte (its stock figures do not move); asking charters are declared a form with
  no `note` and a `more` member.

- **Template 5 is known by number in the client,** as template 3 is; no fallback to a
  chat: with no template 5 in `me`, Settings draws no Ask temper row.
- **No compatibility:** the asking bit is new in every authority mask; no stored value
  holds it, so nothing migrates.
