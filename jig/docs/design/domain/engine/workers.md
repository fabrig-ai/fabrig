# The workers

2026-11-16. Each worker the engine admits is a setting of jig's kind
`worker` (`settings.md`, 1): its name, its places and its secret's
digest. An owner adds, changes or removes one as any setting, by one
decision, and the engine is not restarted. This document is the kind,
how the engine admits a worker by it, and what a client reads of it.
It is written from temper's `draft/config.md`, 6.7. The fleet that
places runs on the workers is `fleet.md`; the channel they open is
`../../protocol/channel/README.md`.

## 1. The kind

- **A worker is named by its number,** from 1 to the fleet's
  `configured_workers` (`../../protocol/limits.md`), the kind's bound.
  Its value is the worker whole:

  ```rust
  // jig_domain_core_vocabulary::settings, re-exported by jig_domain_core::settings
  pub struct WorkerSetting {
      pub number: u32,              // the setting's number (2)
      pub name: Box<[u8]>,          // what the worker's credential names (2)
      pub places: u32,              // the runs it hosts at once, 1 to fleet.slots (4)
      pub digest: Option<[u8; 32]>, // SHA-256 of its secret; none for the in-process worker (3)
      pub withdrawn: bool,          // removed, kept for its name (2)
  }
  pub enum Value { /* ... */ Worker(Box<WorkerSetting>) }
  pub struct Defaults { /* ... */ pub workers: Box<[WorkerSetting]> }

  // jig_domain_people, re-exported by the vocabulary
  pub enum Setting { /* ... */ Worker(u32) }
  pub enum SettingFault { /* ... */ Linked }
  ```

- **Read at the newest:** by the workers' component at each opening,
  by the fleet at each hello (4), and by a deployment document when it
  is built (6). Like every kind but charters, a worker has no versions
  (`settings.md`, 3).
- **Its defaults are the application's,** passed in `Config`'s
  defaults (`Defaults.workers`): a deployment whose workers share the
  engine's process passes its own worker, as temper's box passes
  `local` (temper's `protocol/cluster.md`, 1); an engine whose workers
  dial in passes none, and an owner adds each. An application that
  passes no defaults has none either: `jig-protocol-config` reads no
  worker from the document, whose `engine.hosts.workers` is refused by
  name (7).
- **The current workers** are the defaults with the stored changes over
  them, as for every kind (`settings.md`, 1), changed by `ChangeSettings`
  (`settings.md`, 2) or an accepted proposal of settings (`settings-proposals.md`,
  1), running work staying: a worker change moves no task.

## 2. Numbers and names

- **A number keeps its name** for the deployment's life, as a model's
  does (`settings-catalogue.md`, 1): a set naming another name than its default's
  or its stored change's is out of shape, and so is a name another
  number holds, withdrawn ones included. A rename is a removal and an
  add.
- **A removed worker stays, withdrawn.** A *removed* item keeps the
  setting with `withdrawn` set, its number and its name: it is out of
  admission (4), of the fleet and of the workers a client lists (6). A
  set of its name at its number brings it back with the places and
  digest the set gives. A *reset* returns a worker with a default to it;
  one with no default is out of shape. `withdrawn` is the removal's: a
  set whose value carries it is out of shape.
- **A client numbers a new worker:** the lowest number no worker setting
  holds, withdrawn ones included, and a withdrawn name its own number.
  `jig_client_domain::contract::worker_number` gives it from the
  deployment document's rows (6):

  ```rust
  // jig_client_domain::contract
  pub enum WorkerNumber { Live(u32), Withdrawn(u32), Free(u32) }
  pub fn worker_number(workers: &[WorkerState], name: &[u8]) -> WorkerNumber
  ```

  The core checks only what it is given (3). A change that raced
  another is *stale* (`settings.md`, 2), and the client reads again; a
  number past the bound is refused *count*, naming it.
- **A name** is 1 to `fleet.worker_name_bytes` bytes, within the link's
  `name` bound too: ASCII letters, digits, `-`, `_` and `.`, its first a
  letter or a digit. So a host's name fits, and so does every line that
  prints a worker's name between spaces (temper's `protocol/cluster.md`,
  3).

