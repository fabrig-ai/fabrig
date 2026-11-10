# Standalone mode

## 1. The desired revision

- **What a machine converges to, set locally.** Either a commit, or a
  branch to follow, polled every interval. It lives in the steward's
  state (hearth/docs/design/domain/store/README.md, 6.3) and survives restarts and reboots:

  ```
  desired
    target       x1-prod                      the activator's machine record, in its facts
    repository   the IaC repository's URL     written at bootstrap (../client/README.md, 3.8)
    revision     commit 3f1c2ab0…  |  branch main, every 10 min  |  commit c07a11e9…, from a bundle
    trial        auto | manual
    containers   all  |  mail-dev, chat-dev  |  none    [M5] a development machine's selection (4.7)
    set by       uid 1000, 2 days ago
    generation   7                            bumped at every change
  ```

- **Set by the client:** `hearth revision set <commit>`, or `hearth
  revision follow <branch> --every <d>`. The interval is at least a
  minute, so a fleet of followers does not hammer the forge.
- **A commit from a bundle** (hearth/docs/design/domain/development.md, 6.2): `hearth revision set
  <commit> --bundle <file>` has a builder fetch a git bundle into its
  clone as `refs/hearth-dev/<commit>`, a fetch job (hearth/docs/design/domain/steward.md, 3.3),
  then sets the commit. The pass finds it in the clone and never fetches
  it from the repository's URL; render resolves it from that ref. The
  builder keeps the last 16 such refs, never the desired one's. This is
  how a working tree, committed on no branch, becomes a machine's
  desired revision.
- **`trial: manual`** stops each pass at a prepared slot and waits for
  `hearth trial`, live changes included. A workstation such as
  `x1-prod`, the MVP's machine, reboots, or changes under its user's
  feet, when its user chooses, not when a commit lands. `auto` tries at
  once, live when eligible, and suits servers.
- **The target is the activator's,** in its machine record
  (hearth/docs/design/domain/activator/README.md, 6.1), and the repository the steward's, whose
  builders keep its clone; both are written at bootstrap (hearth/docs/design/client/README.md, 3.8). The
  client changes the revision, never which target the machine is.
- **The steward's state,** a record each in the store's `state/`,
  which only the steward writes, read back at startup (hearth/docs/design/domain/store/README.md, 6.3;
  hearth/docs/design/domain/steward.md, 2.2):
  - the desired revision, above;
  - the mode, the operators' uids and, under an engine, its address,
    its trust and the machine's credential [M4] (hearth/docs/design/client/README.md, 3.8);
  - the hold (5) and the refused prepare keys (4);
  - the last pass: its generation, commit, stage and outcome, and the
    output and prepare keys it made; the bless's render reads its
    commit (hearth/docs/design/domain/steward.md, section 6);
  - orders with their outcomes, until acknowledged [M4] (`wire.md`, 4.5);
  - the rings of signals and check history [M5]
    (hearth/docs/design/domain/operating.md, section 5).

  The repository's read credential, if any, is the builder's, in the
  store's `credentials/` beside `state/` (hearth/docs/design/client/README.md, 3.8; hearth/docs/design/domain/render/README.md, 2.2).
- **Every pass names the generation it serves.** A pass whose
  generation is no longer the desired one stops at its next stage, and a
  new pass starts.

## 2. A pass

The convergence loop is a procedure of `hearth-domain-converge`:
level-triggered, bounded, and stepped by events (`jig/docs/design/domain/connectors.md`,
section 6, is the model). It compares what the desired revision makes
with what runs, and does the next thing that closes the gap. Its
stages that read the repository or build, `fetching`, `rendering` and
`building`, and the live plan in `trying`, are a builder's jobs
(hearth/docs/design/domain/steward.md, 3.3); the steward compares, asks the activator, and
blesses.

