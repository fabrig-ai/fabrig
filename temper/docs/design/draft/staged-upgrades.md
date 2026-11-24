# staged-upgrades: dev.fabrig.ai moves to a new temper through staging.fabrig.ai, its store kept

Draft, 2026-11-11. How temper's own two deployments take a new build while dev.fabrig.ai keeps its store: dev.fabrig.ai, where temper's development runs (`first-release.md`, section 5), and staging.fabrig.ai, a deployment of its own for checks before production. The mechanism is `upgrades.md`'s, bounded by the floor (`store-floor.md`, section 3). This draft says how each upgrade is proved before it reaches dev.fabrig.ai and how every later one runs. It comes from a session with the user on 2026-11-11, who asked for staging at dev.fabrig.ai's build to test the path.

## 1. In one page

- **dev.fabrig.ai's settled store is the floor** (section 2). A store without the floor or below it is refused, naming bridge build `3d1c717d379e`, before planning or writing an upgrade.
- **Registry changes and jobs** (section 3): a registered schema's added field takes a default, a renamed part its former name, and a change no plan derives its owner's written job. No store patch or repository-kept store history is carried.
- **dev.fabrig.ai holds at its build** until the rehearsal and drill pass (section 4).
- **Rehearsed on dev.fabrig.ai's own data** (section 5): the new build's `temper upgrade --check` over a copy of its backup, decoding every record of every family, whether or not a job changes it.
- **Drilled on staging.fabrig.ai** (section 6): staging is made again at dev.fabrig.ai's build for each drill, with a small workload of its own, then takes the new build through a settled stop, the upgrading start and a restore, by a script whose referee says pass or fail.
- **Then dev.fabrig.ai** (section 7), with a backup taken first and the previous build kept.
- **Every later upgrade the same way** (section 8): staging one step ahead for the drill, dev.fabrig.ai's store kept; jobs' code removed after dev has run it.

## 2. Where the two deployments stand

- **dev.fabrig.ai's store is settled at the floor:** store-floor's F0 brings every registered value to its build's layout, retires the other layouts and writes the floor (`store-floor.md`, sections 2 and 3, with its status's departures). The build opens that store and newer ones, not a chain of older stores.
- **An older store is refused by name.** A missing or lower floor, a store behind the binary's levels or values without a registry name bridge build `3d1c717d379e` to open through first. Forms that the bridge also refuses are named without that promise (jig's `protocol/store/opening.md`, 1.8). Nothing upgrades the refused store.
- **What else it carries across:** the configuration (`upgrades.md`, section 10), the forge's setup and its Forgejo's pins (`upgrades.md`, section 11), smith's turns, so agents resume (`upgrades.md`, 7.2), and temper's secrets at their current version. A change of a format outside the registry brings how dev's store gets there in its landing (`store-floor.md`, 5).
- **staging.fabrig.ai is made again at dev.fabrig.ai's build for each drill** (section 6). It is not an older deployment to keep upgrading through floors.

## 3. Registry changes and jobs

- **Derived changes** use the store's texts: an added field takes its default (`= value`), and a renamed declaration or field names its former name (`was`). The registered family's version and the owner's level stay. The open registers the binary's layout and derives plans; values convert as read, not by a catch-up walk (jig's `protocol/store/layouts.md`, 4 and 5).
- **A change no plan derives** brings its owner's written job, as do changes of keys or meaning (`upgrades.md`, 4.3; jig's `protocol/store/jobs.md`, 4). A written conversion reads its values as stored, by the texts the registry holds, and puts them at the binary's layout, in the copy before the switch. Each job's done row keeps it from running twice.
- **After dev.fabrig.ai runs a job,** a small landing removes its code. Its name stays used and its done row stays inert (`store-floor.md`, section 6). No store patch, old registered text or old generated codec is kept for an older deployment.
- **Every landing passes the gate** (`upgrades.md`, section 12): defaults, former names or jobs for registered families; the wires keep their patches and bridges, and formats outside the registry stay fixed at their versions.

## 4. dev.fabrig.ai holds at its build

- **No upgrade until the rehearsal and staging's drill pass** on the build to deploy (sections 5 and 6).
- **A fix it takes meanwhile** follows the same rules as any landing: a registered schema changes by defaults, former names or a job (section 3), wires by their bridges, and a format outside the registry only with its designed way for dev's store to reach it. It does not reopen a promise to stores below the floor.

## 5. The rehearsal on dev.fabrig.ai's data

