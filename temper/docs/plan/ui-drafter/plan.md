# ui-drafter: the plan

Milestone P6 of `temper/docs/design/draft/whole-story.execution.md`: the drafter,
a chat charter of its own, and the design session that runs it
(`temper/docs/design/draft/surfaces.md`, 11.2). Request:
`temper/docs/plan/ui-drafter/inputs/request.md`.

## The numbers, and what T2 starts from

- **Charter 12, the drafter's;** its instructions `inputs/drafter.txt`, byte for
  byte, as the stock's `temper/crates/temper-service/src/stock/drafter.txt`.
- **Template 6, a design session,** named `design session`, run by charter 12.
- **The terminal's T2** (`/draft`, its pane, its sittings on the launchpad)
  starts from `temper_client_domain::pages::chats::DRAFTING_TEMPLATE` (6) and
  `drafting(me, project, executor)`, which knows a chat as a design session by
  template 6's charter in `me`, as `asking` knows Ask temper's; and from
  `temper/docs/design/domain/agent.md`, 8.5. A session is a chat: T2 opens and
  resumes it as it does a chat, and reads its draft as a chat's document.

## The design this cut wrote

On the trunk with the first push, before any spec runs:

- `temper/docs/design/protocol/configuration.md`, 13.4: the rows *charter 12, the
  drafter's* and *template 6, a design session*; *the rules*' delegation names
  charter 12; *Each charter's host families*: charter 12's 1,022, and a charter
  that names no delegate family is declared none of its tools. 13.9, *The rules
  and policy 1*: the reference's widening leaves charter 12 out.
- `temper/docs/design/domain/agent.md`, 4.2 (a charter naming none of inspect,
  modify and shell is given no repository), 4.3 (the delegate family's tools need
  the charter's family), 8.5 (the drafter), and section 10's open question
  narrowed.
- `temper/docs/design/client/README.md`, 3.5: *A new chat*, *With a draft*; *A
  chat's document*, *A design session* and *Its mode*.
- `temper/docs/design/client/tools.md`, 5.3: `chat start`'s row names template 6.
- Status: **Not yet** entries in `configuration.status.md` (13.4, 13.9),
  `agent.status.md` (4.2, 4.3, 8.5) and `client/README.status.md` (3.5), one
  bullet per task, each deleted by the task that builds it; `configuration.status.md`'s
  host-families entry points at `agent.status.md` for the delegate family;
  `draft/surfaces.status.md`'s lag 11 narrowed to the terminal's, and its Open
  item on the drafter on the web says the design takes the default, both.

## Parts and tasks

| Part | Task | Needs | What it builds |
|---|---|---|---|
| drafter-run | `delegate-held` | | `tools_for` declares `goal`, `change` and `delegate` only where the charter names the delegate family |
| drafter-run | `no-workspace` | | `no_workspace(policy)`: a charter naming none of inspect, modify and shell is given no repository |
| drafter-stock | `drafter-stock` | | charter 12, template 6, `Charter(12)` in the delegation, the help, the readers' counts |
| drafter-stock | `reference-widening` | | the reference's jq widening takes charter 12 out |
| drafter-stock | `drafter-run` | `delegate-held`, `drafter-stock` | the meeting point: a design session's first run on the stock, no repository, no `goal` or `change` |
| drafter-web | `design-session-web` | | `DRAFTING_TEMPLATE`, `drafting`, `NewChat { draft }` starting template 6, a reopened session in Split, the view's row |

The request's tests, by task: the instructions byte for byte (`drafter-stock`,
sha256 and length pinned); its declared tools, no `change`, no `goal`, no
workspace (`delegate-held`'s row, `no-workspace`'s row, and `drafter-run`'s story
on the stock); the template starting a session on the drafter
(`design-session-web`'s row, `drafter-stock`'s template, `drafter-run`); the
stock's figures and bounds with it added (`drafter-stock`); the web drawing the
session's draft in Split, a row of `split_draws_the_pane_after_the_conversation`
(`design-session-web`).

**Critical path:** `drafter-stock` (90 min) then `drafter-run` (35 min), about two
hours; the other four run beside it from the first push.

## What was decided, and why

- **"No `change`, no `goal`" is a declaration rail,** not an authority one: the
  drafter's task is a chat's, whose authority allows delegation. temper declared
  every host tool but the settings family's to every charter
  (`configuration.status.md` recorded it), so the delegate family is now held to
  the charter too. Stock charters 2, 4, 6, 9 and 10, which name no delegate family,
  lose the three declarations with it, as 13.4 already said of them; the
  reference's charters carry no `tools` and so name every family.
- **"No workspace" by the charter's flags:** a charter that names none of
  inspect, modify and shell is given no repository, as an asking charter is. It
  holds of charters 9, 10 and 11 already, and of no other stock or reference
  charter. The drafter keeps `agents`: smith keeps sub-agents with no workspace.
- **Charter 12 and template 6,** numbered past Ask temper's, as charter 9 and
  template 5 were numbered past the reference's.
- **The delegation and the box's sixteen places:** a person's start is checked
  against the creator's, the project's and the rules' kinds, so `Charter(12)`
  joins them: fifteen. The reference's charters 7 and 8 would make seventeen, past
  `authority.executors` and `tasks.executor_kinds` (16) and ctl's form; raising
  those limits ripples through jig's profile and the stored sizes, so the
  reference's widening, whose work starts no design session, takes charter 12 out.
- **The web's start keeps its words:** a design session's first message is the
  person's words and "Write this up as a draft beside us.", as a draft chat's is,
  so that the fake model's scripted draft and the journey `chat-draft` hold on a
  fake box whose stock now lists template 6. The rail's "Goal" item and its words
  stay; P1 redraws the menu.
- **Reopened in Split:** built where `Chat` holds its task and `me`; if that
  needs `pages.rs` (another plan's now), it waits, its status sentence kept.
- **"Start goal…"** on a design session is a chat's, its ways the project's
  tracked templates (5.14): nothing new.
- **A public interface changes freely** (the request): `tools_for`'s
  declarations, the stock's lists and the client's start; no compatibility kept.

## Overlaps with drafts beside this one

- `temper/crates/temper-engine-domain/src/route/forge_route.rs`: one call in
  `run_workspace` (`no-workspace`); the seam temper/migration's plans write.
- `temper/docs/design/client/README.md` and `README.status.md`, and
  `temper/docs/design/protocol/configuration.status.md`: status files many plans
  edit. The design commit edits configuration.status.md's long host-families entry
  in one phrase; the rest are new bullets near each file's head.
- `temper/crates/temper-client-domain/src/pages.rs` (look-round-24's
  `role-pages`) is kept off: no task here edits it.
