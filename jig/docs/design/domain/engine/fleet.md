# The fleet

The fleet child domain knows the hosts and the runs they host:

## 1. Hosts and attempts

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
  (../accounts-sources.md, 1). The engine's own slots never take a program
  source: `smith-inline-agent` has no program backend.
- **The host rule:** a host that keeps a person's program logins takes
  only runs that person's accounts pay for, until a run's process tree is
  contained from them (../accounts-sources.md, 1; ../hosts/README.md, 5.1). One person's
  deployment is unaffected. The deployment's document names each host
  the rule keeps a run from, and why. The rule governs placement: a queued
  assignment rechecks it, while assigned and adopted runs continue.
- **Attempts are fenced.** Everything a host sends is dropped unless its
  attempt is its task's live one, but a cancelled attempt's answer. No
  attempt of a task is placed while the fleet knows a host may still run
  another.
- **Answers and turns are acknowledged once committed** (`decisions.md`, 1.2), or once
  fenced off; a copy before then is dropped unacknowledged, one for an
  attempt the core has finished with is acknowledged again.

## 2. Loss of contact and placement

- **Losing contact.** A lost worker's runs are kept for a grace; one that
  reconnects says what it hosts, and the core keeps or cancels each. Past
  the grace they are lost, and their tasks retry. The engine's own slots
  are lost only with the engine (`restart.md`). A worker back as a new
  process, its hello `fresh`, ends at once, suspended, each claim of it
  that its hello does not list (`executors.md`, 6.5).
- **A suspended task goes back to its worker.** Within its window, its
  claim takes that worker's channel alone, and waits for it where that
  channel can't take it; past the window, any host (`executors.md`,
  6.4). An answer names the position of the worker whose channel gave
  it, so that the core knows which worker a stop cut (`executors.md`,
  6.1).
- **A draining worker leaves placement.** A worker that starts to shut
  down says it drains before anything its runs' ends send (../hosts/hub.md,
  7): from then until its channel is lost, the fleet places nothing
  on it, neither a run nor a job, whatever slots free there, while the
  runs it hosts go on to their ends and their turns and answers are
  taken as ever. An attempt the fleet chose for it and has not yet
  assigned there is placed again, as one refused as busy is, and no
  attempt is refused for the drain. A refusal as busy keeps a worker out
  until a slot frees there or the worker changes its word to taking.
  An empty worker can thus recover local capacity. A worker shutting
  down stays out for the rest of its channel. A worker that dials again
  is a new channel, placed on once its logins answer.
- **A paused worker leaves placement too.** An owner's ask to pause or
  drain a linked worker (`workers.md`, 5) is the fleet's to carry: it
  asks the worker's admitted channel, `Request::Pause { channel, mode }`
  (`../../protocol/channel/pause.md`), and from then places nothing on
  that channel, neither a run nor a job, whatever slots free there,
  while the runs it hosts go on to their ends and their turns and
  answers are taken as ever. An attempt chosen for it and not yet
  assigned there is placed again, as for a draining worker. The
  worker's own word, `Event::Paused { channel, state }`, keeps the
  channel out too: whatever the fleet asked, a channel whose worker
  last said paused, draining or drained is placed nothing until it says
  taking. So an ask to resume places on the channel again once the
  worker says it takes work, and a channel lost and dialled again by a
  worker that kept its pause stays out from its word after its hello.
  A worker whose version cannot hear the ask is still placed nothing:
  the pause is the engine's placement, and only the drain needs the
  worker. A worker's new process starts taking: its restart ends its
  pause. A host's view says each channel's pause (`workers.md`, 6).
- **A mismatched worker leaves placement too.** A channel that answered
  an assignment refused for a version (`../../protocol/channel/wires.md`,
  6), or whose engine could not send a running attempt what it holds
  (`../../protocol/channel/layouts.md`, 5.2), is marked mismatched:
  until it is lost, the fleet places nothing on
  it, neither a run nor a job, as on a draining one, while what it hosts
  goes on. The task refused claims again elsewhere, spending no try.
- **A task its worker's layouts cannot keep leaves that channel
  alone.** An assignment or a job the engine's component refuses as
  *untranslated* (`../../protocol/channel/layouts.md`, 5.1) keeps its
  task off that channel until the channel is lost, while the channel
  takes every other run and job; the task claims again, spending no
  try, on any other host that may take it, or waits.
