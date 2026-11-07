# hosts: status

2026-11-07. Where jig's code lags or departs from [hosts.md](hosts.md).

Section 6.3 has no gap in transcript refusal classification across the worker
channel or in closing refused starts without an acknowledgement.
Section 6.3 has no gap in retaining a pre-admission agent fault and its
bounded diagnostic through cleanup, or in cancellation winning first.
Section 8's independent delivery grant for a Finite report is present:
the policy's `modify` and the root's workspace delivery fact are both
required, and delivery leaves the report and failure forms unchanged.
The charter tests cover the grant's combinations, bounds and encoding;
Open and verdict runs still receive no delivery grant. The shell
departure below remains.

- **Not yet:** 3, 5.2, 6.5, 11: `smith-inline-agent` as a hub's agent
  capability. 4: an admitted run's answers passing a held assignment.
  6.1: the engine hub's hello before placement. 8, 9: the test root puts
  a task's message together itself, rendering its words by
  `jig_protocol_tools::words` (`jig/testing/jig-test-domain/src/link.rs`,
  `down`), because its manifest does not link `jig-protocol-charter`.
  The code's side is
  `jig/docs/development/debt/D004-ops-root-jig-wiring.md`. 10: an agent
  process's tree contained.
- **Departs:** 8: a run with no `deliver` keeps the shell its policy
  grants, where the section has it inspect alone: `charter`
  (`jig/crates/jig-protocol-charter/src/charter.rs`) narrows `modify`
  and leaves `shell` as the policy says, and
  `workspace_tools_follow_every_combination_of_run_policy` pins that.
  Such a run can write in its workspace by a command, and no delivery
  carries it out. No reason is recorded.
  9: a sender's label gives a person by number and the deployment by the
  word `deployment`, where the section has a party by name: `Party::sender`
  (`jig/crates/jig-domain-tasks/src/boundary.rs`) reads a `Party`, which
  holds numbers. No reason is recorded.
  9: the user's request of 2026-11-05 has temper's consumers use jig's
  messages, but `temper-engine-domain/src/link.rs`'s `message`,
  `inbox_sender` and `inbox_words` stay beside
  `jig_protocol_charter::message`, because they render other words and
  the request keeps behaviour unchanged; `inbox_sender` starts from
  `Party::sender` (the code's side is
  `jig/docs/development/debt/D004-ops-root-jig-wiring.md`).
