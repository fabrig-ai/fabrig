# ui-reaches: what reaches you, its three classes and Away

The request: `inputs/request.md`, milestone N1 of `temper/docs/design/draft/whole-story.execution.md`, executing `temper/docs/design/draft/what-reaches-you.md`, sections 2 to 6. `notes/common.md` holds the rules every task follows; `notes/contracts.md` the shapes that cross tasks.

## Parts and tasks

Released:

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `kit` | `tasks/reach-kit.md` | | the crate `temper-protocol-reaches`: kinds, classes, a project's table, patterns, `judge`, Stop asking's offers; their tables and the fuzz test | 120 |
| `wire` | `tasks/reach-wire.md` | | `ProjectClasses` and `Reach` on the client wire, `ProjectClasses` and `SettingValue::classes` in the store | 60 |
| `classes` | `tasks/classes-value.md` | `reach-kit`, `reach-wire` | the kit's record conversions; the forge's `Value::Classes`, kind 8's number, stock, judgement and erasure; the store's codec | 90 |
| `classes` | `tasks/classes-kind.md` | `classes-value` | kind 8 declared to jig, taken at the wire's entrances and in a project's reads; `project-classes` for a run's `settings`, `see` and `propose_settings`, Ask temper's charter | 120 |
| `classes` | `tasks/classes-tools.md` | `classes-kind` | `temper ctl settings show project-classes`, settings files, the help pages | 105 |
| `classes` | `tasks/classes-ways.md` | `classes-kind` | the stock ways lay kind 8's tables | 60 |
| `decide` | `tasks/decide-forge.md` | `classes-value` | Away without kinds at every layer; `decided` judged by the classes, a row for every call, decided or asked, kept 7 days under `forge.decided_per_project`; the document's `rows`, `asked` and `cut`; the client's controls of kinds removed | 180 |
| `decide` | `tasks/decide-root.md` | `decide-forge`, `classes-kind` | the caller's person for any task, `Asks: always` judged; the tool's ten kinds, `paths` and `under`; its words naming the reach; the charters; the engine world's table of who decides and what reaches | 150 |
| `keep` | `tasks/keep-asking.md` | `decide-forge` | the person's fact `KeepAsking`, a second replacing the first | 75 |

Released from the client's designer (its design: `temper/docs/design/client/README.md`, 3.5, 5.8, 5.13, 5.14, sections 10 and 11; `browser.md`, 7.2):

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `client` | `tasks/decided-client.md` | `decide-forge`, `classes-value` | one page-held store of the `decided` watches; Decided for you for 24 hours on Home and Needs you, across projects, "Allowed:" from the reach, Overturn from either page; Home's rows whether away or not | 160 |
| `client` | `tasks/away-client.md` | `decided-client`, `classes-kind`, `classes-value`, `decide-forge` | kind 8 read with the project; Away shown by the classes; Settings' section a view of the table; kind 8 in proposal cards; 5.14's choice by the classes | 150 |
| `client` | `tasks/needs-why.md` | `away-client`, `decided-client` | each need's reason line, both shells | 140 |
| `client` | `tasks/stop-asking.md` | `needs-why`, `keep-asking` | Stop asking in place, applied by an owner or proposed; Keep asking for 7 days | 150 |
| `client` | `tasks/reaches-story.md` | `decide-root`, `stop-asking` | the client world's story of N1's acceptance | 100 |

