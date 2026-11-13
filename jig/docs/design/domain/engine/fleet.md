# The fleet

The fleet child domain knows the hosts and the runs they host:

- **Hosts of two kinds** (../hosts/README.md, section 3): workers, which dial in
  with their slots and the workstreams their workspaces hold; and the
  engine's own slots, a configured number, hosted by the engine's own hub
  (../hosts/hub.md), its agent `smith-inline-agent` with no
  workspace, present from construction. A charter says which kinds may
  run it. A run goes to a host with a free slot of a kind its charter
  allows, whose endpoint bounds hold its models, where its sources are
  usable and that the host rule allows (below), preferring a worker that holds its
  workstream. A refusal as busy is no failure.
- **A host's reply bound, by its runs.** A host's hello may name its
  agent's reply bound (../hosts/README.md, section 2). The fleet keeps it with the
  host's bounds and names it for a run's live claim: the bound of the
  host in contact that holds the claim, `Domain::reply_bound(run,
  attempt)`; none where that host's hello named none, or where the claim
  is adrift, waiting, a job's or not the live one. The parent sizes what
  it answers that attempt's calls by it, with its own default where none
  is named. It is no admission reason (below).
- **Endpoints at the hello, program logins after it.** A host's hello
  states its agent's endpoint bounds (../hosts/README.md, section 2). A worker's
  logins follow its hello: the engine sends it, by name, every program
  source it may be given, and places no run on it until the first
  `logins` that answers them (`../../protocol/channel/README.md`, 4.3). Its host's
  half checks each login when told of a new source, on its holder's
  check (../accounts.md, section 5) and at a configured interval
  (`../../protocol/configuration.md`, section 4), and reports what changed.
  The fleet waits for the inventory matching its desired source list,
  including an empty list, and resets that readiness on each channel.
  Hello listings and adoption recovery still run while new placement waits.
  A run is placed only on a host that declares its task's models and,
  for a program source, where that source's login is usable
  (../accounts.md, 3.4). The engine's own slots never take a program
  source: `smith-inline-agent` has no program backend.
- **The host rule:** a host that keeps a person's program logins takes
  only runs that person's accounts pay for, until a run's process tree is
  contained from them (../accounts.md, 3.4; ../hosts/README.md, 5.1). One person's
  deployment is unaffected. The deployment's document names each host
  the rule keeps a run from, and why. The rule governs placement: a queued
  assignment rechecks it, while assigned and adopted runs continue.
- **Attempts are fenced.** Everything a host sends is dropped unless its
  attempt is its task's live one, but a cancelled attempt's answer. No
  attempt of a task is placed while the fleet knows a host may still run
  another.
- **Answers and turns are acknowledged once committed** (`README.md`, 5.2), or once
  fenced off; a copy before then is dropped unacknowledged, one for an
  attempt the core has finished with is acknowledged again.
- **Losing contact.** A lost worker's runs are kept for a grace; one that
  reconnects says what it hosts, and the core keeps or cancels each. Past
  the grace they are lost, and their tasks retry. The engine's own slots
  are lost only with the engine (`restart.md`).
- **A draining worker leaves placement.** A worker that starts to shut
  down says it drains before anything its runs' ends send (../hosts/hub.md,
  7): from then until its channel is lost, the fleet places nothing
  on it, neither a run nor a job, whatever slots free there, while the
  runs it hosts go on to their ends and their turns and answers are
  taken as ever. An attempt the fleet chose for it and has not yet
  assigned there is placed again, as one refused as busy is, and no
  attempt is refused for the drain. A refusal as busy keeps a worker out
  only until a slot frees there; a drain keeps it out for the rest of
  its channel. A
  worker that dials again is a new channel, placed on once its logins
  answer.
- **A mismatched worker leaves placement too.** A channel that answered
  an assignment refused for a version (`../../protocol/channel/wires.md`,
  6) is marked mismatched: until it is lost, the fleet places nothing on
  it, neither a run nor a job, as on a draining one, while what it hosts
  goes on. The task refused claims again elsewhere, spending no try.
