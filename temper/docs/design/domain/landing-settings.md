# A landing's settings

How a project's landing is set: its way and its repositories' gates and
tiers, temper's settings kinds 4 and 5, and the operator's hold that
lets a deployment set them before anything lands. Moved out of
`landing.md`, section 13, on 2026-11-16, when the document's
`application.landing` became these kinds (`../draft/config.md`, 6.5;
`../protocol/configuration/moved.md`).

## 1. Settings

### 1.1 Kinds

Two kinds of temper's join `../protocol/configuration/settings-kinds.md`, 1, each
the forge's, judged in the change's decision:

| Kind | Scope | Numbers | Value |
|---|---|---|---|
| 4, a project's landing | each project | 1 | the members below |
| 5, a project's gates | each project | 1 | `repositories`, each one of the project's repositories at most once: the repository (its forge and id); `gates`, its test gates, each a name, the workflow's file, its ignored paths, whether it blocks, and whether it reads the base (`base`, no when absent), and its `report`, if any: an artifact, its format and its fuzzy tests' name prefix (`landing-proof.md`, 2.3; `landing-search-by-test.md`, 2), each proving the trains into its main (`landing-proof.md`, 2.1); `tiers`, each a name, the workflow's file, its ignored paths, the artifact and its format, the branch it follows (`after-landing-runs.md`, section 1), its span and its description |

Kind 5 was a repository's gates, numbered by the repository's adoption
packed in 32 bits beside its forge's number. A repository's id is the
forge's own, 32 bits (skein's `forge/vocabulary.md`, 1.1), so it doesn't
pack, and kind 5 is the project's: one value holding each of its
repositories (2026-11-16).

Kind 4's members, each with its default and the section it serves:

| Member | Default | Serves |
|---|---|---|
| `by`: `queue` or `turn` | `queue` | `landing.md`, section 1 |
| `batch`: the most changes a landing into a repository's main forms with, from 1 to the deployment's `landing_members` | 1 | `landing.md`, 4.1, `landing-turn.md`, section 2 |
| `combine`: how a train combines its members (`landing-prepare.md`, 1.2), `merge` or `squash`, `main` for the trains into a repository's main and `other` for those into any other branch | `merge`; `squash` | `landing-prepare.md`, 1.2, and `landing-turn.md`, 1 |
| `prepare`, `key`: paths in the home, or none; `home_revision`; `prepare_timeout` | temper's own prepare and key | `landing-prepare.md`, 1.3, `landing-proof.md`, 1.1 |
| `merge_job`: a path in each repository it merges that holds no `.temper/hooks/`, or none; a repository with one runs its `merge` hook and ignores it (`../protocol/hooks.md`, 5) | none: a conflict goes to a resolution at once | `forge/change/repairs.md` and `forge/updates.md`, 1 |
| `close_job`: a path in each repository a goal's branch is in that holds no `.temper/hooks/`, or none, as `merge_job`; `landing_attempts`, 1 to 255 | none; 3 | `forge/branches-closer.md`, 1 |
| `check_timeout`: the delivery checks' wall bound in a run delegated for one of the project's changes, or none | none: the agent's own | `../protocol/configuration/engine.md`, `landing` |
| `hook_timeout_max_seconds`: the ceiling on any of the project's hooks' time, whole seconds from 1 to 86,400, and its merge and close jobs' deadline | 3,600 | `../protocol/hooks.md`, 2.3 |
| `flakes`: whether the project's checks can flake, `none` or `rerun N`, N from 0 to the deployment's `checkout.rerun_processes` | `rerun` at that figure | `forge/judgement.md`, 4 |
| `dependencies_join` | yes | `landing.md`, 4.1 |
| `pins`: each a downstream and an upstream repository; `pins_follow_main` | none; no | `landing.md`, 4.1, `landing-prepare.md`, 1.3 |
| `gate_retries`; `gate_timeout` | 2; one hour | `landing-proof.md`, 2.4; `landing-proof.md`, 2.1 |
| `wait`: `cleared` or `landed`, whether a change into a branch a tier follows lives to cleared or ends at landed | `cleared` | `after-landing.md`, section 6 |
| `tier_retries`; `verification_bound`; `suspects`: `bisect` or `tell_all`; `red_pauses` | 2; two hours; `bisect`; no | `after-landing-runs.md`, 2, `after-landing.md`, 5.2, 6, 7 |
| `findings`: on or off; `triager` and `chooser`, a task or a role; `tester_charter`; `areas`, each a repository and an area | off; the person's chat; the chat; none | `findings.md`, 2, 6, 8 |
| `repairs`: `repair_goals`, `repair_attempts`, `repair_deadline` and `goal`, the repair goal's shape; refused without `wait: landed` | none; when set 2, 2, 24 hours, `goal` required | `red-repairs.md`, section 7 |
| `questions_on_forge` | off | `forge/issues.md`, 1 |
| `temper_dir_sign_off`: whether a change into a landing branch touching temper's method in `.temper/` also waits for one owner's approval on the forge; on is refused where no owner can sign | off | `forge/temper-dir.md`, 5 |

- **What is built:** kind 4's `by`, `batch`, `combine`, `merge_job`,
  `close_job`, `landing_attempts`, `check_timeout`,
  `hook_timeout_max_seconds`, `flakes`,
  `gate_retries`, `gate_timeout`, `wait`, `tier_retries`,
  `verification_bound` and `repairs`, and kind 5
  whole. Each field keeps the form, bounds and default
  `../protocol/configuration/engine.md`'s `landing` gives it, where the
  document's `application.landing` held them. A start now refuses that
  member (`../protocol/configuration/moved.md`, section 3). The table's
  other members join kind 4's record as each is built, each an added
  field with its default (`AGENTS.md`, 2).
