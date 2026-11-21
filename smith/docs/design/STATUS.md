# Status

2026-11-21. A run whose host asks tells its turns' parts as they come,
stops at its answer when a drain carries a grace, and resumes a turn a
drain cut where it stopped (`domain/turn-parts.md`). An agent whose
channel is open leaves a termination signal to
its host, which its kit hands its parent, and cancels on one only after
the host's cancel or once the channel is gone (`protocol/agent.md`, 6;
`domain/host.md`, 4). Where smith's code lags or departs from its design, from
the status files below. Containment remains `domain/README.status.md`, 2, `domain/host.status.md`, 4, `domain/tools.status.md`, 2 and 5, `protocol/README.status.md`, 4 and 9, `protocol/agent.status.md`, 1, 3, 6 and 8, `protocol/events.status.md`, 3.2, `protocol/host.status.md`, 1 and 3, and `testing.status.md`, 6. Beside it: inline live content and starting grants remain `domain/host.status.md`, 2, 9.1 and 7; mid-work messages remain `domain/run.status.md`, 6, and threshold notes `domain/budget.status.md`, 2; program capabilities and live checks remain `protocol/programs.status.md`, 5.1, 5.5 and 8; refresh proof remains `testing.status.md`, 2.3; the account option remains `shell/commands.status.md` (Not yet, the whole document). Benchmark gaps remain `benchmarks/catalogue.status.md`, 1, 3, 4 and 5, `benchmarks/README.status.md`, 6 and 7, and `benchmarks/codex-parity.status.md`, 4 and 6; benchmark departures remain `benchmarks/README.status.md`, 5.3, 6, 7 and 10, and `benchmarks/catalogue.status.md`, 5. Domain-world protocol dependencies remain `testing.status.md`, 2. Whole-buffer tokenizer departures remain `domain/tools.status.md`, 5, and `protocol/events.status.md`, 6. Checkout wording remains `domain/run.status.md`, 3.3, and adapter limit names `protocol/limits.status.md`, 9. Request-dump departures remain `protocol/agent.status.md`, 6, and `protocol/programs.status.md`, 5.1. The local host's token tag remains `protocol/host.status.md` (Departs). `domain/session.md`,
`protocol/channel.md`,
`protocol/charter.md`, `protocol/llm.md` and
`protocol/transcript.md` have no gap. The agent's standalone `--list`
request and its host half's bounded stdin/stdout exchange are built,
without a run slot (`protocol/agent.md`, 6.1; `protocol/host.md`, section 2).
A host call withdrawn unsent is asked again with no uncertain effect, and
a completion whose renewable grant lapses while its host's renewals are
held waits for the next grant, spending no retry, as a long operation
its host's watchdog knows of (`domain/run.md`, 5.2; `domain/session.md`,
4; `domain/host.md`, 4 and 7).
Sub-agent answers have the outcome's room, with UTF-8 boundary cuts and
guidance to ask a new child, narrower; retaining the rest for a
continuation remains deferred (`domain/run.md`, 5.3 and 13;
`protocol/limits.md`, 3.10 and 13). smith has no drafts.

Failed checks name their repository, convention path and given time,
with their exit and retained output tail. A silent check still has no
output to report; phase messages belong to its repository, and no timeout
figure changes (`domain/run.md`, 8.1 and 13).

Design documents:
- [`domain/README.md`](domain/README.status.md)
- [`domain/run.md`](domain/run.status.md)
- [`domain/session.md`](domain/session.status.md)
- [`domain/tools.md`](domain/tools.status.md)
- [`domain/host.md`](domain/host.status.md)
- [`domain/turn-parts.md`](domain/turn-parts.status.md)
- [`protocol/README.md`](protocol/README.status.md)
- [`protocol/channel.md`](protocol/channel.status.md)
- [`protocol/charter.md`](protocol/charter.status.md)
- [`protocol/transcript.md`](protocol/transcript.status.md)
- [`protocol/llm.md`](protocol/llm.status.md)
- [`protocol/agent.md`](protocol/agent.status.md)
- [`protocol/events.md`](protocol/events.status.md)
- [`protocol/limits.md`](protocol/limits.status.md)
- [`protocol/host.md`](protocol/host.status.md)
- [`protocol/programs.md`](protocol/programs.status.md)
- [`shell/README.md`](shell/README.status.md)
- [`testing.md`](testing.status.md)
- [`benchmarks/README.md`](benchmarks/README.status.md)