- **A program that starts without its tools leaves that place to its
  task.** A run whose program started without smith's tools ends
  `RunFailure::Untooled` (`../hosts/hub.md`, 3): a fault of its place,
  the program source's login on that worker, never of its task.
  - **It fails once.** The core ends the attempt
    `End::RefusedWithReason { reason }`, as an untranslated assignment's,
    spending no try (`../tasks/lifecycle-holds.md`, 1), its reason
    `PROGRAM started without smith's tools: account SOURCE on worker
    WORKER`: the program the attempt's main model's endpoint names
    (`claude_code`), the program source that model ran on, by name
    (`../accounts-sources.md`, 1), and the worker's configured name
    (`workers.md`, 2), its number where its setting is gone; cut as an
    invalid start's words are (`../hosts/hub.md`, 3):
    `claude_code started without smith's tools: account claude-alice on
    worker box-2`. An attempt with no program source among its bindings,
    which no program can fail so, keeps the run's own failure,
    `End::Failed(Run)`.
  - **The fleet keeps the fault,** told it by the core in that decision:
    the run, the channel the attempt was placed on (`RunProof.placement`)
    and the source's name. It keeps it on that channel, as it keeps an
    untranslated task: one entry per run and source, in memory, so a
    restart starts with none. While it stands, placement takes that
    source's login there as unusable for that run alone, as a login
    needing sign-in on that host (`../accounts-sources.md`, 1): the holder's
    order passes to the account's next source on that host, or the run
    goes to another host whose login is usable, the claim's recheck
    before its assignment leaves (`run_choice`) judging it the same way.
    Every other run takes that login there as ever.
  - **It lifts when something changes there:** the channel is lost, so
    the worker's next channel starts with none (the worker reopens); the
    fleet sends that channel a check of that source (`check`: a holder's
    check, a declaration of its name again, a terminal login's end,
    `../accounts-sources.md`, 1, and `../accounts.md`, 5); or that channel's `logins`
    report that source other than usable, so a sign-in there makes it
    usable with no fault.
  - **With no place left, the task waits,** trying nothing until a fault
    lifts or another place opens, and says why: where the fleet holds a
    standing fault of the task's run, the core refines a `RunWait::Host`
    that neither a standing hello refusal nor a bounds refusal (below)
    explains, and a `RunWait::Account { why: Host }` for that fault's
    source, into `TaskRunWait::Account { account, source, why:
    TaskWaitWhy::Untooled { worker } }`, naming the fault's account,
    source and worker, `worker` its setting number (`workers.md`, 4). Its
    latest fault, where several stand. Derived, as those refusals are:
    nothing is stored, and no task state, attempt or retry changes.

  ```rust
  // jig-domain-fleet
  pub struct Untooled { pub channel: Token, pub worker: u32, pub source: SourceName } // worker: its position
  impl Domain {
      /// The run's latest standing fault, if any.
      pub fn untooled(&self, run: Token) -> Option<Untooled>;
      /// Whether a fault keeps that run off that source's login on that channel.
      pub fn untooled_on(&self, run: Token, channel: Token, source: &SourceName) -> bool;
  }
  ```

## 3. Relaying

- **Relaying:** messages and cancels down, tool calls up and their
  answers back, each call answered once; a call the fleet cannot pass up
  is answered at once, unserved, or busy while the engine stops
  (`restart.md`, 5), never left without an answer. An application's
  asks go up and their answers back the same way, fenced as calls, an
  ask from an attempt that is not live answered refused, and nothing
  committed for either (`../hosts/hub.md`, 10).

## 4. Graces and restarts

- **The graces are ordered.** The engine's grace is longer than the
  longest a worker may take to stop a run it lost contact for, which the
  worker says at its hello (../hosts/hub.md, 6); a worker whose bound is equal
  to or longer than the engine's grace is refused. So the core places no
  next attempt while a worker may still host the last, and one run at
  most writes a resource at a time.