- **A task its channel's versions cannot carry leaves that channel
  alone.** An assignment or a job the engine's component refuses as
  *untranslated* (`../../protocol/channel/bridges.md`, 6.1) keeps its
  task off that channel until the channel is lost, while the channel
  takes every other run and job; the task claims again, spending no
  try, on any other host that may take it, or waits.
- **Relaying:** messages and cancels down, tool calls up and their
  answers back, each call answered once; a call the fleet cannot pass up
  is answered at once, unserved, or busy while the engine stops
  (`restart.md`, 1), never left without an answer.
- **The graces are ordered.** The engine's grace is longer than the
  longest a worker may take to stop a run it lost contact for, which the
  worker says at its hello (../hosts/hub.md, 6); a worker whose bound is equal
  to or longer than the engine's grace is refused. So the core places no
  next attempt while a worker may still host the last, and one run at
  most writes a resource at a time.
- **The grace is derived from that bound.** jig's profiles give
  `fleet.grace` 240 seconds, above the stop bound of a worker on their
  link profile (5 and 15 seconds) with the drain's default of 180
  seconds: 186 seconds with 5 for an agent's stop and 1 for a save (the
  user, 2026-11-12; `../../protocol/configuration.md`, section 4). The drain dominates
  that bound, so a change to the drain's default, or to the link
  profile, moves the grace with it, and says why; a deployment that
  raises a worker's bound raises the grace with it.
- **A host's agent must run the engine's charters.** The fleet refuses
  a hello, as not admitted and naming why, whose agent's bounds
  (../hosts/README.md, section 2) do not hold every run the engine may place there:
  an endpoint it serves held with a smaller window or output than a
  charter's model needs, never an endpoint it does not declare, which only
  narrows what is placed there (above); turns, spend, time or waiting per
  run smaller than the deployment's most for a run, or a charter's; a checks' timeout longer
  than the reserve's time of a charter that allows a change; a largest
  message smaller than the deployment's message bound (../tasks/messages.md, 2), or
  an inbox that holds fewer than a start carries; fewer turns kept
  unacknowledged than the hub's bound (../hosts/hub.md, 4). So no run is
  placed where smith would refuse its start, and no run refuses a
  message as too large or its start's messages as full. The fleet's
  refusal hands its parent that reason and, for a shortfall, what the
  configuration needs and what the agent offers, so that the refusal's
  text can say both (`../../protocol/channel/README.md`, 4.3); bounds set after the
  start refuse the same way.
  The core exposes these owned requirements as
  `jig_domain_core::settings::fleet_needs::configured(&Config, &Limits)`, for a
  validated configuration with the same limits used to construct the core.
  Its charter count must fit `limits.tasks.charters`, and each charter's
  main model plus alternatives must fit a `u32`. This projection does not
  validate configuration. A root may check a prospective host at startup
  with `jig_domain_fleet::shortfall(&Needs, &Bounds, HostKind)`, the same
  checker used by hello admission. It returns the first shortfall for
  applicable charters in configuration order, then checks deployment-wide
  bounds if any charter applies. Turns, time, waiting and checks name the
  failing charter's number; model window and output also name the model's
  position. Callers can then read required and offered amounts without
  repeating admission policy.
- **A hello waits for the stored settings.** The fleet judges a hello's
  agent bounds against the needs of the settings the store holds
  (`settings.md`, 7), never the configuration's alone: from its construction until
  the start's settings restore tells it their needs (`Event::Needs`,
  `restart.md`, step 1), it holds each hello it hears, admitting and
  refusing none, and judges them in the order heard once told, a first
  start's seeded settings included. A held channel lost meanwhile is
  dropped with its hello. So a worker that dials a starting engine is
  never refused for a charter its store has since changed, whichever of
  the two starts first, and a run it lists is adopted as at any hello.