## 3. Its check

The worker kind joins `settings.md`, 1's one check, with its faults:

- ***shape:*** a name out of 2's form; places 0 or past `fleet.slots`;
  another name at a number, or a name two numbers hold (2); a set that
  carries `withdrawn`; a reset of a worker with no default.
- ***shape*, for the digest.** No digest marks the worker that shares
  the engine's process, which opens on an in-process listener and has
  no secret to check (4). Only a default may lack one, and only where
  the core's workers are in process (`Config.workers_in_process`); a
  set over a digest-less default keeps none, and a set over any other
  number carries one. Removing a digest-less worker is out of shape:
  the box's worker cannot be removed.
- ***count:*** a number past `fleet.configured_workers`, also for a
  stored worker under a limit lowered since, which refuses the start
  naming it (`settings-store.md`, 1).
- ***linked:*** a removal of a worker the fleet holds a channel of
  (5). A decision's alone: at a start no channel is admitted yet.

A refusal names the setting, `Worker(number)`, and its `at` the item,
as every change's does (`settings.md`, 2).

## 4. Admission

- **The workers' table.** The core tells the workers' component
  (`jig-protocol-engine-workers`) the current workers by asking the
  root, `Ask::Workers`, with every worker not withdrawn, by number, its
  name and its digest: at the start's settings restore, before
  `Event::Needs` (`restart.md`; `settings-store.md`, 1), and in every decision
  or release's revision that changes a worker's name, digest or
  withdrawal, part of that decision as `Ask::Catalogue` is
  (`settings-catalogue.md`, 1). The root hands it on, and the component replaces
  its table whole. The fleet is told the same workers, with their
  places, in the same decision.
- **At the opening** (`../../protocol/channel/README.md`, 4.1) the
  component takes the credential's name and secret against the table:
  a worker of that name whose digest is the secret's SHA-256, compared
  in constant time, is accepted at its position, its number less one.
  A worker with no digest is accepted on an in-process listener's
  opening alone, whatever secret it sends. **An unknown name, a
  withdrawn one and a wrong secret are one refusal, *not admitted*,** so
  names cannot be probed, and the worker redials after its backoff as
  for any refusal: a worker started before its setting links once it is
  added. A credential that does not decode stays *unauthorized*.
- **Before its table** the component judges no opening: one heard before
  the start's restore told it the workers waits for them, within the
  opening's deadline (`../../protocol/channel/README.md`, 4.3), as the
  fleet holds a hello until it hears the needs (`fleet.md`).
- **The fleet's positions are the workers' numbers:** position `p` is
  worker `p + 1`. The fleet's membership is the current workers, given
  at the restore and replaced in each decision that changes one; a hello
  at a position with no worker, or a withdrawn one, is refused
  `Position`, and one held by a live channel as before (`fleet.md`).
  Program logins, sign-outs and a host's facts name the worker its
  position's setting names; a sign-out targets every worker not
  withdrawn.
- **Places.** The fleet admits a worker's channel at the smaller of its
  setting's places and its hello's slots, until the worker takes its
  places from the engine at its opening (temper's `draft/config.md`,
  6.3). The channel keeps what it was admitted at until it is lost.

## 5. While it runs