- **The grace is derived from that bound.** jig's profiles give
  `fleet.grace` 360 seconds, above the stop bound of a worker on their
  link profile (5 and 15 seconds) with an engine-away grace of 300
  seconds, the five minutes that cover an engine's start that upgrades
  its store (temper's `draft/archive/redeploys.md`, 5.3), and the drain's
  default of 180 seconds: 326 seconds, 20 of notice and 300 of grace
  outlasting the drain, then 5 for an agent's stop and 1 for a save
  (`../../protocol/configuration.md`, section 4). The engine-away grace dominates
  that bound, so a change to its default, to the drain's or to the link
  profile moves this grace with it, and says why; a deployment that
  raises a worker's bound raises the grace with it. A worker gone for
  good has its runs placed elsewhere once this grace has passed.
- **A restart said holds its runs** (temper's `draft/archive/redeploys.md`, 5.3;
  `restart.md`, *Saying it restarts* and *Held across starts*):
  - **The restart wait,** `fleet.restart_wait` (`fleet.restart_wait_ms`),
    is what the engine's settled stop tells its workers to wait for it:
    1,200,000 milliseconds, twenty minutes, in both of jig's profiles,
    from 0 to 86,400,000; zero says nothing. One setting serves every
    worker. A worker keeps its runs for the longer of it and its own
    grace (`../../protocol/channel/restart.md`, 4).
  - **A held claim.** While the core holds a kept word at a start, each
    claim on a worker the fleet adopts or crosses is held: its deadline
    is its adoption's time plus the word's wait plus `fleet.grace`,
    rather than plus the grace alone. Unlisted at that deadline, it is
    lost there, as one at its grace's end.
  - **Through hellos and losses.** The hold stays with the claim when a
    hello lists it, and when that channel is lost: a held claim adrift
    waits for the later of its hold's end and its grace from the loss.
    A hello received does not show that the worker heard the engine.
  - **Until its worker hears the engine.** The first `logins` on the
    channel that lists it answers the `sources` the engine sent after
    that hello, so that worker's clock has stopped
    (`../../protocol/channel/worker-link.md`, 1.1): the claim is no longer
    held, and a later loss gives it the grace alone.
  - **Discharged.** Once loaded, when no claim is held any more, ended,
    lost or heard, the fleet says so once, `Request::Discharged`, and
    the core clears the word (`restart.md`).
  - **The order holds:** a worker that heard the word stops its runs
    within the longer of the wait and its grace and its stop bound's
    tail, counted from its own notice of a loss that follows the
    engine's close; the hold counts the wait and the engine's grace,
    longer than that bound, from a start that follows that close.

## 5. Host compatibility and capacity

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
- **The needs fit their bound before the fleet holds them.** The fleet
  keeps the charters' needs, in owned bytes, within `fleet.needs_bytes`
  and each held version's room beside it. Both sides of that bound come
  from outside the process, the configuration and the store's settings,
  so each is a refusal before the fleet is given needs past it, never the
  fleet's assertion to meet: a configuration whose charters' needs
  (`fleet_needs::configured`) pass it refuses the start as a
  configuration problem naming `fleet.needs_bytes`
  (`../../protocol/limits.md`), and stored settings whose needs
  (`fleet_needs::current`) pass it refuse the start before
  `Event::Needs` is sent, as `Failure::NeedsPastLimit { held, limit }`
  (`restart.md`). Both ask the fleet's one check:

  ```rust
  pub fn needs_fit(limits: &Limits, versions: u32, needs: &Needs) -> Result<(), NeedsPast>  // jig-domain-fleet
  pub struct NeedsPast { pub held: u64, pub limit: u64 }  // held: the needs' owned bytes, u64::MAX past a u64
  ```

  The fleet's construction and `Event::Needs` keep their assertions,
  which these refusals meet, and its tests build it with needs at the
  bound exactly. A settings change that grows the needs past the bound
  is accepted and refuses the next start, naming the bound to raise;
  refusing it at its proposal is open (`README.md`, section 10).
- **A hello waits for the stored settings.** The fleet judges a hello's
  agent bounds against the needs of the settings the store holds
  (`settings-store.md`, 1), never the configuration's alone: from its construction until
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

## 6. Grants and start outcomes

- **Grants follow their attempts:** each new generation of an LLM
  account's source or an application's credential goes to every live
  attempt that holds one (`README.md`, section 6), and each attempt the fleet keeps
  at a worker's hello, or adopts, is sent its current grants, so no run
  outlives its token for want of a resend. A grant never overtakes its
  attempt's assignment (`decisions.md`, 1.2), so a generation renewed while the
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
  assignment its host found invalid (../tasks/lifecycle-holds.md, 1; ../hosts/hub.md, 3). One lost with its channel leaves its attempt unlisted at the
  next hello, and the start ends lost; the next attempt is refused again,
  or placed.

## 7. Restart and history

- **Restarting** (`restart.md`): the fleet adopts the claims the store
  names, then hears that the loading is done; a run a worker lists that
  no claim adopts is cancelled once a grace has passed.
- **Its history is diagnostic:** the deployment's recent lost workers,
  refused hellos and capacity refusals are kept in memory, within their
  limits, for the deployment's document, and start empty after a
  restart.

Workers are known by their delivery channel and by the worker position
that channel authenticated as: a worker setting's number less one
(`workers.md`, 4). The fleet's membership is the current workers, given
at the start's restore and replaced in each decision that changes one. A
hello at a position with no worker, a withdrawn one, or one held by a live
channel is refused as `Position`, naming that position separately from
capacity refusals, and an admitted worker's slots are the smaller of its
setting's places and its hello's. Program login answers and deployment
facts name the worker its setting names, never a worker message. A
sign-out targets every worker not withdrawn; one not in contact is
unconfirmed at once. The fleet keeps, per worker, whether an admitted
channel holds it and since when, for the deployment document
(`workers.md`, 6). An old channel's result cannot change a new
channel's inventory or complete a later operation. Cleanup of issued names
after reconnect is a separate operation, and its login restrictions remain
until the host establishes absence.
