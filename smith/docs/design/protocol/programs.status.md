# programs: status

2026-11-10. Where smith's code lags or departs from [programs.md](programs.md).

Sections 2 and 5.1 have no known gap in Linux's program-child launch:
status checks and print runs use the agent's own `--program-child`
mode, which execs with Kill on parent death. The argv and malformed-mode
tables cover the launch; the Linux end-to-end abort story observes the
agent, fake program and relay, kills the agent alone and checks that
neither descendant remains running. Simulated program worlds launch
directly and do not prove the kernel signal. macOS also launches directly;
its crash lifetime remains section 9's deferred work.

Sections 5.1, 5.3 and 8 have no known implementation gap in a run with no
workspace replacing the prompt, disabling built-in tools and slash commands,
and starting in its private empty `work` directory, including its summary
session. Every run replaces the prompt by default, with the agent's
`keep_program_prompt` launch choice appending instead. The backend table
covers both choices with and without a workspace, including summary sessions
and resumes; the configuration table checks the default and forwarding.
The fake program records the arguments and initial directory; the
MCP-attached size measurement remains section 9's work.

Section 5.1 has no known gap in each program's file answers or in an
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

Section 3 has no known gap in reading a usable status's optional email,
bounded to 255 bytes, without making usability depend on it. Section 8's
fake status supplies it; this does not establish a real-login render check.

## Gaps and departures

- **Not yet:** 4.2: no test names `ANTHROPIC_PROFILE`, the federation
  variables or `ANTHROPIC_BASE_URL`, which `keeps` drops with every
  `ANTHROPIC_` name, nor a status naming a profile's or a gateway's
  method as signed in otherwise. [in flight: change 1912]
- **Not yet:** 4.2: a source's commands do not set `ANTHROPIC_CONFIG_DIR`
  to the source's `anthropic` directory: `environment` and `sign_in` set
  `CLAUDE_CONFIG_DIR` alone, so a profile in the host's
  `~/.config/anthropic` reaches them. [in flight: change 1915]

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine/executors.md`,
  3; this is a design gap, not an absent implementation.

- **Not yet:** 5.1, 5.5: what the program offers is not read from
  `system/init`'s `capabilities`: `checked_init`
  (`smith-protocol-program/src/backend/stream.rs`) checks its tools and
  MCP servers only. 8: whether the live suite passes on Claude Code
  2.1.312 is unknown; 5.8's explicit current-activation refresh is
  checked through the fake program, with its live model treatment unknown.
  Its two agent-process probes and the summary
  session's probe passed in the after-landing run at main
  `df41529d3ee1`, whose Claude Code release is unknown. No after-landing
  run read holds the 5.10 probes of two programs answering at once or
  children answering in a later activation on one configuration directory
  (testing.status.md, 2.3).
  8: no live walk establishes that the stream rejection seen in the advisor
  and the earlier read-only/review sightings is resolved. The step regression
  exercises a reachable `StructuredOutput` boundary; the rejected live record
  is unknown. The unknown top-level notice regression is synthetic: it
  checks that an outstanding built-in result and a finishing completion
  survive such notices, but does not identify the record refused in the
  live coding-producer sightings or establish their resolution.
- **Departs:** 5.1: an owner may ask a backend for what the program was given
  (`Backend::keep_given`, `Backend::pop_given`): its argv, instructions, MCP
  tools, each input line and its `system/init`, because the program makes its
  own calls and these are all a development deployment can read of them. Never
  its environment or `mcp.json`, which hold the MCP secret. Unasked, nothing is
  kept.
