# Charters, tools and contracts

The core decides a run's charter in its own terms (../engine/executors.md, 1). Its
translation into smith's charter (smith's `domain/run.md`, 3.1; smith's
`protocol/charter.md`, section 2; the version,
`../../protocol/README.md`, section 11), is in one place,
`jig-protocol-charter`, a function of jig's that links smith, which the
engine's protocol layer calls for every run, on workers and in the
engine alike, and which reads every run's result back by the path of
host tools' calls (`hub.md`, 8).

Its `assignment` assembles that charter from the core's run charter, the brief and the run's sources, with the contract's policy, which is jig's, not an application's: a contract's text at most 1024 bytes and a change's `body` at most the contract's words, so a report is a setting of the application's, with the agent's outcome, which holds its title, its body and their framing together (smith's `protocol/limits.md`, 3.10), and the hub's `outcome_bytes` beside it, its checks required to pass, and the workspace tools the run's policy grants. Its `charter` translates what `assignment` assembled, decides `deliver`, and grants `modify` only to a run that can deliver: what a run writes leaves its workspace only by a delivery, so a run with no `deliver` inspects alone, whatever its policy says (a rail, not an instruction). A root calls them and decides none of it.

A Finite run takes `deliver` in one of two ways; an Open run never does (1):

- **A change among its outcomes:** with that change's fields and checks, as above. Its `finish` offers the change, the successful `no_change` and a failure.
- **No change among them:** its outcomes hold a report, its policy's `modify` is set, and its root says its workspace delivers. The assembled charter's `delivers` carries that, as `conventions` carries its paths: a fact of the connectors' items (`../connectors.md`, section 10), false unless a root sets it, which grants nothing by itself. Its `deliver` takes a `title` of at most 1024 bytes and a `body` of at most the contract's words or 1024 bytes, whichever is larger, with its checks required to pass. Delivering isn't finishing: its `finish` offers its contract's forms alone, a report and a failure, and a delivery is no part of its result (smith's `domain/run.md`, 8.4).

Every other run has no `deliver`: a verdict's run, and a report's run whose policy withholds `modify` or whose root doesn't say its workspace delivers.

A run granted `deliver` by a change among its outcomes is also granted smith's `save` (smith's `domain/run.md`, 8.5), so that it can keep its work in progress, unchecked, before it asks a question; a run whose `deliver` its root's `delivers` gave, and every other, is not. Like `deliver`, it is jig's decision, in `charter`.

Its `hosted` puts a run's whole hand-over together from what the hub link kept (the core's `Assignment`), and so reads the brief's placed parts: the brief's sections with their titles, the charter, the start's messages with their senders and words, and the answered calls by their names. What a root gives it is its own: its tools, its endpoints, its deployment and its bounds. A place in the brief that an application offers no section for refuses the assignment. Its `message` puts together, the same way, a task's message to a run under way, with its sender and words. So a root that calls them renders no words, titles or a call's name, and the hub receives the same assignment from each. An application with a tool vocabulary of its own, as temper is (`temper/docs/design/protocol/components.md`), puts its hand-over together itself.

- **Instructions and brief:** the charter's instructions, and the brief's
  sections rendered as smith's titled sections, in order, the
  connectors' sections among them as their connectors render them.
- **Tools:** the task's tool families become smith's:

  | jig's family | In smith |
  |---|---|
  | inspecting, modifying, the shell | smith's inspect, modify and shell, when the run has a workspace, and modify only with `deliver`; on a program backend, the program's built-in tools of these families, each call asked of smith's policy through the program's permission prompt (smith's `protocol/programs.md`, 5.3 and 5.4) |
  | sub-agents | smith's sub-agents; on a program backend, inspect only, each a program session of its own (smith's `protocol/programs.md`, 5.10) |
  | the engine's tools (../engine/executors.md, 3) | host tools, each declared with its schema, its description and whether it writes |
  | each connector's reads, and the effects and topics it offers to agents | host tools, declared by the application; its effects through the engine's `effect`, its topics through `subscribe` |
  | waiting, by its wake policy | smith's `wait` |
  | a delivery, by its result contract or by its policy and workspace, as above | smith's `deliver`, when the run has a workspace that delivers; and smith's `save`, with a change among its outcomes |

  Each host tool's schema and description are written once, with the
  function that decodes its calls and the one that renders its answers;
  a call's input is decoded into the core's or the connector's typed
  request, or answered as an error naming what is wrong. The engine's
  protocol layer decodes every call the fleet has admitted, from workers
  and from the engine's own hub alike, so no domain parses a call's input
  (`../../protocol/channel/README.md`, 5.3).
- **The engine's tools that act on tasks, `introduce` aside, speak a
  party's operations** (../engine/executors.md, 3; ../people/acts.md, 1). Each host
  tool's input names what its operation names, as the run was shown it:
  - `delegate`: a batch, each task with what a party's `Create` names
    (../tasks/README.md, section 4);
  - `amend`: the task, its spec's revision, and the change; a change of
    priority alone names none;
  - `release`: the task, its hold's revision, and, for an effect held
    uncertain, whether to make it again;
  - `cancel`: the task, a reason, and a grace, if the run gives one;
  - `decide`: a proposal by its number, or an escalation with its hold's
    revision, and the choice, with a reason;
  - `propose`: the action, with a reason: a batch; an amendment, naming
    the revision as `amend` does; an effect of a kind a connector offers
    to agents; or a widening of the task's own authority;
  - `withdraw`: the proposal;
  - `subscribe`, `unsubscribe`, a task's own: a topic a connector offers
    to agents, a referenced task's state, or a timer.
