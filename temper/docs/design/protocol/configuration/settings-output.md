# Settings for a model with a larger output

This document holds the settings for a model with a larger output, split from [`settings.md`](settings.md).

## 1. A model with a larger output

`temper-box init`'s limits hold the stock's models, each a window of 1,048,576
tokens and an output of 8,192 (`stock.md`), within its agent endpoints' 48,000-token
ceiling. A model requesting 32,000 tokens, such as gpt-6.1-sol at effort high,
fits that ceiling. A deployment whose model takes more than the endpoint's ceiling
raises its agent's endpoint and overrides its profile's turn limits, each derived
from the agent's largest turn, T, which the start checks name "the
agent's largest turn" (`temper-service`'s `startup/checks.rs`).

- **T,** the configured turn ceiling, must hold smith's largest measured
  turn (`smith-agent-service`'s `profile.rs`). That measurement uses the
  smallest shared room over every endpoint, so one set of bounds
  holds whichever model a charter names (smith's `protocol/limits-derivation.md`, 1.9).
  `temper-box init`'s agent takes 48 calls per response, 32
  conversations (`stock.md`'s table) and 64 models a charter, its endpoints an
  output of 48,000 and a `reasoning_item` of at most 65,536, and efforts
  move no figure of T, smith charging an effort's name at the charter
  codec's fixed bound; T is what smith measures for it,
  within smith's own ceiling of 33,554,432 bytes (smith's
  `protocol/limits.md`, 4, `turn-in-channel`), and init's document
  holds it, written by the test's writer (`../setup/init.md`, 4). A box
  without the demo is measured from its own endpoints and must declare
  every carrier at least as large as that measurement.
  `docs/examples/owned/agent.json` takes init's figures but its
  endpoints and programs, with its own T in `docs/examples/owned.json`.
  Init keeps the demo, its window `gpt-6-luna`'s, so the host part is
  that window's (`stock.md`; `../setup/init.md`, 6).
- **The store's ceiling:** jig's `standard` page holds a turn's frame of
  a transcript of up to 33,554,432 bytes, any turn smith admits, and the
  store's frame and the worker wire carry it (jig's
  `protocol/store/transcripts.md`, 1); small's base page ceiling is
  4,270,667 bytes, holding a loaded transcript of at most 4,270,603 bytes
  after the frame's 64. Its effective page ceiling is the larger of
  that base and the profile's key ceiling plus the settings revision
  bound derived from the deployment's effective capacities (`../store-versions.md`,
  1). A deployment on `small` keeps T's full frame within that ceiling.
  The page ceiling admits
  the largest shipped ordinary transcript (jig's
  `protocol/limits.md`). A deployment with larger output or
  reasoning items derives T from its own agent and checks that its full
  frame fits the selected profile's page.
- **The limits T sets,** "both hubs" being the engine's and each
  worker's; init's values are its agent's T's, in its document:

| Limit | Relation |
|---|---|
| both `hub.turn_bytes` | T |
| the engine's `core.resume_bytes` | the larger of T + 4 and 4,194,304 (4 MiB), so that a coding run's whole window resumes after an interruption, not one turn's worth (`../../domain/worker/README.md`, section 2) |
| both `hub.transcript_bytes` | resume + 12 × ⌊resume / 4⌋ |
| both `hub.turn_queue_bytes` | 8 × T |
| the engine's and each worker's `io.output` | the same room, the greatest worker room derived from a turn of T's frame and the workers' configured rooms |
| `root.journal.transcript_bytes` | resume |
| `root.loads.transcript_bytes` | T |
| `root.loads.bytes`, `root.loads.reply_bytes` | T + a temper record's size |
| `root.loads.read_bytes` | init: 2 × T; holds T and a row and a record |

