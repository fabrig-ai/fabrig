# ui-wait-words: the binding contract

The shapes that cross the plan's two parts, `bounds` (jig) and `words` (temper). The design they follow: `jig/docs/design/domain/engine.md`, section 8 (*A run its hosts are all too small for says which bound keeps it out*); `jig/docs/design/client/domain/contract.md`, 5.1; `jig/docs/design/protocol/client.md`, section 3; temper's words, `temper/docs/design/client/README.md`, 5.5 (*A run's wait*), and ctl's, `temper/docs/design/client/tools.md`, 6.1.

## Why

proof-2's goal waited on "no host has room" for good: its charter grew to 100,000 turns after the box's worker (800 turns) was admitted. `needs::replace` clears the worker's eligibility for that charter and keeps it admitted, so `fits` keeps the run off it for ever, no hello is refused, and `standing_refusal` stays suppressed by the admitted worker: the task reads `TaskRunWait::Host`. temper's startup refuses such a charter at a start (`temper-service/src/startup/checks.rs`), so the case is a charter changed after it.

## The fleet (`jig-domain-fleet`)

```rust
// history.rs: the standing fact keeps what the refusal already computes.
pub struct HelloRefusalRecord {
    pub channel: Token,
    pub at: Wall,
    pub slots: u32,
    pub stop_bound: Duration,
    pub why: HelloRefusal,
    /// `HelloRefused::figures`: some for an agent's shortfall with amounts.
    pub figures: Option<Figures>,
}

/// Every admitted, connected, non-draining host of an allowed kind falls
/// short of a bound of one charter version (domain/engine.md, section 8).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BoundsRefusal {
    pub kind: HostKind,
    pub why: Option<AgentShortfall>,
    pub figures: Option<Figures>,
}

impl Domain {
    #[must_use]
    pub fn bounds_refusal(&self, version: Version, kinds: Kinds) -> Option<BoundsRefusal>;
}
```

`bounds_refusal` answers:

- **none** when no admitted, connected, non-draining host of an allowed kind exists (the standing refusal's case), when `version` is no configured version (`fits`' "an unconfigured charter has no needs"), or when one such host falls short of no bound of `version`;
- otherwise `kind` is `Worker` if any such host is a worker, else `Engine`, and only that kind's hosts are read further;
- **the bounds and their order** are `shortfall`'s for one charter, then the deployment-wide ones: each model need's declared endpoint `Window` then `Output` (an undeclared endpoint is skipped, as in `charter_shortfall`), `Turns`, `Time`, `Waiting`, `Checks`, then `Spend`, `Message`, `Inbox`, `Unacknowledged`; `Model` is never answered;
- `why` is the first of them that **every** one of those hosts falls short of, naming `version.charter` (and the model position for a window or output); `figures` is `Figures { required: <version's need>, offered: <the most any of those hosts offers> }`, and for `Checks` `offered` is the least checks timeout among them (a checks shortfall is a timeout longer than the reserve); durations in milliseconds, as `figures` gives them;
- where each host falls short of some bound but no one bound keeps them all out, `why` and `figures` are both none.

Busy slots, a source's readiness, logins, model declarations, exclusion and the host rule play no part: those are `Host`.

## The core (`jig-domain-core`, `documents/task_wait.rs`)

```rust
pub enum TaskWaitRefused {
    Hello { kind: HostKind, at: Wall, why: HelloRefusal, figures: Option<Figures> },
    Bounds { kind: HostKind, why: Option<AgentShortfall>, figures: Option<Figures> },
}
// with jig_domain_fleet's HostKind, HelloRefusal, AgentShortfall and Figures
```

`waiting`, for `Err(RunWait::Host)`: the standing refusal first, as now, now with its `figures`; when it answers none, `core.fleet.bounds_refusal(version, kinds)`, `version` the task's `charter_version` (none: plain `Host`) and `kinds` `charter_of(task).hosts`; its answer becomes `Refused(Bounds { kind, why, figures })`; none stays `Host`. Account and credential waits keep their precedence; nothing is stored.

## The client contract (`jig-client-domain`)

```rust
// contract/deployment.rs
pub enum AgentShortfall {
    Model { charter: u32, model: u32 },
    Window { charter: u32, model: u32 },
    Output { charter: u32, model: u32 },
    Turns { charter: u32 },
    Spend,
    Time { charter: u32 },
    Waiting { charter: u32 },
    Checks { charter: u32 },
    Message,
    Inbox,
    Unacknowledged,
}
/// What a host bound needs and what is offered, in the bound's own unit,
/// a duration in milliseconds (client/domain/contract.md, 5.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Figures {
    pub required: u64,
    pub offered: u64,
}

// contract/llm.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TaskWaitRefused {
    Hello { kind: HostKind, at: Wall, why: HelloRefusal, figures: Option<Figures> },
    Bounds { kind: HostKind, why: Option<AgentShortfall>, figures: Option<Figures> },
}
```

`Figures` is reached as `jig_client::contract::Figures`, as `AgentShortfall` is. The deployment document's `HelloRefusalLine` is unchanged but for its `AgentShortfall`, which now keeps the three charters the engine's translation dropped.

## The wire (`jig-client-protocol-wire/schema/client-v19.schema`, in place)

```
record AgentShortfallTurns { charter: u32 }
record AgentShortfallTime { charter: u32 }
record AgentShortfallWaiting { charter: u32 }
enum AgentShortfall { model: AgentShortfallModel, window: AgentShortfallWindow, output: AgentShortfallOutput, turns: AgentShortfallTurns, spend, time: AgentShortfallTime, waiting: AgentShortfallWaiting, checks: AgentShortfallChecks, message, inbox, unacknowledged }
record ShortfallFigures { required: u64 offered: u64 }
record TaskWaitHello { kind: HostKind at: u64 why: HelloRefusal figures: option ShortfallFigures }
record TaskWaitBounds { kind: HostKind why: option AgentShortfall figures: option ShortfallFigures }
enum TaskWaitRefused { hello: TaskWaitHello, bounds: TaskWaitBounds }
```

Arms keep their order, so every existing tag keeps its number; the new records precede their consumers, as the generator requires.

## ctl's JSON (`temper-client-tools`, `render/document/llm.rs`), the `words` part's

`--json` keeps `run_wait` as jig gives it, in the renderer's existing shapes:

- `{"refused":{"hello":{"kind":"worker","at":N,"why":WHY,"figures":FIGURES}}}`
- `{"refused":{"bounds":{"kind":"worker","why":SHORTFALL,"figures":FIGURES}}}`, `SHORTFALL` `null` when none;
- `FIGURES` is `{"required":N,"offered":N}` or `null`;
- `turns`, `time` and `waiting` print as the charter-bearing shortfalls do: `{"turns":{"charter":N}}`.

## What the `bounds` part leaves in temper

The `bounds` task edits temper only so that it compiles, and the `words` task replaces each of these:

- patterns gain `..` or the new fields: `TaskWaitRefused::Hello { kind, at, why, .. }`, `AgentShortfall::Turns { .. }`, `Time { .. }`, `Waiting { .. }`;
- `TaskWaitRefused::Bounds` arms: the page's `choice::refused` gives `agent_shortfall(w, why)` for a `why`, else `W::WaitCapacity`'s words; ctl's `refused_text` prints `no host takes it`, and its JSON `{"bounds":{"kind":KIND}}`;
- test literals gain `figures: None` and the shortfalls' charters.
