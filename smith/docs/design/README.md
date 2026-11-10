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
2. **domain/run.md:** one activation: the charter, the tools a run serves and
   relays, messages and waiting, results and their contracts, delivering
   a change, the budget, the answer.
3. **domain/session.md:** one conversation with an LLM: turns, transcripts,
   providers, prices, sub-agents.
4. **domain/tools.md:** what a session does to the workspace.
5. **domain/host.md:** the other side: what a host owes a run and is owed, the
   channel, supervising agent processes, the local host, and one process
   through the inline agent.