- **One call's payload:** skein bounds a call's decoded input by the
  completion's answer, eight bytes a token of the model's output, at
  six bytes a payload byte with its framing, so at init's output of
  8,192 a call carries at most about 10,662 bytes; the agent's
  `tool_payload` is the largest smith admits at init's models, and
  `checkout.message_bytes` equals it, so the checkout accepts the
  agent's largest call, and the engine's message and result bounds are
  the payload less two (`stock.md`). A deployment whose models' output is
  larger raises them with its T, up to 262,144, smith's own bound on a
  host's input and answer (smith's `protocol/limits.md`, 2.4). A note's body is not bound by it: a
  run builds a body past one call in parts, by line ranges, up to
  `notes.body_bytes` (jig's `domain/engine/briefs.md`, section 2;
  `../../domain/agent/agents.md`, 3). `propose_settings` can name that note and
  its revision instead of inline items: the whole settings file is resolved
  internally, checked for the proposing task's note access and matching
  revision, parsed and proposed as one value (`../../domain/agent/agents.md`, 1).
  One owner acceptance makes one revision, without increasing a model's
  output or a deployment budget. A decided call replays its saved proposal
  before resolving a note again.
- **Memory:** `engine.memory_bytes` is the engine's byte budget
  (`../../domain/root.md`, 4.1), independent of a model's edge bounds.
  The start checks those bounds, without comparing memory with a sum
  or naming a deficit. A pipe the engine cannot adopt is refused
  naming what it lacks, never a bare `fd N refused`.
  `docs/examples/owned.json`'s declarations are the user's budget for
  the owned deployment and do not move with a box's.
- **The budgets** follow the model's C (`stock.md`), so a larger output gives
  larger spends with no setting changed: output 32,000 gives C 1,081.
- **The reference's room** (`../../draft/goal-as-designer.md`, section 6):
  room for a whole spec in a `change`'s words. Its
  agent declares `tool_payload` 63,740 and, on every endpoint, an
  `output` of 48,000, and its charters' models take the same `output`
  (`reference.md`), the draft's fallback of about 64 KB. skein's rule between them
  is (payload + 260) × 6 ≤ output × 8. `output` isn't sent to the providers
  it uses, so it changes no model's behaviour.
  `docs/examples/reference/room.json` holds this pair. Smith refuses
  131,072 and 98,500 at `turn-in-transcript`: `session.said_text`,
  1,245,440 bytes, passes `transcript.said_text`'s 1,048,576-byte ceiling.
  The fallback's T is 18,431,842 bytes, within smith's 33,554,432-byte
  ceiling. The stock keeps its figures, 10,662 and 8,192, and
  `temper-box init`'s T of 16,803,810.
- **What moves with the payload,** P, each named by the start at its path
  against the agent's figure, beside the limits T sets above:

| Limit | Relation | The start names it against |
|---|---|---|
| `tasks.message_bytes`, `tasks.result_bytes` | P − 2 | the agent's message (`tool_payload` less 2) |
| `tasks.inbox_bytes` | 82 × (P − 2), one of each for the standard inbox | its own bound |
| each worker's `hub.event_bytes` | P − 2 | `tool_payload` less the 2-byte separator |
| `checkout.message_bytes` | P | the agent's `call_bytes` |
| `forge.client.op_bytes` | at least `tasks.result_bytes` | `tasks.result_bytes` |
| `forge_component.body_bytes` | 2 × `forge.client.op_bytes` + 21 | `forge.client.op_bytes` |
| `forge.client.answer_bytes` | at least 2 × `forge_component.body_bytes` + `forge_component.string_bytes` + skein's file-record bytes; the reference takes 255,570 | the forge client's validity check |
| both `hub.outcome_bytes` | at least P; `standard`'s 131,072 holds the reference's | the agent's outcome (`tool_payload`) |
| both `core.call_answer_bytes` | at least P; follows it unless overridden | the agent's call input |