- **Standing hello refusals are volatile facts.** The fleet keeps the latest
  actual refusal for each host kind, even with no diagnostic history room.
  Admitting a hello clears only that kind's fact; duplicate hellos neither
  replace nor clear it, and channel zero's reserved-channel refusal is excluded.
  `standing_refusal(kinds)` returns the newest allowed kind's fact (engine
  wins equal wall times), only while no admitted, connected, non-draining
  host of any allowed kind exists. Busy slots and model eligibility do not
  weaken that suppression. The engine's construction placeholder is not an
  admitted hello. History eviction and fact drains do not affect this query;
  a new fleet starts with neither fact. A fact keeps its refusal's figures,
  the `HelloRefused` amounts: some for an agent's shortfall that has them,
  none for every other reason.
  The core uses it only to refine an agent's Active Due or Preparing
  `RunWait::Host` into `TaskRunWait::Refused(TaskWaitRefused::Hello { kind,
  at, why, figures })`. Account and credential waits keep their precedence. No task
  state, placement decision, attempt, retry or stored record changes.
- **A run its hosts are all too small for says which bound keeps it out.**
  Hosts admitted before a charter grows stay admitted: `needs::replace`
  only clears their eligibility for it, and no hello is refused, so the
  standing refusal stays suppressed while the run can never be placed on
  them. Where `standing_refusal` answers none, the core asks the fleet
  `bounds_refusal(version, kinds)`, `version` the charter version the
  task's run takes and `kinds` its hosts. It answers only while at least
  one admitted, connected, non-draining host of an allowed kind exists and
  every one of them falls short of a bound of that version, the bounds
  `shortfall` checks for one charter (a declared endpoint's window and
  output, turns, time, waiting, the checks timeout), then the
  deployment-wide ones (spend, message, inbox, unacknowledged turns). Its
  `kind` is the worker if any of those hosts is a worker, else the engine,
  and it reads that kind's hosts alone: `why` is the first bound, in that
  order, that every one of them falls short of, and `figures` the
  version's need and the most any of them offers, the least for the checks
  timeout, so that a client can say what the largest host takes. Where
  each falls short of another bound, `why` and `figures` are none. The core
  refines the wait into `TaskRunWait::Refused(TaskWaitRefused::Bounds {
  kind, why, figures })`. Busy slots, a source's readiness, logins, model
  declarations and the host rule never make this answer: those are waits
  for room or readiness, and stay `Host`. It is derived as the standing
  refusal is: nothing is stored, and no task state, placement, attempt or
  retry changes.
- **Grants follow their attempts:** each new generation of an LLM
  account's source or an application's credential goes to every live
  attempt that holds one (`README.md`, section 6), and each attempt the fleet keeps
  at a worker's hello, or adopts, is sent its current grants, so no run
  outlives its token for want of a resend. A grant never overtakes its
  attempt's assignment (`README.md`, 5.2), so a generation renewed while the
  assignment waits for its commit reaches the attempt after it.
- **Every start and adoption ends once:** answered, lost, withdrawn (the
  core cancelled it), or refused (busy, a duplicate, or invalid).
- **A refused start keeps nothing.** A host's answer that refuses its
  start, at its entrance or as a start its agent refuses (../hosts/hub.md, 2
  and 3), ends a run the host never hosted. The fleet closes the
  attempt as it hears it, and never acknowledges it: one refused as busy
  is placed again, as no failure; one refused as invalid ends its start,
  refused. The task's class comes from the answer's payload: a refused
  transcript is transient, and any other invalid start permanent, an
  assignment its host found invalid (../tasks/lifecycle.md, 6; ../hosts/hub.md, 3). One lost with its channel leaves its attempt unlisted at the
  next hello, and the start ends lost; the next attempt is refused again,
  or placed.
- **Restarting** (`restart.md`): the fleet adopts the claims the store
  names, then hears that the loading is done; a run a worker lists that
  no claim adopts is cancelled once a grace has passed.
- **Its history is diagnostic:** the deployment's recent lost workers,
  refused hellos and capacity refusals are kept in memory, within their
  limits, for the deployment's document, and start empty after a
  restart.

Workers are known by their delivery channel and by the configured worker
position that channel authenticated as. A hello naming an unconfigured
position or one held by a live channel is refused as `Position`, naming that
position separately from capacity refusals. Program login answers and
deployment facts name the configured worker, whose name is supplied at startup, never
from a worker message. A sign-out targets every configured worker; one not in
contact is unconfirmed at once. An old channel's result cannot change a new
channel's inventory or complete a later operation. Cleanup of issued names
after reconnect is a separate operation, and its login restrictions remain
until the host establishes absence.
