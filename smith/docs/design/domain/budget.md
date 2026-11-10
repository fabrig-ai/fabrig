# A run's budget and spend

- **One budget per run,** in the host's unit: what the run may spend
  across all its sessions, sub-agents' included; with wall time and
  turns. It is checked at the entrance against the agent's `Limits`; a
  run that asks for more is refused, naming what was too large and the
  most the agent admits.
- **Each part has a role.** Time bounds how long the work takes and spend
  what it costs: those are what a run plans by, and what its model is
  told (2). Turns are a loop guard, set generously, never the unit of
  planning, since a turn's worth differs across tools and providers.
- **Wall time runs from admission.** Preparing, awaiting a first message
  and waiting for one all count against it (`run.md`, section 6).
- **Spend is real.** Every host prices completions. A host that pays a
  flat rate prices them notionally, so spend still bounds the work and
  weighs uncached input as a metered host's would; such prices are for
  the budget, not a bill.
- **Every rate is positive:** input, cached input, output, and cache
  writes when priced apart. Every completion uses a token, and its charge
  is rounded upwards (session.md, section 6), so each costs at least one
  unit: spend bounds how many completions a run makes as well as what
  they cost, and neither a completion's maximum cost nor the reserve's
  spend is ever zero. A charter that prices any model at zero at any rate
  is refused at the entrance, naming the rate. A model that costs nothing
  per token, a local one or one paid by subscription, is priced
  notionally, as any flat-rate host's is.
- **Prices are the charter's,** per model: integer amounts for input,
  cached input, cache writes and output per a number of tokens, cache
  writes at the input's rate unless priced apart. Each completion is
  priced as it is made (session.md, section 6). The unit means nothing to
  smith: it is temper's deployment unit, or what the local host's
  configuration says.
- **Shares.** Each session the run opens gets a share of what is left
  (main all of it, a sub-agent its effective share, `run.md`, 5.3), and per-kind
  token caps as its limits. When what is left above the reserve is spent,
  the run winds down (1).
- **Reserved before it is made.** Before each completion, its
  sub-agents' included, a session reserves that completion's maximum
  cost from the run's budget: the input it sends and the output its
  `max_tokens` allows, at its model's prices. The input's maximum counts
  what the agent sends as at most one token per byte, plus an allowance
  its limits give for what the protocol layer and the provider add. Each
  input token is reserved at the highest of the input, cached and
  cache-write rates, since the provider, not smith, decides which each
  token is. No order among the rates is required. When
  the completion ends, the difference is returned. A completion whose
  maximum does not fit what is left to it is not made: ordinary work
  that does not fit above the reserve starts the wind-down (1), and a
  completion that does not fit what is left at all ends the run for its
  budget. So a run never spends past its budget, and stops a little early
  rather than late. All of it is inside the run, with no call to its
  host.
- **On a program backend,** the program makes each completion, so
  none is reserved before it is made. Each is priced as it is reported,
  at the charter's prices, and counted as a turn; one that passes the
  ordinary pool is taken by the reserve, and its overrun is recorded as
  spend (../protocol/programs.md, 5.7). The run still never spends past
  its budget by more than one completion.
- **A compaction is a completion** like any other (session.md, section
  8): reserved, priced, and counted as a turn.
- **Spend is told:** cumulative in each turn, and whole in the answer.

## 1. The reserve and the wind-down

- **Deferred wind-down work follows reclaim.**
  `wind_down_promoted_after_reclaim` makes successive child closes and
  main's finishing yield ready only in the next iteration. The run world
  holds this under load in `smith-run-world`,
  `orders::wind_down_promoted_after_reclaim_closes_children_once` and
  `orders::wind_down_promoted_after_reclaim_defers_main_yield`.

- **The reserve** is the charter's `reserve { turns, time }`, which the
  product's presets give a default (`../shell/README.md`). Its spend is
  derived: its turns, each at the main model's maximum completion cost
  (its window of input at its highest input rate, and its most output).
  When the charter's contract allows a change, the reserve's time holds
  at least one run of the checks (`check_timeout`, ../protocol/limits.md,
  2.3). A finish in the wind-down then always has time to check what it
  delivers, and a charter whose reserve cannot is refused at the
  entrance, naming both. A budget that
  cannot hold its reserve is refused at the entrance.