```
stage        on                                          to           does
waiting      the interval elapsed (a branch), the        fetching     a fetch job: the branch, or the
             desired revision changed, the backoff                    commit if the clone lacks it
             elapsed, startup
fetching     the commit C                                rendering    a render job: the target at C
                                                                      (domain/render.md)
             git failed, or the job crashed (domain/steward.md, waiting      back off, or wait (4.4)
             5.1)
rendering    the spec S                                  comparing    the output key: resolution reused
                                                                      by its key, or resolved
             a configuration error (../domain/render/errors.md, 1), or  waiting      wait for another commit
             the job crashed
             an operational failure (../domain/render/errors.md, 1)     waiting      back off
comparing    the running slot is blessed but is not      re-blessing  ask the activator to bless it again
             the default: a cut bless, or the firmware                (../domain/slots/README.md, 2.3, and ../domain/slots/bless.md, 1)
             fell through to it
             S equals what runs: the running slot, or    waiting      converged
             the live state's prepare key
             (../domain/slots/README.md, section 3)
             S's prepare key is refused (4.4)            waiting      reported; wait for another key
             S equals the prepared, untried slot         trying
             otherwise                                   building     a build job: K, from S
re-blessing  blessed: the default is the running slot    comparing
             refused                                     waiting      back off
building     the output, keyed K                         preparing    ask the activator to place K with S's
                                                                      slot parts and trial window
             a failure, the job crashed included         waiting      back off, refuse, or wait (4.4)
preparing    prepared                                    trying       or waiting, if trials are manual
             refused or failed                           waiting      back off, refuse, wait or rebuild (4.4)
trying       a plan live job: eligible (../domain/slots/trial.md, 7),  (respawn)    ask the activator to try live with the
             how auto or live                                         plan: delta, swap, units (../domain/slots/trial.md, 8);
                                                                      a new steward is spawned over them
             not eligible, or live refused or failed     trying       ask an ordinary trial, the plan's
             before the swap                                          reasons kept as a fact
             the activator took the trial                (reboot)     a reboot or a soft reboot into the
                                                                      slot (../domain/slots/trial.md)

over the live state, after the respawn
blessing     settled, the minimal bless holds            waiting      ask the activator to bless live: the
             (../domain/slots/trial.md, 8, L7), the bless's render                  one-shot set to the slot's trial entry
             answered (../domain/steward.md, section 6)
             a touched unit failed, the criterion        waiting      ask the live fall back (../domain/slots/bless.md, 2.5);
             failed, the bless's render failed or                     refuse b's prepare key
             crashed, or the window nearly over
             the activator fell back live at L6          waiting      refuse b's prepare key

in the new slot, after the reboot
blessing     the minimal bless holds (../domain/slots/bless.md, 1.1),    waiting      ask the activator to bless
             the bless's render answered (../domain/steward.md,
             section 6)
             a unit failed that the target's             (reboot)     ask the activator to fall back
             `bless.ignore_failed` does not list
             the bless's render failed, or crashed       (reboot)     ask the activator to fall back
             neither, within the trial's window          (reboot)     the dead-man reboots into the default

in the old slot, after a fall back
waiting      the activator reports "fell back" from b    waiting      refuse b's prepare key
```

- **"S equals a slot"** means the same prepare key (hearth/docs/design/domain/slots/README.md, 2.2):
  the same output key, the same digests of the slot parts (`data`,
  `secrets`, `boot`; hearth/docs/design/domain/render/spec.md, 1.3), the identity's and the secret
  store's current generations (6), and, from M5, the same trial window
  (hearth/docs/design/domain/operating.md, section 3). converge computes it as the activator does,
  from S and the generations in the activator's facts, and compares it
  with the slot records there (hearth/docs/design/domain/slots/README.md, 2.3). A change to the kernel's
  command line alone changes no output and still needs a new slot.
- **Re-blessing comes first.** While the running slot is not the
  default, prepare is refused (hearth/docs/design/domain/slots/README.md, 2.3), so the pass first asks
  the activator to bless the running slot again, which moves the
  default to it (hearth/docs/design/domain/slots/README.md, section 4, and hearth/docs/design/domain/slots/bless.md, section 1.3). It runs even under a hold
  (5): it changes nothing but makes the boot default agree with what
  runs.
- **The desired output is pinned** while it is desired, and the
  outputs the activator reports for its slots are pinned too, so
  collection never takes what a slot runs or a pass is about to place
  (hearth/docs/design/domain/store/README.md, 5.1).
