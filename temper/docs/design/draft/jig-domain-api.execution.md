# jig-domain-api: execution

2026-11-17, brought up to date on 2026-11-26. How [jig-domain-api.md](jig-domain-api.md) gets carried out: the engine first as a pass-through every root adopts, then port by port, then the clean-up (the draft's section 15). It designs nothing: each milestone writes its own design from the draft into jig's documents and their status as it runs, and lands whole. Plans use the focused and fuzzy suites alone. A milestone's figure is the draft's section 16 counts it lowers, read on main once it lands.

## 1. The milestones

Each is a goal, one change of one concern. A milestone that grows past a day's change splits, and this file says how. Every milestone that moves a port moves all three roots in the same goal.

| Milestone | Draft parts | Stream | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| J1, the engine as a pass-through: `Engine` in `jig-domain-core` over `Core`, with `step`, `due`, `fire`, `next_deadline` and `idle` keeping today's clocks and timers' admission, `In::Legacy` and `Out::Legacy` carrying today's `Event` and `Request` and nothing else, and a temporary access to the whole `Core`, shared and exclusive, by method and not by `Deref`; the counts of the draft's section 16 measured again, each with its command from the draft's notes written here, those a pattern finds added to jig's and temper's old-forms ratchets (`jig/scripts/old-forms.txt`, `temper/scripts/old-forms.txt`) | 4.1, 4.5, 15, 16 | engine | none | the counts' baseline | done, by goal 19159 (`jig/docs/design/domain/engine/vocabulary.md`, 2; the counts and their commands, below); its figure is read after landing |
| J2, jig's roots on the engine: `ops`'s root and the test root hold `Engine` in place of `Core`, and the kits and worlds that drive the core (`jig-core-kit`, `jig-conformance`, `jig-test-service`) drive it; no behaviour moves | 15 | roots | J1 | counts 1 and 3 | done, by goal 19160 (`jig/docs/design/domain/root.md`, 3; `jig/docs/design/domain/testing.md`, 4); its figure is read after landing, count 1 now reading the pass-through's `In::Core` and `Out::Core` in both roots, as the draft's section 16 counts `Legacy`'s |
| J3, temper's root on the engine: `temper-engine-domain` and the engine service's crates that drive the core hold `Engine`; no behaviour moves | 15 | roots | J1 | counts 1 and 3 | done, by goal 19161 (`jig/docs/design/domain/engine/vocabulary.md`, 2.3): temper's root and the tests that built a `Core` hold `Engine`; its figure is read after landing, temper's count 1 adding the pass-through's `In::Core` and `Out::Core` and its count 3 losing `due`, `next_deadline` and `quiescent` |
| J4, the store's loads: one `Load` and its one `Loaded` with a typed result, paced by the engine; store answers' order and a released callback's place among them kept as named predicates with their tests; the ten named loads and their `busy` flags gone | 4.4, 5.1, 18 | engine | J2, J3; the user's word on one order for every root (the draft's section 18), or its default | counts 1, 2, 3 and 8 | not started |
| J5, the contract's crate: `jig-domain-contract`, below the children and the core's vocabulary and above `jig-domain-link` alone, one module per capability and `ops`, with the operations' values and the connector contract's types moved down and re-exported along their old paths, deciding helpers left with the core; programming-model 4.5's and 4.6's wording of the draft's 14.1 | 6.1, 14.1 | contract | J1 | none of its own: moves | done, by goal 19163 (`jig/docs/design/domain/connectors.md`, 2; `skein/docs/foundation/programming-model.md`, 4.5 and 4.6); `requests` gets its module with its asks (J7), and `jig/docs/design/domain/root.md`, 6's translation goes as connectors speak the contract (J7) |
| J6, the declarations: the `Declaration` given at construction and checked, effect kinds' form, recovery and waiting taken from it and from the connector's configuration; roots' installing of hold kinds gone; resources' roles and holds still said up, and members' holdings still judged | 6.2 | engine | J4, J5 | counts 1, 4 and 5 | not started |
| J7, the asks: one `Ask` of nine capabilities, each case a judgement, a notification or an operation that ends later, one `Up` with `Up::Act`, `Released`; a connector asked only about the capabilities it declared; the decision's assertion over judgements alone; each root's asks routed with one arm per connector; `ops`'s connectors and the test connector speaking it, temper's as the draft's section 18 says | 6.3 to 6.5, 6.7, 14.1, 18 | engine | J6 | counts 1, 2, 5 and 8 | not started |
| J8, the start through the ports: each phase's exchange a port's (the engine's loads, the generation's switch, the wires and the start's marker, the journal's numbering restored, what waits held by admission, each connector restored and caught up by its lifecycle's asks, loading its own records), the script's order kept; roots' `restart.rs` gone | 5.1, 5.5 | engine | J4, J7 | counts 1, 2, 3 and 8 | not started |
| J9, a run's brief: the preparation context; each named connector planning its sections, the engine placing them within the budget, then gathering and cutting what it placed; `BriefCorePlanned`, the gathers and `CompleteBrief` gone, and the roots' fixed answers to them | 6.6 | engine | J7 | counts 1, 2 and 8 | not started |
| J10, a run's workspace and claim: workspace items with their writes, holds and credentials in the same planning; the claim checked again as it is made; run preparation inside the engine, its activations paced by it; `assemble` one function in each root; `WorkspaceRequest`, `WorkspacePrepared`, `RunPrepared` and `ClaimPrepared` gone | 4.4, 6.6, 10 | engine | J9 | counts 1, 2, 3 and 8 | not started |
| J11, the engine's own slots as a worker: a worker root of the hub and `smith-inline-agent`, with no workspace, in the engine's process under its own protocol layer and an in-process address; the host port in the core's vocabulary, the channel's wire unchanged; `jig-domain-hub-link`'s tables (saved parts among them) and retry deadlines moved to the worker's path, then the hub, the inline agent and the link gone from the engine's root | 5.2, 9 | hosts | J8, J10 | counts 1, 2, 7 and 8 | not started |
| J12, decoded at the edge: calls, turns and answers decoded by the engine's protocol layer as they arrive, within today's bounds; a connector's call in two parts, staged inert; fencing after decoding, keeping what binds a call; disposal on every path; a settled call's rendered bytes kept; charters' sizes checked at configuration; the hand-outs and payload tables gone, decoded calls carried as today's actions until J13 | 8, 14.2 | hosts | J11 | counts 1, 2 and 8 | not started |
| J13, the operations: `Act`, `Actor`, `Op`, one `Read` and one refusal's shape, in today's shapes only, with authority's table of which actor may make which operation; made by the clients' component and by a run's decoded call; `people::Ask` and `NamedAction` gone | 7.1 to 7.3 | engine | J5, J12 | counts 1, 2 and 3 | not started |
| J14, procedures' steps and the parties' records: a procedure's envelope around its operations, `ProcedureStep`'s lists gone; a party's retained requests and acted records in the new shape, keys judged equal or conflicting as today, `Seen` keeping its rule, each record changing as `AGENTS.md`, 2 says a registered schema changes | 7.1, 7.4 | engine | J13 | counts 1 and 2 | not started |
| J15, the remaining ports: parties' (5.3), accounts' (5.4), the application's (5.5) and `Resume` (5.6); `Legacy` empty | 5.3 to 5.6 | engine | J14 | count 2 at 0 | not started |
| J16, one class per input: `class` and `idle`; `ops`'s connectors' protocol layers delivering a settlement and its news apart, each connector keeping the observation its news is judged against; `Class::Mixed`, the split and the baselines gone, the three mixed events' outcomes and stories kept | 4.4, 14.4 | engine | J15 | count 1 | not started |
| J17, the engine is the API: `Legacy` and the access go, the core emitting the ports itself, the old vocabulary deleted, and `In` and `Out` renamed skein's `Event` and `Request` | 4.5, 15 | engine | J16 | counts 2 and 3 at 0 | not started |
| J18, private children: `jig-boundary`'s check that no production crate but the core, its vocabulary, the contract and the children names a child's crate, its children's own test kits and worlds allowed; every other dependent through the core's vocabulary, its re-exports listed; `jig/docs/design/README.md`, 4's escape hatch gone | 10, 14.3 | inside | J17 | counts 4, 5 and 6 at 0 | not started |
| J19, one continuation: a flight's id as the one reply token, the claim and procedure holdings moved into it, the core's own work apart from `In` (D002), and the re-narrowing arms that go with them (D017) | 10 | inside | J18 | D002's measure | not started |
| J20, `Core` grouped: its state in owned parts by concept, each with its own `impl` (D003) | 10 | inside | J19 | count 10's `Core` | not started |
| J21, a task's record: its common fields and one executor part (D014), changing as `AGENTS.md`, 2 says a registered schema changes | 10 | inside | J20 | count 10's `TaskRecord` | not started |
| J22, the conformance kit: it answers jig's ports itself, and an application's adapter is its step, its release, its two functions to and from jig's ports and its systems' fakes (D030) | 12 | kit | J17 | count 9 | not started |
| J23, the reference root: `ops`'s root copied again to the draft's section 11; `jig/docs/design/domain/root.md`, `jig/docs/design/domain/engine/README.md`, `jig/docs/design/domain/connectors.md` and `jig/docs/design/domain/hosts/README.md` read whole against the new edge; a guide for an application's author, from the draft's section 3, in `jig/docs/design/README.md`, 11; D004 closed | 11, 17 | roots | J21, J22 | count 8 | not started |

## 2. Order

- **One milestone, every goal in it at once** (the user, 2026-11-26: "start executing it with a single milestone", with work "in parallel when it makes sense to do so and it's reasonably safe"). Each goal waits for the goals its row needs, and temper starts it when they have ended.
- **No priority mark:** jig's domain API isn't one of v0.1.0's critical pieces. Its goals take places when they are free.
- **Side by side,** where the files they change are apart:
  - J2 and J3, which change jig's roots and temper's, and J5 beside both, since its moves keep every old path;
  - J4 beside J5;
  - J8 beside J9 and J10: the start's phases and a run's preparation change different files of the three roots (`restart.rs` against `assemble.rs` and the brief's routes) and different cases of the old vocabulary;
  - J22 beside J18 to J21, once J17 has landed.
- **One at a time otherwise,** each from main once the one before it has landed: every port moves the same files of the three roots, so no two branches change them at once. The inside stream is J19, J20 and J21 in that order, since all three change the core's readers of a task's record. J11 waits for J8 as well as J10, since the start makes the links the hub's move changes.
- **What runs beside it** (dev.fabrig.ai on 2026-11-26): the landing train has landed. The milestone judging whose red it is still changes temper's root and the release of a held change (`whose-red.md`, 6), and another party's goals can't be a goal's dependencies, so J3 moves no behaviour and takes those changes in as they land; J6 and J7 come after J3 and J4.
- **The user's word on one order for every root** (the draft's section 18) comes before J4 starts. Without it, J4 takes the default and says so in its design.
- **Each milestone lowers its counts** and none raises one; a count the ratchet holds fails the gate if it rises.
- **This file's states:** each goal marks its own row, so goals side by side don't change the same line. The draft's status file is rewritten by J23 alone.

## 3. The counts

Baseline measured at `98305c3322` (2026-11-26). The commands below read one Git revision, `HEAD` by default. For a per-root figure, the three roots are `ops`, the test root and temper, in that order. Set up `jig_api_count` below from the repository root before running the numbered commands; for example, `jig_api_count 1 98305c3322` reads count 1 at the baseline.

1. **Distinct edge variants named in the roots' non-test sources:** 212, 317 and 252; **349** in their union. `jig_api_count 1`.
   As in [the notes](jig-domain-api.notes.md), section 3, this reads cases of `Ask`, `Held`, `Now`, `Request`, `Write` and `Event` under `core::`, `jig_domain_core::` (normalized to `core::`) and `connector::`. It also reads `In::` and `Out::` cases whether bare or module-qualified, including `Legacy` once named in a root. None is named yet.
2. **Variants carried through `Legacy`:** **270**. `jig_api_count 2`.
   These are 76 `Event`, 38 `Ask`, 46 `Held`, 40 `Now`, 8 `LinkNow` and 5 `Write` in `vocabulary.rs`, and 38 `Ask` and 19 `Event` in `effects/connector.rs`.
3. **Distinct core methods called in the roots:** 23, 25 and 67; **73** in their union. `jig_api_count 3`.
4. **Accesses to children's state in the roots:** 17, 36 and 386; **439** in total. `jig_api_count 4`. The `child_state` ratchet records **53** in jig and **386** in temper.
5. **Children's events named in the roots:** 15, 16 and 78; **109** in total. `jig_api_count 5`. The `child_event` ratchet records **31** in jig and **78** in temper.
6. **Production manifests outside the core, its vocabulary, the contract and the children naming a child's crate:** **27** (77 in the broader count that includes test kits). `jig_api_count 6`.
7. **Non-test lines in `jig-domain-hub-link`:** **1,309**. `jig_api_count 7`.
8. **Non-test lines in the roots, and in temper's wiring:** 3,671, 4,762 and 34,683; **43,116** in total. Of temper's lines, **13,019** are in 31 wiring files. `jig_api_count 8`.
9. **Functions of the conformance kit's `Application`:** **26**. `jig_api_count 9`.
10. **Fields in `Core` and `TaskRecord`:** **99** and **96**, respectively. `jig_api_count 10`.

Counts 3 and 4 read `core.`, `core().` and `core_mut().` alike, and count 3 excludes the two access methods themselves (`jig/docs/design/domain/engine/vocabulary.md`, 2.3). Counts 4 and 5 read source text, while the ratchets strip comments; the figures agree at this baseline. To check the ratchets in the working tree, run `(cd jig && python3 -I -B scripts/old_forms.py)` and `(cd temper && python3 -I -B scripts/old_forms.py)`.

```bash
jig_api_count() {
  python3 -I -B - "$@" <<'PY'
import re, subprocess, sys, tomllib

count = int(sys.argv[1])
rev = sys.argv[2] if len(sys.argv) > 2 else 'HEAD'
roots = ['jig/examples/ops/domain/src/', 'jig/testing/jig-test-domain/src/',
         'temper/crates/temper-engine-domain/src/']
children = 'accounts authority brief fleet notes people tasks views'.split()


def git(*args):
    return subprocess.check_output(['git', *args], text=True)


def read(path):
    return git('show', f'{rev}:{path}')


def source(prefix):
    for path in git('ls-tree', '-r', '--name-only', rev, '--', prefix).splitlines():
        if path.endswith('.rs') and not re.search(r'/(tests|fuzzy)/|/(tests[^/]*|fuzzy)\.rs$', path):
            yield path, read(path)


def body(path, decl):
    return re.search(r'^pub '+decl+r' \{\n(.*?)^\}', read(path), re.M | re.S)[1]


if count in (1, 3, 4, 5):
    access = r'\bcore(?:\s*\(\s*\)|_mut\s*\(\s*\))?\s*\.\s*'
    state = access + r'(?:' + '|'.join(children) + r')\b'
    event = r'\b(?:jig_domain_)?(?:tasks|fleet|people)::Event::\w+'
    legacy = r'\b(core|jig_domain_core|connector)::(Ask|Held|Now|Request|Write|Event)::([A-Z]\w*)'
    port = r'\b(In|Out)\s*::\s*([A-Z]\w*)'
    union, total = set(), 0
    for root in roots:
        text = '\n'.join(t for _, t in source(root))
        if count == 1:
            hits = {('core' if q == 'jig_domain_core' else q, name, case)
                    for q, name, case in re.findall(legacy, text)}
            hits.update(('engine', name, case) for name, case in re.findall(port, text))
        elif count == 3:
            hits = set(re.findall(access + r'([a-z_]\w*)\s*\(', text)) - {'core', 'core_mut'}
        else:
            hits = re.findall(state if count == 4 else event, text)
        print(root, len(hits))
        if count in (1, 3):
            union.update(hits)
        else:
            total += len(hits)
    if count in (1, 3):
        print('union', len(union))
    else:
        print('total', total)
elif count == 2:
    v = 'jig/crates/jig-domain-core-vocabulary/src/'
    specs = {v+'vocabulary.rs': 'Event Ask Held Now LinkNow Write',
             v+'effects/connector.rs': 'Ask Event'}
    total = 0
    for path, names in specs.items():
        for name in names.split():
            cases = set(re.findall(r'^    ([A-Z]\w*)\s*(?=[({,]|$)', body(path, 'enum '+name), re.M))
            print(path, name, len(cases))
            total += len(cases)
    print('total', total)
elif count == 6:
    names = {'jig-domain-' + child for child in children}
    excluded = names | {'jig-domain-core', 'jig-domain-core-vocabulary', 'jig-domain-contract'}
    dependents = git('grep', '-l', '-E', r'^jig-domain-('+'|'.join(children)+r')[. =]',
                     rev, '--', '*Cargo.toml').splitlines()
    paths = [entry.split(':', 1)[1] for entry in dependents]
    broad = sum(path != 'Cargo.toml' and not re.search(r'/jig-domain-core(?:-vocabulary)?/', path)
                for path in paths)
    production = 0
    for path in paths:
        if path == 'Cargo.toml' or re.search(r'/(tests|testing|benchmarks)/', path):
            continue
        doc = tomllib.loads(read(path))
        if doc.get('package', {}).get('name') not in excluded and names & doc.get('dependencies', {}).keys():
            production += 1
    print('production', production, 'broad', broad)
elif count == 7:
    print(sum(t.count('\n') for _, t in source('jig/crates/jig-domain-hub-link/src/')))
elif count == 8:
    wiring = re.compile(
        r'^(admission|assemble|boundary|deadline|decision|domain|journal_stop|lib|limits|link|'
        r'loads|push|restart|settling|start_wait|stopping|translate|wire|route)\.rs$|'
        r'^route/(asks|held|now|parts|opened|public|host_route|applications|document|see)\.rs$|'
        r'^route/(parts|document)/[^/]+\.rs$')
    total = 0
    for root in roots:
        lines = {p[len(root):]: t.count('\n') for p, t in source(root)}
        root_total = sum(lines.values())
        print(root, root_total)
        total += root_total
        if root == roots[-1]:
            chosen = {p: n for p, n in lines.items() if wiring.search(p)}
            print('temper wiring', len(chosen), 'files', sum(chosen.values()), 'lines', chosen)
    print('total', total)
elif count in (9, 10):
    specs = {
        9: [('jig/testing/jig-conformance/src/harness.rs', 'trait Application', r'^\s+fn ')],
        10: [('jig/crates/jig-domain-core/src/lib.rs', 'struct Core',
              r'^\s+(?:pub(?:\(crate\))? )?[a-z_]+:'),
             ('jig/crates/jig-domain-tasks/src/boundary.rs', 'struct TaskRecord', r'^\s+pub [a-z_]+:')],
    }
    for path, decl, pattern in specs[count]:
        print(decl, len(re.findall(pattern, body(path, decl), re.M)))
else:
    sys.exit('expected a count from 1 to 10')
PY
}
```