- **The ordinary pool** is what is left above the reserve, in each part
  of the budget. Main's ordinary work and every sub-agent's share draw
  from it; only main's finishing draws on the reserve.
- **The wind-down** begins when the ordinary pool is spent in any part:
  turns or time down to the reserve's, or a completion whose maximum does
  not fit above it. It is for a run that is working. A run waiting for a
  message when its ordinary time runs out, or awaiting its first, parks
  instead (`run.md`, section 6): it has nothing to finish until a message comes,
  and its host decides what wakes it. The run's budget is in one of two
  stages, ordinary or winding down, and the change is a fact (`run.md`, section 10
  ).
  An admitted Open finish request also starts this phase (`run.md`, section 6), with
  an explicit host cause and a host-request note, without charging spend or
  claiming budget exhaustion. It uses the same reserve and hard deadline.
  1. Live sub-agents are closed and settled, each call answered once,
     and their jobs end with them (session.md, section 5). Main's own
     jobs run on: a `finish` that delivers, or a `deliver`, cancels them
     before its checks (`run.md`, 8.1), and the run's end, which closes main's
     session, cancels any left.
  2. In Open, main gets the reserve's completions for a last text reply,
     with no tool choice. The note says to reply with what it has and
     promise no later work. A successful text end publishes its fence
     and parks, including with unread words queued (`run.md`, section 6).
     In Finite, main then gets at most the reserve's turns of completions,
     the first with the run's note that the budget is spent and it is to finish now
     with what it has. They offer only `finish`, `deliver` when granted,
     granted `wait` to park until another message,
     and host tools whose declaration sets `reserve`: the same tool definitions
     are sent, and the note names the reserve host tools for meeting outstanding
     obligations. The model is told to choose among those only (session.md, section 5).
  3. A result accepted then is accepted (`run.md`, section 9). A main that has not
     finished when the reserve's turns are used, or its time is out,
     answers failed for its budget.

  On a program backend, the program's turn is ended and the note sent
  as its next user message. The program's tools are not narrowed, so
  the agent refuses every call but `finish`, granted `deliver`, granted `wait` and host tools
  declared for the reserve from then on,
  each as an invalid call is answered (../protocol/programs.md, 5.7).
  An Open run on a program backend gets its last reply as
  ../protocol/programs.md, 5.9 says.
- **A sub-agent's share** runs out on its own: its final turn (`run.md`, 5.3) ends
  it, and its parent goes on.

## 2. What the model is told

All of it is text the run writes, kept in the transcript, and computed
from the domain's clock, so a world sees it exactly. Durations are told in whole
seconds, rounded down, so a share or what is left is never overstated. A
time under a second that is not zero is told as less than a second. The
exact values are the facts', in milliseconds (../protocol/events.md).

- **`## Budget`,** in main's system text: its wall time and turns as they
  stand when it opens; that spend is bounded too; the reserve and the
  wind-down; and how sub-agents draw on what is above the reserve (`run.md`, 5.3).
- **`## Your share`,** in a sub-agent's (`run.md`, 5.3).
- **A usage line** at the end of every sub-agent's result: whether the
  child stopped for its budget (`run.md`, 5.3), what it used, in turns and time,
  and the turns and time the run has left above its reserve.
- **Open replies,** under `## Replying` (`run.md`, section 6), and a last-reply note
  in place of Finite finishing instructions (1).
- **The waking instruction** (`run.md`, section 6) scopes prior exhaustion to the earlier
  activation and names the fresh allowance in `## Budget`.
- **The wind-down's note** (1) and **a child's final-turn note** (`run.md`, 5.3),
  appended.
- **Threshold notes,** with messages mid-work (`run.md`, section 6): as the run's
  time and turns pass set marks, a note saying what is left.

No amount of spend is shown: the host's unit means nothing to a model
(`run.md`, section 13).
