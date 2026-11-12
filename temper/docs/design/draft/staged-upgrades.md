# staged-upgrades: dev.fabrig.ai moves to a new temper through staging.fabrig.ai, its store kept

Draft, 2026-11-11. How temper's own two deployments take a new build without starting afresh: dev.fabrig.ai, where temper's development runs (`first-release.md`, section 5), and staging.fabrig.ai, a deployment of its own for checks before production. The mechanism is `upgrades.md`'s, which goal 648 builds with its baseline moved to the store `main` wrote at `bc23255eb7`. This draft says what the first upgrade needs beyond that goal, how it is proved before it reaches dev.fabrig.ai, and how every later one runs. It comes from a session with the user on 2026-11-11, who asked for it after goal 648 lands: the store patches dev.fabrig.ai's store needs to reach `main`, and staging at dev.fabrig.ai's build to test the path.

## 1. In one page

- **dev.fabrig.ai's store is already at the baseline** (section 2): its build's stored layouts are `bc23255eb7`'s, so once goal 648 lands it reads as level 0 and nothing before the baseline needs patching.
- **The catch-up** (section 3): every stored layout `main` changed in place after the baseline and before goal 648's gate lands comes in as a patch from the baseline. Today that is goal 63's change to three of jig's families and goal 187's to temper's.
- **dev.fabrig.ai holds at its build** until it upgrades (section 4): a fix taken there keeps every stored schema as the baseline's.
- **Rehearsed on dev.fabrig.ai's own data** (section 5): the new build's `temper upgrade --check` over a copy of its backup, decoding every record of every family, not only the patched ones.
- **Drilled on staging.fabrig.ai** (section 6): staging runs dev.fabrig.ai's build with a small workload of its own, then takes the new build through a settled stop, the patching start and a restore, by a script whose referee says pass or fail.
- **Then dev.fabrig.ai** (section 7), with a backup taken first and the previous build kept.
- **Every later upgrade the same way** (section 8): staging one step ahead of dev.fabrig.ai, never reset, both stores kept.

## 2. Where the two deployments stand

| | Build | Its store |
|---|---|---|
| dev.fabrig.ai | `54d36edb95`: `main` at `e8e952b8ac` with goals 409 and 580 | made 2026-11-10, kept since; its stored layouts are the baseline's |
| staging.fabrig.ai | `main` at `2e28582bd2` | fresh, empty; its layouts are past the baseline (goal 63's) |

- **Why dev.fabrig.ai's store is at the baseline:** the thirteen stored families' schemas at `54d36edb95` and at `bc23255eb7` are the same text, and the stored crates differ by one test fixture. Its build records jig's layouts in the store and refuses a store whose layouts differ (jig's `protocol/store/records.md`, 3), and dev.fabrig.ai opens, so the layouts its store records are the baseline's.
- **What else it carries across, unchanged to `main`:** the configuration (limits' defaults grown, a model's cache life optional; `upgrades.md`, section 10), the forge's setup and its Forgejo's pins, so git.fabrig.ai's Forgejo migrates nothing (`upgrades.md`, section 11), smith's turns, so agents resume (`upgrades.md`, 7.2), and temper's secrets.
- **staging.fabrig.ai is made again at dev.fabrig.ai's build** (section 6): a store past the baseline at no level is refused as an older build's, and staging's holds nothing yet.

## 3. The catch-up

- **What it is:** each stored layout `main` changed in place after `bc23255eb7`, landed before goal 648's gate, becomes a patch from the baseline in goal 648's landing, with its line in the golden store's table of patches. The gate finds each one once `main` is merged into the goal's branch: a schema whose layout differs from its kept text with no patch fails it (`upgrades.md`, section 12).
- **Today, goals 63 and 187,** on `main` since 2026-11-11: goal 63 (standing roles) changes three of jig's families (jig's `domain/tasks/standing.md`, section 7), and goal 187 (rules by path) one of temper's (`../domain/forge/change/gates.md`):

| Family | Change | Every old record takes |
|---|---|---|
| `jig-store-tasks` 9 | `TaskRecord.activation_open`, `TaskRecord.session`; the activity lines `stood` and `session_ended` appended | `activation_open` false, `session` none: no task before it stood |
| `jig-store-settings` 1 | `Model.cache_life_ms` inserted; `EndedBy`'s `standing` appended | `cache_life_ms` 0, the configuration's "none declared" |
| `jig-store-people` 14 | `EndedBy`'s `standing` appended | nothing: an appended variant |
| `temper-store` 11 | `LandingRule.sets` appended | `sets` empty: a rule naming no set covers every change, as every rule did before |

- **Goals in flight that change a stored layout in place** (on 2026-11-11: 625, `jig-store-people`'s project deletion; 64, `temper-store`'s follows): one that lands before goal 648 joins the catch-up; one that lands after brings its own patch, the gate refusing it otherwise.
- **The first patch of jig's own families:** goal 648's stories patch with a fixture that changes no family, so the catch-up is the first real one. Its line over the golden store, made by the baseline's code, is its proof.
- **Every landing passes the gate:** a merge into `main` outside the queue runs the gate's patches test too, or an unpatched change lands past it.
- **Carried by goal 648** where it fits before it lands; otherwise by a goal right after it, before any upgrade.

