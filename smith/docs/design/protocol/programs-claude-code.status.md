# programs-claude-code: status

2026-11-27. Where the tree's code lags or departs from [programs-claude-code.md](programs-claude-code.md).

- **Not yet:** 1.1 and 1.10: fast mode's `--settings` for an LLM at `fast` [in flight: change 20560].
- **Not yet:** 1.7: a completion charged by the speed its stream's usage names [in flight: change 20565].

Section 1.1 has no known gap in each program's file answers or in an
answer not asked for. The agent service hands a file answer to main or
a child by that backend's file owners alone (`Backend::owns_file`), so
a tool's answer whose owner equals a program's pipe token goes on to
the machine; the service's table delivers one, with 2, 3 and 5
children, at seven points before, between and after the children's
directory answers and status checks. A backend that gets an answer
with nothing outstanding fails its run with the detail `smith: a
private file answer the program did not ask for`, or drops it once
ending; the backend's table checks seven such rows. The request of
plan `program-burst-files` asked for every order in which the file
component may deliver children's and main's files: those seven points
stand for it, not an interleaving of each program's whole sequence of
run root, instructions and outcome files, since the cause was an
owner's identity, not an order. The deployment's abort is attributed
to that cause by reading and by those rows; its evidence is outside
the repository, and no deployment run since landing is on record.

- **Not yet:** 1.1, 1.5: what the program offers is not read from
  `system/init`'s `capabilities`: `checked_init`
  (`smith-protocol-program/src/backend/stream.rs`) checks its tools and
  MCP servers only.
- **Departs:** 1.1: an owner may ask a backend for what the program was given
  (`Backend::keep_given`, `Backend::pop_given`): its argv, instructions, MCP
  tools, each input line and its `system/init`, because the program makes its
  own calls and these are all a development deployment can read of them. Never
  its environment or `mcp.json`, which hold the MCP secret. Unasked, nothing is
  kept.