- **Compare before building.** Most commits change nothing for most
  targets: their output key is the running slot's, and the pass ends in
  milliseconds after the fetch (`hearth/docs/design/draft/archive/factory.md`, 6.2).
- **A prepared slot is reused.** When the inactive slot already holds
  S, untried (a manual trial, or a pass interrupted by a restart), the
  pass goes straight to trying.
- **The pass ends at the reboot.** The trial slot's own steward starts
  afresh, reads its state and the activator's facts, and its pass begins
  in `blessing`, with a render job of the slot's own revision, offline
  (hearth/docs/design/domain/steward.md, section 6). A broken hearth release in the new slot
  cannot bless itself, whether its steward never asks or its builder
  cannot render the revision that built the slot, so it cannot strand
  the machine.
- **A failed unit the target expects does not fail a trial.** A
  desktop often carries one that fails routinely; the target lists it
  in `bless.ignore_failed`, and the minimal bless counts only the
  others (hearth/docs/design/protocol/format/README.md; hearth/docs/design/domain/slots/bless.md, 1.1).
- **Activation is the cheapest that covers the change.** Live first,
  when hearth/docs/design/domain/slots/trial.md, 7's rules hold: a builder plans it, as a job, from
  the two outputs' tree files and the live parts of the running and the
  new spec (hearth/docs/design/domain/render/spec.md, 1.4; hearth/docs/design/domain/steward.md, 3.3), and the activator checks
  again. Otherwise the activator chooses a soft reboot or a one-shot
  reboot by hearth/docs/design/domain/slots/trial.md, 1's rule (the kernel's digests and its command
  line). `status` shows which a prepared slot needs, and why it is not
  live. A reboot is a clean `systemctl reboot` with
  `SYSTEMCTL_SKIP_AUTO_SOFT_REBOOT=1` (hearth/docs/design/domain/slots/trial.md, 3); only the dead-man
  and a failed live fall back use `systemctl reboot -ff`.
- **A live bless ends the pass converged** with the base still the
  default: the next boot, whenever it comes, is the new slot's ordinary
  trial, whose own pass begins in `blessing` (hearth/docs/design/domain/slots/trial.md, 8, L9). The
  pass makes no reboot for it.
- **Effects are keyed or conditional.** prepare is keyed by target and
  revision; a trial is conditional on the facts the pass read. A pass
  that restarts in the middle finds what it made and goes on.

## 3. One target at a time

- **A pass converges one target, then the next.** In M3 a machine runs
  one target. In M5 a host runs its containers' targets too (7), and
  the loop takes them in a fixed order, one pass each.
- **One prepare and one trial per machine at a time.** A target has one
  inactive slot, the store builds within its own bounds
  (hearth/docs/design/domain/build/README.md, section 4), and a trial may reboot the whole machine.
- **A slot-changing command from the client while a pass is in
  `preparing` or `trying` is refused as busy** (exit 75). One that comes
  while the pass fetches, renders or builds stops the pass at its next
  stage, and the command goes first.
- **A client's `render`, `diff` or `build` is a job too:** it waits in
  the steward's queue, in the machine's own lane, behind the job in
  flight there (hearth/docs/design/domain/steward.md, 3.4).

## 4. Failures and backoff