- **Taken at the next start.** The forge builds each project's landing
  from kinds 4 and 5 as they stand when it restores its settings, and
  keeps it until the next start. A change of either, accepted while the
  engine runs, is stored and waits. Gates and tiers are positional in
  the proofs and trains in flight (`landing-proof.md`, 2.1, and `landing-turn.md`, 2), so
  a change mid-run would leave them naming another gate. The
  settings-fields page and each kind's line say "at the next start".
- **Judged when proposed,** by the forge with the root's facts, each
  refusal naming its field. It refuses what a start checked of the
  document's member: a bound, a path's form, a gate's or a tier's name,
  a repository given twice, `batch` past `landing_members`, `flakes`'
  N past the deployment's `checkout.rerun_processes`. It also
  refuses `turn` without the limits its prepare job needs
  (`../protocol/configuration/engine.md`, *Prepare under `turn`*), and a
  repository that isn't the project's. A removal or a reset is always
  good: the project lands by the defaults above.
- **Kind 5 fits jig's settings carrier.** Each of its fields keeps its
  own bound, from the deployment's `forge` limits, but their product can
  pass what one application setting value carries (jig's
  `ApplicationValue`, `client-v26-part1.schema`, 1,114,240 bytes): four
  repositories of 64 gates and 32 tiers, each with four ignored paths
  of 1,024 bytes, already do. So the judge also refuses a kind 5 value
  whose encoding passes that carrier, naming `repositories` and the
  bound, read from the carrier's generated limit rather than restated.
  Every count's own maximum stays; only the whole is capped, and the
  default profiles fit far within it (2026-11-16).
- **Named** `project-landing` and `project-gates`, with the `project`,
  in a settings file and a run's `settings`, `see` and
  `propose_settings`. Their values are JSON with the names and forms of
  `../protocol/configuration/engine.md`'s `landing` members. Kind 4's is
  an object of its fields, as `{"by": "turn", "batch": 4, "merge_job":
  "tools/merge"}`, `flakes` as `"none"` or `{"rerun": N}`. Kind 5's is `{"repositories": [{"forge": F,
  "repository": R, "gates": [...], "tiers": [...]}]}`, each gate and
  tier as that section writes one.
- **Read by a branch's path** (`forge/change/branch-path.md`): the project's `by`,
  and each repository's gates and tiers with the branch each tier
  follows, as the forge holds them, so that a path draws "prepare",
  "prove" and "turn" under `turn` and a tier's stops where it has one.
- **`turn` is refused** for a project with a repository where temper
  cannot fast-forward main to a candidate (`landing-turn.md`, section 1: on GitHub, no
  `Always` bypass of a rule requiring a pull request, or a required
  merge queue; or an installation that cannot write statuses or
  dispatch workflows), naming the repository and the setting that
  would let it land.
- **A gate's name** is letters, digits, `-` and `_`, unique in its
  repository, and its status context's last segment.
- **The stock's deployment ceiling** admits what landings and tiers are
  granted (kinds 3, 4, 6, 7, 9, 12, 15, 16 and 17, and the forge's
  procedures `landing` and `tier`), since a project's ceiling may not
  pass the deployment's; each grant grows every row that carries
  authority, so none is added that a release at close can spare.

### 1.2 What adoption says

- **Under `turn`,** for each repository: its gates, each workflow
  checked to exist at the landing branch's tip and to name
  `workflow_dispatch` alone among its triggers, and, with a report, to
  declare `temper_tests`, or said otherwise in words; whether the forge marks a pull request merged once its base
  holds its head (Forgejo's "autodetect manual merge"), which lands a
  merged member's pull request (`landing.md`, section 10); where one of the project's
  trains squashes, whether the forge lets a pull request be recorded
  merged by hand (Forgejo's "allow manual merge"), which a squashed
  member's record needs (`landing-turn.md`, section 1); and whether the branch's rules
  admit the landing's pull request with the gates' statuses on the
  candidate.

### 1.3 Held by the operator

A deployment's operator may start its engine with every landing held,
so that the settings a landing reads can be set before anything lands
under the stock's. dev.fabrig.ai's redeploy that moves its landing into
kinds 4 and 5 does (`../draft/config.execution.md`, N5).

- **A file says it:** `landing-held` in the data directory, any content,
  read at each start. When it is there, the engine says once on stderr
  `landing held: <path>; remove it and restart to land`. For that whole
  run the forge holds every landing: no train forms, no queue merge is
  made and no landing task is made, into any branch. A goal's branch
  and the goal closer's land step are held too. Everything else runs:
  changes are made, produced, reviewed and checked, and settings
  change.
- **A change ready to land waits.** Its panel says `landing held until
  the engine starts without landing-held`, where a waiting change says
  why it waits (`forge/change/panel.md`, 1). Nothing it waits for is
  lost: at the next start without the file it lands as it would have,
  under the settings in force then.
- **Removed, it ends at the next start,** never while the engine runs.
  The hold belongs to the start, as kinds 4 and 5 do, so the restart
  that takes them takes the release too.
- **A test** gives the fact in its world's construction, as the box's
  `under_box` is given.

## 2. Open questions

- **Kinds 4 and 5 taken at once.** A change waits for the next start
  because gates are positional in the proofs in flight. Default: stays
  so until a design reconciles a train's gates with a changed list.
