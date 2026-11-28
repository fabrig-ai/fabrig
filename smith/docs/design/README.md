# smith's design

## 1. In one page

- **A kit and an agent.** smith covers what a flexible LLM agent does,
  whatever it is for: conversations with any provider, tools (its own on
  a workspace, its host's, MCP servers'), sub-agents, budgets, results
  judged against a contract, changes checked and handed over, transcripts
  to resume from. The same crates build `smith`, an agent with standard
  features that runs on its own or under any host that speaks its
  protocol.
- **Not only a coding agent.** A chat assistant, a researcher, a triager,
  a reviewer, an operator acting through its host's tools, a coordinator
  of other agents, and a producer of code changes are all runs of the
  same agent, told apart by their charters. Nothing in smith assumes code,
  repositories or a workspace at all.
- **One vocabulary, two kinds of agent.** An agent runs one run at a
  time: as a process its host spawns, talking to it over one channel
  whose vocabulary is defined here (domain/host.md) and whose bytes are the
  protocol design's (`docs/design/protocol/`); or inline, as smith's
  domains in its host's process, speaking the same vocabulary as
  entities (domain/host.md, section 9). A host (temper's worker, smith's local
  host, a test's world) starts runs, serves what they ask of it and
  hears how each ends.
- **skein's layering** (`skein/docs/foundation/programming-model.md`): every part is a domain,
  a protocol layer and io, so a project built on smith imports it at the
  layer it needs: the domains in its worlds, the protocol crates to talk
  to an agent or read its transcripts, the shells and binaries to run
  one, or everything in one process (`domain/README.md`, section 2).
- **The domain is complete** (skein/docs/foundation/programming-model.md, section 4). A world
  of domains and fakes runs everything an agent does, with no protocol and
  no io.
- **Policy is data.** A host shapes a run entirely by its charter and its
  answers: instructions, a brief, the tools it may call and those the host
  serves, what counts as done, where to look for checks and guides, a
  budget. smith interprets no workflow vocabulary: no roles, task kinds,
  verdict names, message kinds, forges, branches or pull requests
  (`domain/README.md`, section 3).
- **A workspace is optional, and generic.** A run may be given
  directories to work in, some writable; one that is a git working tree
  gets git's protections and may start from a merge in progress. A run
  without one works through its host's tools, MCP servers and its
  messages. A change is what the writable directories hold, checked; what
  making it durable means (files kept, a commit, a push, a pull request)
  is the host's: smith asks for a delivery and tells the LLM how it
  went.
- **Any model, any provider.** The conversation vocabulary is
  provider-neutral; each provider's API is skein's shared LLM client's.
  A run starts on the model its charter names and may give its sub-agents
  others.
- **Failures are visible.** The LLM sees what went wrong, bounded in size:
  a failing check's output, an edit that matched nothing, a result that
  broke its contract, a host's refusal in the host's own words. So does
  the operator: a run's failure, an agent's and a host's reach the person
  or the operator typed and are shown in words, with bounded detail,
  whichever kind of agent runs, and never in what the LLM is told. A
  failure is never replaced by a fixed message, for either.

## 2. Reading order

1. **domain/README.md:** this page: what smith is, its parts, how it stays
   generic.
2. **domain/run.md:** one activation: the tools a run serves and relays,
   results and their contracts, the budget, the answer.
3. **domain/run-start.md:** the charter, the start, the prompt, and the opening.
4. **domain/run-messages.md:** receiving and reading messages, waiting, parking, and resuming.
5. **domain/run-delivery.md:** checks, delivery, merges in progress, and saving unchecked.
6. **domain/session.md:** one conversation with an LLM: turns, transcripts,
   providers, prices, sub-agents.
7. **domain/session-context.md:** admission against the window, compaction, and keeping context small.
8. **domain/tools.md:** what a session does to the workspace.
9. **domain/host.md:** the other side: what a host owes a run and is owed, the
   channel, supervising agent processes, the local host, and one process
   through the inline agent.
10. **domain/turn-parts.md:** a turn told in parts for a host that keeps a
   run's place across its restart: the answer and each call's result as
   they come, a drain at the answer, and a cut turn resumed.
11. **protocol/limits.md:** declared quantities, startup checks, memory and what happens at a limit.
12. **protocol/limits-derivation.md:** how each layer's limits and a window's room are derived.
13. **protocol/programs.md:** choosing and signing in to a program backend, and what it promises.
14. **protocol/programs-claude-code.md:** running Claude Code, from the command and tools to its stream and sub-agents.
15. **benchmarks/README.md:** what benchmarks measure, when they run, and how tasks and suites are laid out.
16. **benchmarks/agents.md:** agent adapters and model tiers.
17. **benchmarks/results.md:** the result schema and statistics.
18. **benchmarks/isolation.md:** isolating attempts, logins, and process trees.
19. **testing.md:** the testing strategy, fakes, scenarios, checks, and layout.
20. **testing-tiers.md:** the tiers, including simulated worlds, the real loop, and end to end.
21. **drawn-agent.md:** what the fake model says in smith's worlds: a body for every tool, the slots, the refusals it targets, its weights.
22. **shapes.md:** the words in which smith's worlds count the shapes their completions reached.
23. **parks.md:** a session's states at a park, as kinds smith's worlds draw, with what each becomes if the run resumes or ends.
