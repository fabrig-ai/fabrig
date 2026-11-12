# hosts: status

2026-11-11. Where jig's code lags or departs from [README.md](README.md).

`hub.md`, section 4 has no known gap in turn fences taking only offered messages:
messages withheld by parts or a pace stay in the task's inbox when a
later offered message is read. Selecting parts belongs to the core and
adds no host-side deferred work.

`README.md`, section 2 and `charters.md` have no known gap in endpoint bounds at hello and
charter-supplied model names; the host keeps no model allowlist.
`README.md`, section 2's optional application-supplied reply bound crosses the hello
as a byte count, independent of run admission, with no known gap.
The fleet exposes it for the attempt hosted there (`../engine/fleet.md`).

`hub.md`, section 6 has no known gap in a loss told with why or a worker's
return: each side's link component names each loss's cause, which
temper's service prints (`../../protocol/channel/README.status.md`;
`temper/docs/design/protocol/cluster.md`, 3), and the engine's world takes back a worker
lost fifty times, its closing connections aborted at the stall deadline.

`hub.md`, section 7 has no known gap in a worker's hub saying it drains: a
shutdown's `CancelAll` sends `LinkUp::Draining` before anything its
cancelled runs send, and a shut hub sends it again after each opening
(`jig-domain-host/src/domain.rs`, `step`; the hub's tests
`a_shutdown_says_draining_before_any_cancelled_runs_answer` and
`a_shutting_hub_says_draining_again_after_each_opening`).

Section 5.1 has no known implementation gap in serializing list reads
with source checks and sign-outs, giving smith the grant's value below
the hub, or cancelling a read on link loss without an answer. A read
takes neither a slot nor a workspace.

`hub.md`, section 3 has no gap in transcript refusal classification across the worker
channel or in closing refused starts without an acknowledgement.
`hub.md`, section 3 and `charters.md` have no known gap in a refused start's class: any
invalid start but a transcript's spends a permanent try
(`jig-domain-hub-link`'s `payload::ending`), and an agent's
`FromAgent::StartBusy` ends its start refused busy, which jig's test
worker sends for smith's busy.
`hub.md`, section 3 has no gap in retaining a pre-admission agent fault and its
bounded diagnostic through cleanup, or in cancellation winning first.
`charters.md`'s independent delivery grant for a Finite report is present:
the policy's `modify` and the root's workspace delivery fact are both
required, and delivery leaves the report and failure forms unchanged.
The charter tests cover the grant's combinations, bounds and encoding;
Open and verdict runs still receive no delivery grant. The shell
departure below remains.

`charters.md` has no known gap in a change's `body` following the contract's
words, or an independent report delivery's body taking the larger of
those words and 1,024. The outcome still holds all fields and framing
together; a field's bound alone does not promise that the whole fits.
`README.md`, section 10 defers the general message-rendering allowance.

- **Not yet:** 3, 5.2, 9: `smith-inline-agent` as a hub's agent
  capability. 4: an admitted run's answers passing a held assignment.
   the engine hello remains `hub.status.md`, 1. 7: the test-root message rendering gap remains `charters.status.md` (Not yet, the whole document). 8: an agent
  process's tree contained.
- **Departs:**  the shell-without-delivery departure remains `charters.status.md` (Departs, the whole document).
  7: a sender's label gives a person by number and the deployment by the
  word `deployment`, where the section has a party by name: `Party::sender`
  (`jig/crates/jig-domain-tasks/src/boundary.rs`) reads a `Party`, which
  holds numbers. No reason is recorded.
  7: the temper message-rendering departure remains `charters.status.md` (Departs).