- **Places and digest change at the next opening.** A change of a
  linked worker's places or digest is accepted, and the channel keeps
  its admission: its operator restarts the worker, whose drain lets its
  runs end (temper's `draft/archive/redeploys.md`, 6.1), and its next opening
  takes the new setting.
- **Removing a linked worker is refused** *linked*, naming its setting:
  a decision withdrawing a worker the fleet holds an admitted channel of,
  draining or not. Its operator stops it first, and its runs end
  interrupted, as at any stop.
- **One not linked is withdrawn at once.** A lost worker's runs are kept
  for the engine's grace and placed again past it, as for any lost
  worker (`fleet.md`), since its next opening is refused.
- **Up or down is its link's,** as the fleet keeps it: up while it has
  an admitted channel, down once that channel is lost.
- **Paused, drained and taking again** (temper's
  `draft/archive/worker-restarts.md`, section 2). An owner asks a linked worker
  to take no new work, to drain, or to take work again:

  ```rust
  // jig_client_domain::contract, and its twin in the core's asks
  pub enum WorkerPauseMode { Resume, Pause, Drain } // the client's names for jig_domain_link::PauseMode
  pub enum Ask { /* ... */ PauseWorker { worker: u32, mode: WorkerPauseMode } } // worker: its setting's number
  ```

  It is no settings change: no revision, no key, nothing stored, and
  asked again it is asked again, the last ask standing. The core hands
  it to the fleet, which asks the worker's channel and stops placing on
  it (`fleet.md`, *A paused worker leaves placement too*;
  `../../protocol/channel/pause.md`). It is answered at once: accepted
  once the fleet has it, or refused, *unlinked*, when the fleet holds no
  admitted channel of that worker, or *not theirs*, as any act an owner
  alone may make. What the worker does next is read from the deployment
  document (6). Its restart ends its pause: its next channel starts
  taking.

## 6. What a client reads

- **The deployment document's `worker_settings`:** every worker setting,
  withdrawn ones included, in number order, each with its link's state:

  ```rust
  // jig_client_domain::contract, and its twin in the engine's documents
  pub struct WorkerState {
      pub number: u32,
      pub name: Box<[u8]>,
      pub places: u32,
      pub digest: Option<[u8; 32]>,
      pub withdrawn: bool,
      pub linked: bool,        // the fleet holds an admitted channel of it, draining or not
      pub since: Option<Wall>, // linked: that channel's admission; else its last channel's loss
      pub pause: WorkerPause,  // that channel's pause (5); taking when not linked
  }
  pub enum WorkerPause { Taking, Paused, Draining, Drained }
  pub struct Deployment { /* ... */ pub worker_settings: List<WorkerState> }
  ```

  `pause` is the worker's last word on its channel (`fleet.md`), but
  *paused* where the fleet was asked to pause or drain it and the
  worker has said nothing since, or still says taking: so a worker whose
  version cannot hear the ask reads *paused*, as the fleet places
  nothing on it, and never *draining*. `WorkerPause` is the client's
  projection of `jig_domain_link::PauseState`, kept because the
  client's contract names no domain crate.

  `since` is none for a worker no channel of which was admitted since
  the engine started: the fleet keeps it in memory, per worker, and a
  restart starts without it. The rows are at most
  `fleet.configured_workers`, priced in the document's bound as its
  hosts are. A worker's runs are its host's, `Deployment.hosts`, by name.
- **Who reads it:** temper's `temper ctl worker` tools
  (temper's `client/ctl/workers.md`) and its Workers tab
  (temper's `client/pages/workers.md`). A digest is no secret, so it
  shows.

## 7. Store, wires and the document

- **The store:** jig's settings family gains the `worker` identity and
  value, an added variant of a registered family's schema, which moves no
  version (`AGENTS.md`, section 2); the families that name a setting
  (a refusal's facts, a proposal's heads) gain the identity each by its
  own rule.
- **The client wire** moves its version, unbridged (temper's
  `draft/archive/redeploys.md`, 3.1 and section 4), once, for all of this
  document's: `Setting::Worker`, `SettingValue::Worker`,
  `SettingFault::Linked` and `Deployment.worker_settings`.
- **The worker channel's wire** keeps its version: its credential and
  its reasons are as they were, and only the reason a refused name is
  given changes (4).
- **The pause** (5) moves the client wire again, unbridged, for
  `Ask::PauseWorker`, its refusals and `WorkerState.pause`; and the
  worker channel's wire to 19, bridged (`../../protocol/channel/pause.md`,
  2).
- **The engine's document:** `engine.hosts.workers` is refused by name,
  "workers are settings in the store", as `engine.charters` is
  (`../../protocol/configuration-engine.md`, section 1), and no digest file is
  read.

## 8. Open questions

- **A join with no digest typed:** an operator copies a digest from the
  worker's host today. A one-use token that makes the secret and gives
  the engine its digest is temper's later work (`draft/config.md`, 6.7).
- **Places from the machine:** proposed from a worker's CPUs and memory
  rather than typed. Default: typed.
