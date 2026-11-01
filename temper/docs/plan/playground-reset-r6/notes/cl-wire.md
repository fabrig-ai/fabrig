# The client wire's change for playground-reset-r6 (cl)

The exact schema of `notes/contracts.md` T8 as `cl-wire` builds it, and how its new values
read against the engine's answers (contracts T4 to T6), for every task that encodes or
decodes them (sv's family translation, cl's client tasks, the client world).

## 1. The schema's change

In `temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema` (file and
generated module keep their `v1` name):

- Before `enum Request`, new:

  ```
  # make_forge: a forge made by its kind, then set up or connected by its number
  # (temper/docs/design/draft/playground-reset.md, section 3).
  record ForgejoChoice { api: text 259 git: text 1024 web: option text 1024 }
  enum ForgeChoice { owned, github, forgejo: ForgejoChoice }
  record MakeForge { choice: ForgeChoice }
  # remove_forge: a forge removed whole (draft playground-reset.md, section 4).
  record RemoveForge { forge: u16 }
  ```

- `enum Request`: `start_demo` renamed `start_playground` in its place (tag 2), and three
  variants appended, in this order:
  `enum Request { adopt: Adopt, set_up: SetUp, start_playground, return_: Return, connect: Connect, install: Install, reachable: Reachable, accept: Accept, make_forge: MakeForge, remove_forge: RemoveForge, end_playground }`
- `enum DemoState { not_started, running, ended }` becomes
  `enum PlaygroundState { not_started, running, resetting }` (same tags: `resetting` takes
  `ended`'s 2).
- `enum Read { demo }` becomes `enum Read { playground }`.
- `versioned record DemoAnswer {...}` becomes `versioned record PlaygroundAnswer { project:
  u32 forge: u16 provider: u32 owner: bytes 1024 name: bytes 1024 state: PlaygroundState
  cues: list 8 text 256 }`, members unchanged.
- Before `enum Outcome`, new: `record MadeForge { forge: u16 }` (`Made { id }` is the first
  repository's, already taken).
- `enum Outcome`: six variants appended after `accepted`, in this order:
  `made: MadeForge, full, not_offered, removing, playground, reset`.

Renamed variants keep their tags and appended ones take new tags, so every byte the wire
encoded before encodes the same: the committed journeys' asks are unchanged.

**The version.** The family line moves from `family temper-client-wire 11` to `12`, and
`CLIENT_VERSION` with it, in the task that records the journeys again (`cl-journeys`), not
in `cl-wire`: every forge ask carries the version, so its move stales both committed
journeys (`temper/docs/design/client/journeys.md`, 8), which the gate's
`temper-client-journeys` test replays. Until then the shape is v12's at version 11
(`notes/cl-progress.md`, Contract requests, 1).

## 2. Generated Rust names

`w` is `temper_client_protocol_wire::v1`.

- `w::Request::{StartPlayground, MakeForge(w::MakeForge), RemoveForge(w::RemoveForge),
  EndPlayground}`; `w::ForgeChoice::{Owned, Github, Forgejo(w::ForgejoChoice)}`.
- `w::Read::Playground`; `w::PlaygroundAnswer` / `w::PlaygroundAnswerParts`;
  `w::PlaygroundState::{NotStarted, Running, Resetting}`.
- `w::Outcome::{Made(w::MadeForge), Full, NotOffered, Removing, Playground, Reset}`.
- `temper_client_protocol_wire::{playground_answer, decode_playground_answer}` replace
  `demo_answer`/`decode_demo_answer`; the `Limits` members `demo_answer_*` become
  `playground_answer_*`; new `forgejo_choice_api`, `forgejo_choice_git`,
  `forgejo_choice_web_some`, each its schema ceiling.
- `decode_requesting` refuses (None) a `make_forge` of `forgejo` with an empty `api` or
  `git`, or a `web` that is `Some` and empty, and a `remove_forge` of forge 0.

## 3. How the engine's answers read on the wire

The reading cl's client tasks build on, asked of sv's family translation (temper-service
`family.rs`) once fe's answers exist (contracts T4 to T6; `notes/cl-progress.md`, Contract
requests, 2):

| Engine answer | Wire `Outcome` |
|---|---|
| `MakeAnswer::Made { forge }` | `made { forge }` |
| `MakeAnswer::Refused(MakeRefusal::Full)` | `full` |
| `MakeAnswer::Refused(MakeRefusal::NotOffered)` | `not_offered` |
| `RemoveAnswer::Removing` | `removing` |
| `RemoveAnswer::Refused(RemoveRefusal::NoSuchForge)` | `connect_refused { why: no_such_forge }` |
| `RemoveAnswer::Refused(RemoveRefusal::Playground)` | `playground` |
| `EndAnswer::Reset` | `reset` |
| `EndAnswer::NotOffered` | `not_offered` |
| `StartAnswer::*` (`StartPlayground`'s) | as `StartDemo`'s today: `running`, `start_refused { not_offered }`, `start_failed` |
| none (past retention) | `unknown`, as today |

`Read::playground` answers `PlaygroundAnswer` with the `Playground` row's state
(`Running` → `running`, `Resetting(_)` → `resetting`, no row → `not_started`), its
`project` and `forge` from the row (0 while none), and `provider`, `owner`, `name`, `cues`
from `application.demo` as the demo's answer has them today; no record while the document
has no `application.demo`.

## 4. In between (cl-wire to the engine's playground)

Until sv's family translation routes them, `cl-wire` leaves:

- temper-service: `make_forge`, `remove_forge` and `end_playground` answered at once
  `not_offered` (an `Answered` from `to_root`, never reaching the root);
  `start_playground` routed as `start_demo` was; `Read::playground` answered from the
  demo's settings and state as `Read::demo` was, the engine's `DemoState::Ended` as
  `resetting`.
- the client: `PlaygroundState::Resetting` read as the client's `DemoState::Ended` (the
  retired playground), so nothing a person sees changes until cl's playground task reads it
  as resetting.