- **A backup first:** dev.fabrig.ai's deployment directory, the active generation and the transcripts, copied with the engine stopped (`upgrades.md`, 3.3).
- **Then `temper upgrade --check`** by the new build, over a copy of the backup, working in a scratch directory on the same machine as the deployment's own user (`upgrades.md`, 3.1). Nothing serves, no effect is made and nothing leaves the machine. It reports the jobs and their counts and time, the whole time, what it refused, and whether the workers may stay. Registry planning, jobs, the check and restores run as a start's; the rehearsal stops before the switch and says that what follows it is not rehearsed (jig's `protocol/store/opening-rehearsal.md`, 1).
- **Every record decoded:** after the floor and format checks and before planning, the check decodes every registered value by its layout's text in the store's registry, and every other record by its current format; frames by the frames' texts. This covers every family, whether or not a pending job changes it. It names the first that does not decode with its family and key, or transcript and offset; otherwise it reports the count decoded (jig's `protocol/store/opening-rehearsal.md`, 1). A store the open refuses as older is refused before this decoding too, naming `3d1c717d379e` (`staged-upgrades.md`, section 2).
- **Its numbers set the drill's bounds:** the service manager's start timeout covers the registry's writes, the jobs, the check and restore, and its stop timeout the settled stop (`upgrades.md`, section 8).

## 6. The drill on staging.fabrig.ai

- **Made again at dev.fabrig.ai's build for each drill,** on a fresh store, with its own sign-ins (`store-floor.md`, section 6). No staging store is kept through a floor move.
- **A workload of its own,** given before the drill and kept: a scratch repository on git.fabrig.ai under staging's own deployment id, one worker, one LLM account on the cheapest stock model, a goal through a landing, a chat that replied and waits, and a coding task mid-run. The user agreed to its worker and account on 2026-11-11.
- **Never dev.fabrig.ai's store:** a copy carries dev.fabrig.ai's deployment id, so its branches and markers on the forge, and its secrets; started, it would act as dev.fabrig.ai. dev.fabrig.ai's data goes only to the rehearsal (section 5).
- **The drill,** one script, run once per new build:
  1. Its own backup (section 5), then a settled stop.
  2. The new build swapped in, engine and worker, and started: where a layout needs registering or a job is pending, it upgrades a copy, opens it, and switches (`upgrades.md`, 6.1).
  3. **The referee,** as `upgrades.md`, section 13's story: the deployment's and the installation's ids; the person signed in; the account and its source; the project and its repository; the landing recorded or going on; the chat's turns read; the mid-run task's attempt crossed and its next started; the next task number above every kept one; no effect made twice on the forge; the start's job lines and counts equal to the rehearsal's on staging's own backup.
  4. **A restore:** stopped, the backup put back, the previous build started on it, and the same referee against the state before the drill; then the new build again. Going back after the new generation's first commit is a restore (`upgrades.md`, 3.2), so this is dev.fabrig.ai's way back.
- **Its output:** one line per check and a verdict.

## 7. dev.fabrig.ai's upgrade

1. The rehearsal (section 5) and the drill (section 6) pass on the build to deploy.
2. A backup, taken with the engine stopped.
3. The settled stop, the new build swapped in for the engine, and the start, which registers the layouts and runs the jobs as the rehearsal did. The workers stay or follow as `redeploys.md`, section 7 says.
4. The referee's checks that need no workload of their own: ids, sign-ins, accounts, projects, the start's job lines and counts, goals going on.
5. **Going back:** before the new generation's first commit, `temper upgrade --back` and the previous build; after it, the backup restored with the previous build, as staging rehearsed (`upgrades.md`, 3.2).

## 8. Every later upgrade

- **Staging runs one step ahead:** dev.fabrig.ai's build until a new build is chosen, then that build through the drill, then dev.fabrig.ai follows.
- **dev.fabrig.ai's store is kept; staging is made again for each drill.** It starts at dev's build, never below the floor (`store-floor.md`, section 6). A drill keeps its workload through the candidate's upgrade and its restore, not through later drills.
- **Each build to deploy** is rehearsed on a fresh backup of dev.fabrig.ai (section 5), then drilled (section 6).
- **After dev runs the jobs,** their code goes in a small landing. Dropping a whole older form moves the floor and its named bridge together (`store-floor.md`, section 6); another store left behind goes through that bridge or starts afresh, by the operator's choice.

## 9. Who carries what

| Part | Owner |
|---|---|
| Registry changes and jobs (3) | each changed family's owner |
| The full decode (5) | jig's store and temper's `upgrade --check` |
| Backups, staging's machine and its making again (5, 6) | the deployments' operator |
| The rehearsal, staging's workload and drill script, dev.fabrig.ai's upgrade (5 to 8) | temper/migration |

## 10. Open questions

- **How often staging drills:** per build deployed to dev.fabrig.ai, or per landing of a stored change. Default: per build deployed.
- **staging's workload grown with temper:** each new kind of stored record, such as a follow, added to it when it lands. Default: so, by the goal that adds the record.