- **A sub-agent's answer:** smith's `run.answer_bytes` follows P as the
  outcome does, the larger of 4,096 and P (smith's `protocol/limits-derivation.md`,
  1.10), so a survey a goal's run asks through `sub_agent` answers in up
  to 63,740 bytes on the reference and 10,662 on the stock. temper
  declares nothing for it: smith derives it at the agent's start, holds
  it under `render-in-result` and prints it among the agent's effective
  limits. A longer answer is cut, its words saying how much was left out
  and to ask again, narrower (smith's `domain/run.md`, 5.3).
- **A batch:** `standard`'s `authority.batch` is 32, so a `change` of up to
  32 changes passes (jig's `domain/authority-checks.md`, section 1), still
  within `tasks.batch`, `tasks.batch_bytes` and the task room a batch
  needs. Each model's figures don't enter it. It costs a retained denial
  300 findings at most where it took 92, and a batch's aggregate 512
  grants and 544 executors where it took 144 and 153: bounds on what a
  value may hold, none of them allocated ahead. It stands in jig's profile
  and not the reference's, because `tasks.batch`, 160, already sizes
  `standard` for it and a batch's count isn't a model's figure.
- **A brief:** `standard`'s `hub.charter_bytes` is 262,144, and smith's
  charter allowance is the larger of 65,536 and twice the payload (smith's
  `protocol/limits-derivation.md`, 1.10). The reference's is 127,480, so this receiving
  allowance binds before the hub's 262,144: the engine fits a charter to
  the smaller of the two where its process has the agent, and to 65,536
  where it doesn't know its worker's (`../../domain/agent/room.md`, 2, and `../../domain/agent/README.md`, 8
  ). Its coding charter holds a 63,740-byte spec beside its
  instructions and tools. A section past the room those leave is cut with
  its `[cut N bytes; whole: READ]` line, including a 131,072-byte section,
  whatever the hub admits. The box's charter allowance stays at 65,536.
- **A report:** a change's `body` follows the call's payload
  (`../../domain/agent/charter.md`, 3).

## 2. Open questions

- **A reference call of 128 KB** (`settings-output.md`, 1): the 131,072-byte payload and
  98,500-token output pass the session's rendered feedback beyond the
  transcript's `said_text` ceiling. Raising that ceiling changes a stored
  format and needs its owner's decision. Default: the admitted 63,740-byte
  payload and 48,000-token output; no codec ceiling changes.

- **Large arguments through a real program** (`settings-output.md`, 1): the fake-program
  table carries 65,536 and 100,000 bytes whole, while the live story asks
  for 65,536 and 131,072. Default: compile that story during a plan and
  run it after landing; no real-provider result is established by the
  fake or by startup admission.

- **A Claude-only box on the small profile.** Its largest turn's store
  frame passes small's page ceiling (`settings-output.md`, 1), where `standard`'s holds any
  turn smith admits: a page decision of jig's or skein's profile.
  Default: such a box runs `standard`, as `temper-box init`'s document does.

- **A message, a report or a person's spec past what a box takes**
  (`stock.md`, `settings-output.md`, 1). The user's request of 2026-11-03 sizes a message at
  128 kB and a report at 256 kB, and a box's are its agent's message,
  10,660 bytes, since a call's payload is bounded by its model's output:
  128 kB in one call needs an output of about 98,500 tokens, and a
  message past one offer, 200,068 bytes at luna's window, fits no offer.
  A spec a person writes stops at `people.words`, 8,192, people-v14's
  ceiling, past which the format would change, below the largest seen,
  19 kB; `tasks.spec_bytes`, 131,072, is reached only by a model whose
  output admits that payload. Default: as they stand, `standard`
  holding 262,144 for a deployment with larger models. The user's to
  choose, with no answer yet: smith's messages and host answers take a
  bound of their own, an offer's (smith's `protocol/limits.md`, 13), so
  a box's results reach about 200 kB at luna's window while a call stays
  within the payload; or init's models declare a larger output where the
  provider allows it, raising every agent figure with T and the budgets'
  C; or this default stays. Past `people.words`, people-v14's ceilings
  change in place (jig's `protocol/configuration.md`, section 8).

- **A brief past the host part's room** (`stock.md`). The same request sizes a
  run's brief at 1 MB, where `standard`'s `root.brief.brief_bytes`,
  336,713, a whole spec among it, fits the room the host part leaves
  charter 5 at init's models, the least a stock charter keeps: 340,091
  bytes, less 1,472 for its 64 sections' titles and framing and a
  1,906-byte margin. It is past the largest fabrig's development has had,
  200 kB.
  1 MB needs a window of about 2.7M tokens or a larger share of
  it for the host. Default: as it stands. The user's to choose, with no
  answer yet: smith's room split asked again with measurements (smith's
  `protocol/limits.md`, 13, "A window's halves"), or a larger window
  declared for a model whose provider has one.