Still being written: `ways-away` (kind 7's `away` member gone end to end), after `stop-asking` and `keep-asking`.

The client's tasks run one after another, since each writes the view's `words.rs`, `look.rs`, `shared/reach.rs`, `temper.css` and the gallery's `samples.rs`, `tests.rs` and `names.txt` (the designer's call, kept: those files don't merge well across writers).

**Critical path as it stands:** `reach-kit` (120, landed) → `classes-value` (90) → `decide-forge` (180) → `decided-client` (160) → `away-client` (150) → `needs-why` (140) → `stop-asking` (150) → `reaches-story` (100), about 16 hours of implementer time from here; `classes-kind`, `classes-tools`, `classes-ways`, `decide-root` and `keep-asking` run beside it.

## What was decided

- **One pure crate for the judgement,** `temper-protocol-reaches`, since both the forge's `decided` and the client read a project's classes: the Away switch's showing, the reason line and Stop asking's offers come from the same table and the same function (`temper/docs/design/domain/forge.md`, 5.10).
- **Kind 8, a project's classes,** a new kind of temper's, rather than members of kind 7: kind 7 is read by the client alone and the engine decides nothing by it, where kind 8 is judged in every `decided` (`temper/docs/design/protocol/configuration.md`, 13.2).
- **`paths` is no call's kind:** a call names its own kind and the paths it would touch, and each path set they touch is judged as an item of its own; the strictest class wins (5.10). The draft's table lists `paths` as a kind "per path set", which this keeps.
- **A rule in words** is an item with words narrowing its subject; code can't check words, so a call names the item it decides under (`under`), and that item's class stands for its subject (5.10).
- **Path sets are the project's,** their patterns over any of its repositories' paths, since the draft's example names one repository's tree and N2's rules by path judge a change in its own repository.
- **The bounds** of a table: 20 items, words 120 bytes; 6 sets, names 40 bytes, 6 patterns of 80; a call's paths 16 of 256 bytes. A full table's encoding is kind 8's bound.
- **A stored format and the wire change freely** (the request): kind 8's record and the store's variant are new; later batches change `Away`, `SetAway`, `Decided` and its document.

- **Every judged call writes a row,** decided or asked, so that the question that follows a `decided` that said ask can say why it came: the forge's `decided` document answers both lists (`rows`, `asked`, 128 each, and `cut`), and the client matches a need to its asking task's newest asked row (`temper/docs/design/domain/forge.md`, 5.9).
- **The bound, 1,024 a project,** shared by decided and asked rows, kept 7 days; at the bound the oldest row past its day goes, and with none past its day the call is answered ask and writes nothing, as the draft has it. A `took` past the options writes no row.
- **`Asks: always` reaches the forge** as `always`, judged as every item asking, so its need says "this goal always asks"; the charters now tell such a goal to call `decided` too, every answer being Ask.
- **The stock project's table is Agents review's,** the way of kind 2's stock rule 1 (`configuration.md`, 13.4), and each stock way sets its own (13.10).
- **The tool's Ask names a set by its number,** "changes to path set N", since a call's answer is rendered from its kept reach, which holds no name; the agent reads the name with `see settings project-classes`.
- **Away's kinds go in one task,** `decide-forge`, at every layer at once (the forge's row, the store, the wire, the client's controls), so that no trunk state judges by bits it no longer sends; the client's view of the table is `away-client`'s, and kind 7's `away` member, which the classes now replace, goes last (`ways-away`).
- **Left as the draft's lags, named in its status:** a standing role's person, waiting for R1; the paths an asker's change touches at its head, which N2's rules by path read (`whole-story.execution.md`'s N2 now names it); the TUI's reason line, temper/tui's with T1.
- **Schema writers in sequence:** `reach-wire`, `decide-forge`, `keep-asking`, `ways-away` (`notes/common.md`), since generated code and goldens don't merge.
- **Two tasks in one crate at once, on disjoint files or hunks:** `classes-kind` beside `decide-forge` (the wire's `lib.rs`: `settings_kind` against `SetAway`'s validation; smith's `calls.rs`: the settings enums against `decided`'s decoder; `temper-engine-domain`: reads and `see` against `SetAway` and the call's answer), and `classes-ways` beside `decide-forge` in `temper-service/src/tests/family.rs`, each on its own cases.

## Overlaps with drafts beside this one

- **`temper/docs/design/protocol/configuration.status.md`** is also `ignored-engine/restore-stories`': this cut adds its entries at the file's head, far from that task's 13.2 entry.
- **`temper/docs/design/domain/forge.status.md`** is also `look-round-24`'s (`asked-sentence`, merged; `ci-workflows`; `rule-view`): this cut adds its entries after the 5.8 and 5.9 departure, apart from theirs.
- **The client wire's schema and generated code** were changed by `look-round-24/ways-wire` and `tui-box-front/forges-linked` (both merged on their trunks): `reach-wire` adds records after `ProjectWays` alone; a merge that meets theirs regenerates rather than hand-merges.

- **`temper-engine-domain-ways`:** `look-round-24/ci-workflows` changes `path.rs`, `path/derive.rs`, `tests/derivation.rs` and `tests/fuzzy.rs`; `classes-ways` changes `ways.rs`, `lib.rs` and `tests.rs` alone.
- **`temper/tests/engine/domain/tests/suite/ways.rs`** is also `look-round-24/rule-view`'s: `classes-ways` changes its expected stock items' rows alone. Unavoidable: the stock ways' items are what it checks.
- **`temper/tests/engine/domain/tests/suite/settings/kinds.rs`** and **`temper/tests/client/tests/suite/settings_tools.rs`** are `ignored-engine/restore-stories`': this plan's kind 8 rows go to `configuring.rs` and the settings form's own tests instead.
- **`temper/docs/design/client/tools.status.md`** is also `restore-stories`': this cut's entry is at the file's head.
- **`temper-service`** was changed whole by `ui-engine-split`'s merged tasks on their trunk; this plan touches `family.rs`, `limits.rs`, `stock/*` and their tests, a merge meeting theirs taking both.

- **`temper/tests/client/tests/suite/away.rs`:** `decided-client` changes its readers of Home's facts and `decide-root` only the assertions of the tool's words, which may run at once; different lines.

## Who hears first

**`temper-engine-domain`** (the owner of coding runs writes it too):
- `classes-value`: compile-forced arms for `Value::Classes` in `route/kinds.rs` (`way_refusal`, `approval_for_projects`) and `route/ways.rs` (`project_ways`), kind 8 treated as kind 7 is where kind 7 isn't asked for.
- `classes-kind`: `route/kinds.rs`'s `read_refusal` takes kind 8; `route/see.rs` parses `settings project-classes`; `route/host_route.rs`'s newest-settings fallback answers kind 8's stock; `tests/kinds.rs`.
- `decide-forge`: `boundary.rs`'s `Application::SetAway` loses `kinds`; `CallAnswer::Decided` gains `reach` (`store.rs`, `translate.rs`); `route/applications.rs`; `route/decided.rs` hands `always: false`; **`route/forge_route.rs`**: one arm, `forge::Request::Decided` passing `reach` on; forced arms elsewhere (`route/host_route.rs`, `route/see/forge.rs`, `route/playground.rs`, `route/ways.rs`, `admission.rs`).
- `decide-root`: `route/decided.rs`'s person rule for any task with a person, and `always`.

**`temper-client-domain`** (the TUI's owner):
- `decide-forge`: `AwayRow.kinds`, `DEFAULT_KINDS`, `forge::away::ask`'s `kinds` and `pages::away::Act::Kinds` removed; fixtures of `v1::Decision` take a `reach`.
- `keep-asking`: compile-forced arms for the wire's `PersonFact::keep_asking` in the dismissals' readers.
- `decided-client`: `pages::decided`, a page-held store of the `decided` watches; Home's facts' `away` becomes `decided` and `spell`; `AwayForm`, `DecisionLine` (with `project` and `reach`) and `DecisionTask` move to `pages::decided`; Overturn on Needs you.
- `away-client`: kind 8's settings slot, `Side::classes`, `forge::classes`; `away_shown` by the classes.
- `needs-why`: `pages::needs::why` and `ready`; the shown card's classes slot; `ways::progress`'s new argument.
- `stop-asking`: `pages::stop_asking`; `PersonFact::KeepAsking` sent and read.
- `ways-away`: `Ways` without `away`.