Classes, as hearth/docs/design/domain/render/errors.md, 1 and hearth/docs/design/domain/build/README.md, 4.3 type failures:
*operational* (hearth/docs/design/domain/build/README.md's transient): a tool or a resource failed, and
the same input may succeed later; *configuration*: the revision or the
secret store is wrong and must change; *refused*: the slot itself cannot
be made or run, so its prepare key is refused.

| Failure | Class | What follows |
|---|---|---|
| `git fetch`: the remote unreachable, a timeout | operational | back off; a new pass |
| the commit or the branch unknown | configuration | wait until the desired revision changes |
| render: a decode, schema, evaluation, guard, collision or resolution error, such as a package apt cannot locate, or a snapshot timestamp whose archive lacks the suite (hearth/docs/design/domain/render/spec.md, 2.5, and hearth/docs/design/domain/render/errors.md, 2) | configuration | wait for another commit: render is deterministic, so retrying cannot help |
| render: git or the resolution's mmdebstrap failed, timed out or could not start; fetching the indices failed with no answer or a broken one; a name did not resolve (hearth/docs/design/domain/render/spec.md, 2.5, and hearth/docs/design/domain/render/errors.md, 2's `tool`) | operational | back off |
| a build's fetch failed: the network, a 404 from the mirror, a digest mismatch (hearth/docs/design/domain/build/README.md, 4.3) | operational | back off |
| a build step failed: a maintainer script, `customize.sh`, a limit, the scan (hearth/docs/design/domain/build/README.md, 4.3) | refused | refuse the prepare key |
| out of space | operational | an alarm; collection (hearth/docs/design/domain/store/README.md, 5.2); back off |
| a secret the spec declares is missing at prepare (hearth/docs/design/domain/slots/prepare.md, 4) | configuration | wait for the secret store's generation to change: an import |
| the activator found a layer that does not match its record (hearth/docs/design/domain/activator/README.md, section 5) | none: the store is corrupt | never a refusal: a builder's verify job hashes the layer again and, if it is corrupt, deletes it and its descendants, and the pass builds again (hearth/docs/design/domain/store/README.md, 4.4; hearth/docs/design/domain/steward.md, 3.3) |
| a job lost twice alone: its builder died or hung under it (hearth/docs/design/domain/steward.md, 5.1) | `crashed`, classed by its job: a fetch's, a verify's or a collection's operational; a render's configuration; a build's refused | back off; wait for another commit; refuse the prepare key. Reported with the job, its stage and the exit status or signal |
| the trial fell back: no bless within the window, or a failed unit `bless.ignore_failed` does not list; or a live trial fell back (hearth/docs/design/domain/slots/bless.md, 2.5) | refused | refuse the prepare key |
| a change not eligible for live, or a live trial whose delta failed to build or verify (hearth/docs/design/domain/slots/trial.md, 7 and 8) | none: not an error | an ordinary trial; why it was not live is a fact |

- **Backoff** applies to operational failures only. It is jittered and
  doubles from 1 minute to a ceiling of an hour (starting values), and
  starts over when a pass converges or the desired revision changes. A
  branch's poll and the backoff are separate timers: a pass starts when
  both allow it. Render's own overall deadline for a resolution (10
  minutes, hearth/docs/design/domain/render/spec.md, 2.5) bounds one attempt.
- **A refused prepare key is never tried again on its own.** Refusals
  are kept in the steward's state, bounded (the last 16), each with why
  and how long ago. A new commit that makes the same prepare key stays
  refused; one that makes another, by its output, its slot parts or a
  generation, is tried. This is what stops a boot loop.
- **A refusal never covers the running slot.** It names the slot that
  failed, prepare key and all, so a rotation that falls back (6)
  refuses only the new generation's slot, and the running one stays
  converged.
- **An operator clears a refusal:** `hearth retry <target>` clears the
  target's refusals, and the next pass tries again; `hearth trial` and
  `hearth prepare --at` act on a refused key when asked, and clear it if
  the trial is blessed. A failed unit that fails every boot belongs in
  `bless.ignore_failed`, not in a retry.
- **A hearth bug is not a crash loop.** A revision that crashes a
  builder ends, after one retry, in a wait or a refusal like any
  failure of its class, while the steward stays up, keeps its faces and
  reports the crash with the revision that caused it (hearth/docs/design/domain/steward.md, 5.1).
- **A failed pass is a fact and an event** (`wire.md`, section 5), with its stage
  and its typed reason. `status --check` exits 1 while one stands.

## 5. The local hold

- **`hearth hold` stops convergence.** No new pass starts. A pass in
  flight stops at its next stage; a build stops after its current step,
  and what it built stays in the store.
- **It never stops a trial from ending.** A slot on trial is blessed or
  falls back whatever the hold says: a hold cannot keep a machine on an
  unblessed slot.
- **It stays until `hearth release`,** across restarts and reboots, in
  the steward's state, with who held it (the peer's uid), when, and the
  reason given.
- **It does not stop the operator.** `prepare`, `trial` and `rollback`
  still work while held. A manual change that convergence should not
  undo is made under a hold.