- **A revision is shown before it is named** (../engine/executors.md, 3): this
  translation renders each revision where the run sees it, and a call
  answered moved with what the task is now.
- **A connector's topics offered to agents** are declared by the
  application with its connector's host tools, each with its schema and
  description, as its offered effects are. A `subscribe` call naming one
  is decoded by the engine's protocol layer into the connector's typed
  topic, and checked as a read is: the task's tools hold `subscribe`, and
  a read grant covers the resources the connector says the topic
  concerns (../authority.md, 8.5; ../connectors.md, 5.3).
- **Result contracts** (../tasks/README.md, section 3) become smith's (smith's
  `domain/run.md`, 7.1): a report; a verdict, with its labels, fields and
  follow-ups; a connector's result, as a delivery the run must make
  through its workspace before it finishes, with the fields its workspace
  asks for, or a successful `no_change` verdict with bounded explanation
  and no fields or items; a failure. The no-change verdict maps to the
  task's distinct successful `NoChange` result, with no resource.
  The core judges a result again as it commits it
  (../tasks/lifecycle.md, 7).
- **Budget, models, waiting:** the run's budget, its turns, spend and
  time, its spend reserved from its task at its claim, in the
  deployment's unit, and its reserve of turns and time for finishing
  (smith's `domain/budget.md`, 1); its models, from its task's LLM choice
  (../tasks/README.md, section 3), the main session's and those its sub-agents may
  use, each with its effort, on an endpoint named as the agent's
  configuration names it, with its window, its output and its prices
  (../engine/executors.md, 1); for each model's LLM provider, the account its check
  chose, whose grant says the source, and so the backend, since an
  endpoint names no account (../accounts.md, 3.4; smith's
  `protocol/programs.md`, section 3); the charter's waiting time, and
  whether a run resumes its transcript or starts fresh.
- **What jig asks of smith's budget** (smith's `domain/budget.md`
  ): before each completion, its sub-agents' included, smith reserves
  that completion's most (its input at the highest input rate, its
  output at the output rate) from the run's budget, and returns the
  difference when the completion ends. When the ordinary part of the
  budget is spent, the run winds down on its reserve and finishes; a
  completion that does not fit what is left at all ends the run for its
  budget. All of it is inside the run, with no call to its host, so a
  run never spends past its budget (../authority.md, section 7). On a
  program backend, which makes its own completions, smith cannot reserve
  one first: the program may pass the ordinary part by one completion of
  its main model, which the reserve takes, and the run stops at its next
  turn (smith's `protocol/programs.md`, 5.7 and section 7). Such a run's
  reserve holds one more of those completions (../authority.md, section 7),
  so it never spends past its budget either.
- **The core places no run smith would refuse.** A task whose budget
  cannot hold its run's reserve and one of its main model's most costly
  completions, and on a program backend one more again, is held for its
  budget (../authority.md, 7 and 8.3); the fleet admits no host whose agent
  cannot run the engine's charters, and places a run only on a host that
  declares its endpoints and where its sources are usable (../engine/fleet.md
  ). When a change of settings changes the charters (../engine/settings.md,
  3), the fleet judges its connected hosts afresh from what each
  declared at its hello: a host now short of a charter stays connected, its running
  attempts with it, and takes none of that charter's new runs; its next
  hello is admitted or refused as any is. The same admission decision is public as
  `jig_domain_fleet::shortfall`, using the core's owned
  `settings::fleet_needs::configured` requirements, so an application's startup can
  diagnose its prospective host before it says hello. A start smith refuses
  all the same fails its
  attempt as invalid, naming why (`hub.md`, 3).

## 1. Open compatibility

A task's ending (../tasks/lifecycle.md, 9) is smith's mode for its run: a task its
result ends runs `Finite`, and one its requester ends runs `Open`. An
Open run is granted the wait and never delivery.

An Open task's program path is admitted as a Finite task's is, since
smith runs an Open run on a program backend, keeping the program across
its turns (smith's `protocol/programs.md`, 5.9): selection and the
claim's recheck judge its models and grants by the same view as any
task's, each source, account and host still checked and the binding
revalidated exactly.

Each admitted hello declares the program endpoint numbers its channel
serves, the engine host's among them, fixed for that channel. At most 64 distinct
numbers are admitted, whatever the number of models: the cap is kept
with the hello's other bounds, and the worker wire's ceiling is the
same. The wire and the domain each refuse a longer list, or one naming
a number twice, before keeping it, and a malformed list refuses the
channel without counting an agent short.

A refused agent start carries its typed cause, `StartCause::{Other,
Grants}`, apart from the operator's bytes. Either is an invalid start:
it spends a try of the permanent class, an assignment its host found
invalid (../tasks/lifecycle.md, 6), as any invalid start but a refused transcript
does, so that a host that refuses the same start every time ends its
task's tries rather than spawning an agent for ever. No model gave
anything, so none is ever the invalid class, which is a result's. An
agent that refuses a start for want of room, as smith does when it is
busy, says so (`FromAgent::StartBusy`), and the hub answers the start
refused busy, as at its entrance: no failure, placed again
(../engine/fleet.md).