## 4. dev.fabrig.ai holds at its build

- **No upgrade from `main` until the catch-up lands:** dev.fabrig.ai runs `54d36edb95`.
- **A fix it takes meanwhile** is taken onto that build and changes no stored schema: `git diff --stat bc23255eb7 <fix> -- '*/schema/*.schema'` prints nothing. One that did would leave a store no build of `main` can patch.

## 5. The rehearsal on dev.fabrig.ai's data

- **A backup first:** dev.fabrig.ai's deployment directory, the active generation and the transcripts, copied with the engine stopped (`upgrades.md`, 3.3).
- **Then `temper upgrade --check`** by the new build, over a copy of the backup, patching into a scratch directory on the same machine as the deployment's own user (`upgrades.md`, 3.1). Nothing starts and nothing leaves the machine. It reports the patches and their counts, how long patching took, what it refused, and whether the workers may stay.
- **Every record decoded:** the check decodes every record of every kept family by its version's kept text, whether or not a pending patch changes the family, and names the first that does not decode with its key. The store's layouts row records jig's, skein's and smith's families but not temper's, and `temper-store` changed in place five times while dev.fabrig.ai's store was kept (fields appended to kept forge answers among them). A record left in an older layout refuses the catch-up's walk of `temper-store` (goal 187's patch) at dev.fabrig.ai's start; decoded beforehand, it is found in the rehearsal, with every other family's.
- **Its numbers set the drill's bounds:** the service manager's start timeout covers patching, and its stop timeout the settled stop (`upgrades.md`, section 8).

## 6. The drill on staging.fabrig.ai

- **Made again at dev.fabrig.ai's build,** on a fresh store, with its own sign-ins.
- **A workload of its own,** given before the drill and kept: a scratch repository on git.fabrig.ai under staging's own deployment id, one worker, one LLM account on the cheapest stock model, a goal through a landing, a chat that replied and waits, and a coding task mid-run. The user agreed to its worker and account on 2026-11-11.
- **Never dev.fabrig.ai's store:** a copy carries dev.fabrig.ai's deployment id, so its branches and markers on the forge, and its secrets; started, it would act as dev.fabrig.ai. dev.fabrig.ai's data goes only to the rehearsal (section 5).
- **The drill,** one script, run once per new build:
  1. Its own backup (section 5), then a settled stop.
  2. The new build swapped in, engine and worker, and started: it patches a copy, opens it, and switches (`upgrades.md`, 6.1).
  3. **The referee,** as `upgrades.md`, section 13's story: the deployment's and the installation's ids; the person signed in; the account and its source; the project and its repository; the landing recorded or going on; the chat's turns read; the mid-run task's attempt crossed and its next started; the next task number above every kept one; no effect made twice on the forge; the start's patch lines equal to the rehearsal's.
  4. **A restore:** stopped, the backup put back, the previous build started on it, and the same referee against the state before the drill; then the new build again. Going back after the new generation's first commit is a restore (`upgrades.md`, 3.2), so this is dev.fabrig.ai's way back.
- **Its output:** one line per check and a verdict.

## 7. dev.fabrig.ai's upgrade

1. The rehearsal (section 5) and the drill (section 6) pass on the build to deploy.
2. A backup, taken with the engine stopped.
3. The settled stop, the new build swapped in for the engine, and the start, which patches as the rehearsal did. The workers stay or follow as `redeploys.md`, section 7 says; until it is built, they are swapped together with the engine.
4. The referee's checks that need no workload of their own: ids, sign-ins, accounts, projects, the start's patch lines, goals going on.
5. **Going back:** before the new generation's first commit, `temper upgrade --back` and the previous build; after it, the backup restored with the previous build, as staging rehearsed (`upgrades.md`, 3.2).

## 8. Every later upgrade

- **Staging runs one step ahead:** dev.fabrig.ai's build until a new build is chosen, then that build through the drill, then dev.fabrig.ai follows.
- **Neither store is reset.** staging's grows through every build, so its drills cross patch after patch, as dev.fabrig.ai's will.
- **Each build to deploy** is rehearsed on a fresh backup of dev.fabrig.ai (section 5), then drilled (section 6).

## 9. Who carries what

| Part | Owner |
|---|---|
| The catch-up and the full decode (3, 5) | goal 648, or the goal after it |
| Backups, staging's machine and its making again (5, 6) | the deployments' operator |
| The rehearsal, staging's workload and drill script, dev.fabrig.ai's upgrade (5 to 8) | temper/migration |

## 10. Open questions

- **temper's families' layouts in the store's row,** as jig's are, so a store is refused at open rather than at a patch's walk. Default: not now; the full decode (section 5) finds them first.
- **How often staging drills:** per build deployed to dev.fabrig.ai, or per landing of a stored change. Default: per build deployed.
- **staging's workload grown with temper:** each new kind of stored record, such as a follow, added to it when it lands. Default: so, by the goal that adds the record.
