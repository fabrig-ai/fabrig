# subagent-burst: a program-backed run survives a burst of sub-agents

On a real deployment at fabrig `94d1f6ce16`, a temper goal's root run on the Claude Code program backend (Opus at xhigh, sub-agents on another provider's model, `agents` and `shell` granted) failed four attempts in a row as `Agent(Exited)`. Each time the model had asked five or six `sub_agent` calls in one message, and the agent process aborted on its main thread within about 15 s of the last one. Its `claude` child and that child's `--mcp-relay` were left running, reparented to init, still spending on their login. The run's failure detail held only the last 1,024 bytes of the panic report, the backtrace's final frames, so the panic's message was lost. An earlier goal that asked fewer sub-agents at once lived. Evidence is outside this repository, so the words here are what you have. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes four tasks, and say what you dropped

1. **The failing test first, then the fix: a burst of sub-agents past a run's room never panics.** At the lowest tier that holds it, a program-backed parent asks more `sub_agent` calls in one message than its run's conversation room (`run_conversations`) allows. Rows: two (as `smith/tests/agent-process/tests/suite/program_children.rs` has), exactly the room, one past it, and six, with children on the program and on the plain LLM path. Each call past the room is answered as admission designs (`Busy` or `TooMany`), the parent goes on, and nothing aborts. Read these candidates first, without assuming one:
   - `smith-domain-run/src/run/child.rs`: `run.children.insert(..).expect("admitted child count fits run slots")`, where `children` is sized `run_conversations - 1` (`run.rs`) while admission checks the domain-wide slabs and `agent::plan`'s `means.conversations`;
   - `smith-agent-service/src/service/down.rs`: `owners.insert(Owner::Llm).expect("LLM connection route capacity")`, with routes sized from socket counts (`program.rs`, `stages.rs`);
   - `smith-protocol-program/src/backend/start.rs`: `expect("server bounds admitted calls")`.
   Fix the cause the test finds, and say which it was.

2. **A failed agent's detail keeps the panic's message.** `skein-fail-stop` writes the message, then a forced backtrace; the kept detail is the tail (`smith-host-protocol/src/process.rs`'s tail, `jig-domain-host`'s `tail`), so a backtrace pushes the message out. Keep the report's `panicked at` line and message alongside the tail, within the same bound. A row of the existing detail tests.

3. **A program's children don't outlive their agent.** Program children are spawned in their own process group with no parent-death signal (`skein-shell/src/linux/process.rs`), and the release profile aborts on panic, so nothing kills them. When an agent process is reaped, its program's process group ends too, or the children are spawned to die with their parent (`exec_dying_with_parent`); choose the one the shell's design already supports, and say why. A row where the agent aborts and no child remains.

4. **The design says so:** smith's run design where a run's conversation room and child admission are written, `domain/host.md` 4 for the kept detail, and the shell's design for the children's lifetime.

## Not in this plan

A charter's words, a model or effort, and any figure: the room stays as configured.

**Tests stay cheap:** a new case of one behaviour is a row of an existing table test, and a new story goes in a world below the real loop, on a scripted program and a fake model. Failing tests first; no test reaches a real host or a real LLM. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change, or an interface change in skein's `shell`, is a question first. smith names neither jig nor temper. `walk = false`.
