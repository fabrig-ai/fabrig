# program-burst-files: parallel sub-agents on the Claude Code program backend never abort the agent

On a real deployment at fabrig `b4555e6785`, a goal's root run on the Claude Code program backend asked several `sub_agent` surveys in one message. Each of four attempts then aborted the agent process within minutes:

```
thread 'main' panicked at smith/crates/smith-protocol-program/src/backend/files.rs:325:46:
one pending private file request
```

That is `Backend::file_event`'s `self.files.pending.take().expect(..)`: a private-file event reached a program backend that had no private file request pending. `subagent-burst`'s rows (2, 4, 5 and 6 children, program and direct) all passed, so they don't drive this path as the deployment does. The run's failure was read as `invalid: unreadable result`. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes three tasks, and say what you dropped

1. **The failing test first,** at the lowest tier that holds the program backend and the agent service's file routing (`smith-agent-service/src/service/program.rs`'s `file_event`, `backend.rs`'s owner match on `token(Owner::Files)`, `program_children.rs`'s rows): a program-backed parent asks 2, 3 and 5 `sub_agent` calls in one message, and the children's private-file events (their run roots, instructions and outcome files, opened, stored and removed while the parent's own are in flight) are interleaved with the parent's in every order the file component may deliver them. Find which event reaches a backend with nothing pending: a child's event routed to its parent by a shared owner token, a second event for one request, or an event after the backend ended. Rows of the existing table.

2. **The fix:** each private-file event reaches the backend whose request it answers, and an event a backend didn't ask for is a failure of that run with a reason, never an abort of the process. Say which cause it was.

3. **The design says so:** smith's program backend design where a run's private files and its children's are written.

## Not in this plan

A charter's words, a model or effort, and any figure.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, on a scripted program and a fake model; no test reaches a real host or a real LLM. Failing tests first. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. smith names neither jig nor temper. `walk = false`.