- **A roll back needs no hold.** It refuses the prepare key it rolled
  back from (4), so the loop does not try that slot again; a new
  commit with another key, a fix or a revert, converges as usual. This is the
  standalone form of the run side's rule that a roll back holds a
  machine until the desired state matches again (`hearth/docs/design/draft/archive/factory.md`,
  section 9).

## 6. Secrets rotated

- **Rotating a secret is a new slot of the same output** (hearth/docs/design/decisions.md
  ). `hearth secrets import` bumps the secret store's
  generation, which the activator reports to its steward as a fact.
- **The next pass sees the running slot's generation behind,** which
  changes the prepare key (hearth/docs/design/domain/slots/README.md, 2.2, and hearth/docs/design/domain/slots/prepare.md, 4), and prepares a new
  slot from the output it already holds: no build, a snapshot and a
  shift (hearth/docs/design/domain/slots/prepare.md, 1 and 2), then a trial, which is a soft reboot
  when the kernel is unchanged; never live, since a rotated secret is
  not in the output's diff (hearth/docs/design/domain/slots/trial.md, 7, rule 2).
- **A rotation that falls back** refuses only its own prepare key (4).
  The running slot, at the older generation, stays; the next import
  bumps the generation again and makes another key to try.
- **A generation is a number, never a digest of a value,** so no fact
  says anything about a secret's contents.

## 7. Containers [M5]

- **A host's spec names the containers it runs** (`target.containers`,
  hearth/docs/design/protocol/format/README.md, 3.5). Each is a target with slots of its own on the host's
  top level, which the host's activator prepares (hearth/docs/design/domain/slots/containers.md, 1).
- **The loop takes the containers first, one at a time, and the host's
  own target last,** since the host's trial may reboot the machine and
  interrupt the rest.
- **One desired revision for the machine.** The host and its containers
  converge to the same commit, with the same trial policy (1): a
  container is a target of the host's revision, rendered and built by
  the host's builders, in the host's store (hearth/docs/design/domain/build/README.md, section 2), and
  placed by the host's activator at its steward's request.
- **A container's trial is live when eligible** (hearth/docs/design/domain/slots/containers.md, 8), the
  host's activator swapping the overlays inside the container's mount
  namespace and acting on its units through `systemctl --machine=`.
  **Otherwise it is a restart on its new root,** watched, with a fall
  back to the old root if it does not come up healthy, as the bash
  fleet's `switch-container` does (hearth/docs/design/domain/slots/containers.md, 2 and 3). The host's
  hearth judges both from outside; no hearth runs in the container in
  M5 (hearth/docs/design/domain/slots/containers.md, 5).
- **Its bless is in the same pass.** Where a host's pass ends at its
  reboot, a container's goes on: once the restart or the live swap has
  answered, the pass
  waits for the minimal criterion and the container's blessing checks,
  read from the host, and asks the host's activator to bless, or to fall
  back on a failed unit that counts (hearth/docs/design/domain/slots/containers.md, 3). A container's
  fell-back prepare key is refused as a host's is (4).
- **Each container's output key is compared on its own,** so a commit
  that changes only one container's target leaves the host and its
  other containers alone.
- **A container's first slot** is prepared and tried by its host's pass
  like any other, with nothing to fall back to (hearth/docs/design/domain/slots/containers.md, 3); its
  secrets are imported for its own target (hearth/docs/design/client/README.md, 3.6).
- **A container the spec no longer places on the host** is removed in
  the same pass, before the host's own target: the steward asks its
  activator to stop it and delete its slots, which unpins their outputs,
  and its data stays, reported as orphaned (hearth/docs/design/domain/slots/containers.md, 7).
- **A development machine may run a selection** of its containers, set
  by `hearth containers`, by `retarget` or by `hearth dev up`
  (hearth/docs/design/domain/development.md, 3.4), and refused on a machine that does not accept
  mutable targets, so the run side's hosts always run their whole spec.
  The pass converges only the selected containers; one left out is
  never built, prepared or started, and one that ran is removed as
  above, its data kept, and reported as left out rather than orphaned.
  The host's own spec, output and prepare key are unchanged.
