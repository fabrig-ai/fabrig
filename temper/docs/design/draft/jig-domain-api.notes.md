# jig-domain-api: notes

The reasons behind [jig-domain-api.md](jig-domain-api.md), the map of today's edge it rests on, and the commands behind its counts.

## 1. Reasons

- **What the user asked** (2026-11-17): "this session is about jig domain layer only, protocol and client set aside for now. I have the impression that it grew organically, and would like to see what we can do about it", and first, "given the functionality that jig offers, what would be its ideal API ... If we were to design it greenfield ... Still in line with skein's programming-model.md and still in line with the kit idea".
- **The shape** of the draft's sections 4 to 12 was the first answer; the user took its five departures (the draft's section 14) as recommended: "okay I'm convinced".
- **The way there** was the user's: "Create this new API as a facade over existing code, with no consumer yet, then slowly migrate all consumers ... and then cleanup making the facade the real API". The draft changes one step, giving the facade every root at once as a pass-through (the draft's section 15), since the parts it must absorb differ per root and a facade with no consumer is checked by nothing.
- **The aim, sharpened** (the user, 2026-11-26): "The end goal is a sane and nice public facing API for jig, the kind of thing you'd want to see if you started a project consuming jig. Ideally concerns are clearly separated, jig is the core and takes care of that, connectors are built on top and consume it / augment it." The draft's section 3 answers it from an author's side, and its section 6 is arranged for a connector's author.
- **The review round** (one, 2026-11-17) found fifteen gaps, all taken: the start's own exchanges (5.1); typed load terminals and stated orders (5.1); timers' admission and clocks (4.1); declarations narrowed to what is static, with holdings and roles still judged (6.2); asks of three kinds (6.3); a run's preparation context, planned then gathered, checked again at its claim (6.6); acts only in today's shapes, procedures' envelopes, actors' permissions and keys (7.1, 7.4); a settled call's rendered bytes kept (8); disposal on every path of a decoded call (8); the crate graph kept acyclic (6.1); `jig-domain-hub-link`'s tables moved before it goes (9); the stored-record rule of `AGENTS.md`, 2 (10, 15); the children's own worlds (10); the map's holes; and the order's splits. The update of 2026-11-26 had no review round of its own.
- **The update of 2026-11-26** followed a read-only survey of main at `de275134b0`, which checked every claim the draft made about code and documents. A week's work had added waits and their bounds, holds' words, worker drains, suspensions and saved parts, layouts named by fingerprint, a connector's catching up in place of two restart stages, task lines' parts and the settings the service applies, and had split the documents sixteen of the draft's citations named. Each is in the draft where its port or capability is, and the counts are taken again.
- **The draft's readings,** not the user's words:
  - the engine in `jig-domain-core` rather than a crate of its own, so that no dependent changes for a name (4.1);
  - the operations' values in the contract crate, so that a procedure's author names jig's operations as a party's client does (6.1);
  - **capabilities in place of the twelve families of asks** (6.3), so that a connector's author reads only what it takes part in and the engine asks only what was declared; the families mixed concerns (one held adoption, projections and a closing task's effects; another split a run's parts from its claim), and an application's smallest connector is its lifecycle and one capability;
  - typed numbers where an author meets them, and a connector's number as a type of its own (6.1, 7.1);
  - skein's names, `Event` and `Request`, for the engine's input and output once the old ones are gone (4.1), so that jig reads as every other step machine;
  - what a connector can rely on, and what it promises, said once (6.7);
  - today's names kept out of the draft's tables and in the map below, so that the draft reads as the API rather than as its move;
  - the engine's own slots as a worker in its process, so that every host is reached one way, the crate graph stays acyclic and `jig-domain-hub-link` has nothing left to do (the draft's section 9);
  - a connector's own observation for news, so that a settlement and its news can arrive apart (14.4);
  - and the defaults of the draft's section 18.

## 2. Where today's edge goes

At `de275134b0`, outside tests.

| Today | Where it goes |
|---|---|
| `Event`: `ReadLoaded`, `TranscriptLoaded`, `SeenConversationLoaded`, `EndedReadLoaded`, `DecidedProposal`, `EndedProposal`, `EndedTaskLoaded`, `PartyInputsLoaded`, `EndedPersonTask`, `EndedEscalation`, `DelegateValidated`, `DelegateInputFailed`, `Notes(..)`'s pages and failed loads | `In::Store(Loaded)`, its typed result (5.1) |
| `Event`: `Fill`, `Census`; `Tasks(Restore)`, `Tasks(Restored)`, `People(Restore)`, `SettingsCommit` | `In::Store` and the start's phases (5.1) |
| `Event`: `Fleet(..)`, `RunCall`, `RunInput`, `RunText` | `In::Host` (5.2) |
| `Event`: `TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload`, `PartPayload`, `NamedAnswer`, `SettledCall` | decoded at the edge: `In::Host`'s turns, answers and calls, a settled call's rendered bytes (the draft's section 8) |
| `Event`: `NamedAction`, `SubscribeTopic` (its read resources checked by authority), and `Notes(..)`'s writes, edits, index and recalls | `Act` or `Read` (the draft's section 7) |
| `Event`: `SignIn`, `Read`, `Watch`, `WatchRun`, `Unwatch`, `Delivered`, `StreamDelivered`, `PublicRead`, `People(..)` | `In::Party` (5.3) |
| `Event`: `Key`, `ChangeModel`, `ChangeSettings`, `ProposeSettings`, `Application` | `Act` by `Actor::Party` (7.1) |
| `Event`: `Act`, `PersonTask`, `StartRecurring` | `Act` by `Actor::Deployment` (7.1) |
| `Event`: `EffectStart`, `ProcedureStep`, `Marked`, `View(..)`'s documents made and subjects changed, `ApplicationPart`, `ProjectionSettled`, `Brief(..)`, `BriefAssembled`, `WorkspacePrepared`, `ClaimPrepared`, `ClaimRefused`, `ConnectorRepair`, `Waits` | `In::Connector(Up)` (6.4), or gone with run preparation (6.6) |
| `Event`: `Account(..)` | `In::Account` (5.4) |
| `Event`: `Period`, `Stopping`, `Settling` | `In::Application` (5.5) |
| `Event`: `PublicDurable`, and `Tasks(..)` and `Fleet(..)` callbacks fed back | `In::Resume` (5.6) |
| `Event`: `Activate`, `EffectDeadline`, and `Tasks(..)` that roots build | inside: scheduled by the engine, fired by `fire`, or the declarations and `Up` (6.2, 6.4) |
| `Request::Write`, and the start's wires | `Out::Write` (4.3, 5.1) |
| `Request::Ask`'s `Connector` | `Out::Ask` and its nine capabilities (6.3) |
| `Request::Ask`'s `Workers` and `Providers` | `Peer::Application` (5.5) |
| `Held`: `Load`, `ReadLoad`, `NotesLoad`; `Now`: `StartPreparation`, `SeenConversation`, `LoadEndedRead`, `EndedProposal`, `DecidedProposal`, `EndedPersonTask`, `EndedTask`, `PartyInputs`, `EndedEscalation`; the start's generation switch | `Peer::Store` (5.1) |
| `Held`: `Assign`, `ReadList`, `Sources`, `CheckSource`, `SignOutSource`, `Grant`, `Relayed`, `Inbound`, `Acknowledge`, `AcknowledgeTurn`, `Cancel`, `Refuse`, `TurnBusy`, `SettledCall`, `CallAnswer`, `NotesWritten`, `NotesDeleted`, `ReadUsage`, `PauseWorker`, `AcknowledgePart`, `PartBusy` | `Peer::Host` (5.2) |
| `Now`: `PartBusy`, `RelayBusy`, `CallBusy`, `DropCall`, `Undelivered`, `EffectAnswer`, `SettledCallRefused`, `NoteBusy`, `Link(Asked)`, `Link(Relayed)`, and the reads' answers `TaskRead`, `SettingsAnswer`, `PersonAnswer`, `ProposalRead`, `NotesIndexed`, `NotesRecalled`, `NotesRefused` | `Peer::Host` (5.2), the reads' answers one (7.2) |
| `Now`: `Call`, `Ask`, and `Link`'s `DropPayload`, `TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload`, `PartPayload` | gone: decoded at the edge (the draft's section 8) |
| `Held`: `Acted`, `Read`, `Delivery`, `WatchEnded`, `PeopleReply`, `Result`, `PublicRead`, `NeedArrived`, `CarriedUnused`, `SourceSignedOut`, `ApplicationDelivery`, `ApplicationWatchEnded`, `Applied`; `Now`: `SignInRefused`, `WatchRefused`, `RunText` | `Peer::Party` (5.3), an application's document as a part (6.6) |
| `Held::Connector` | `Peer::Connector(Released)` (6.5) |
| `Held`: `Relay`, `TaskTerminalAcknowledged`, `TaskTurnKept`, `TaskPartKept`, `StopRun`, `PublicWait` | `Peer::Resume` (5.6) |
| `Now::Account` | `Peer::Account` (5.4) |
| `Request::ProjectMade`; `Held`: `Drained`, `Restarting`; `Now`: `BriefFailed`, `RestoreRefused`; the start's header | `Peer::Application` (5.5) |
| `Now`: `Activate`, `CompleteBrief`, `BriefCorePlanned`, `WorkspaceRequest`, `RunPreparationFailed`, `RunPrepared`, `DropAssignment` | gone: run preparation inside (the draft's section 10), connectors' parts planned and gathered (6.6) |
| the 67 core methods temper's root calls, and `ops`'s and the test root's, 73 in all | an input of a port, an output, or inside (4.5) |
| roots' access to `core.tasks`, `core.people`, `core.fleet` and the other children | gone (the draft's section 10) |
| the hub and `jig-domain-hub-link` in the engine's root | gone: the engine's own slots a worker (the draft's section 9) |

The connector capabilities' `Ask`, `Up` and `Released` now cross the engine's port. The remaining run preparation and connector operations outside it are listed in `jig/docs/design/domain/engine/vocabulary.md`, 2.5.

## 3. The counts' commands

Run from the repository root. The first, third and fourth print their counts at `422be3b57f` and at `HEAD`; the second reads the working tree. J1 records the counts it starts from in the execution file.

The edge, the roots' reach into the core, the debt's measures and the manifests (the draft's sections 2 and 16, counts 3 to 7, 9 and 10):

```bash
python3 -B - <<'PY'
import re, subprocess as s, tomllib

def git(*args):
    return s.check_output(['git', *args], text=True)

def read(rev, path):
    return git('show', rev + ':' + path)

def source(rev, prefix):
    for p in git('ls-tree', '-r', '--name-only', rev, '--', prefix).splitlines():
        if p.endswith('.rs') and not re.search(r'/(tests|fuzzy)/|/(tests[^/]*|fuzzy)\.rs$', p):
            yield p, read(rev, p)

roots = ['jig/examples/ops/domain/src', 'jig/testing/jig-test-domain/src',
         'temper/crates/temper-engine-domain/src']
children = 'accounts authority brief fleet notes people tasks views'.split()
child_names = {'jig-domain-' + n for n in children}
excluded = child_names | {'jig-domain-core', 'jig-domain-core-vocabulary', 'jig-domain-contract'}
counts = [
    ('jig/crates/jig-domain-core/src/lib.rs', 'struct Core',
     r'^\s+(?:pub(?:\(crate\))? )?[a-z_]+:'),
    ('jig/crates/jig-domain-tasks/src/boundary.rs', 'struct TaskRecord', r'^\s+pub [a-z_]+:'),
    ('jig/testing/jig-conformance/src/harness.rs', 'trait Application', r'^\s+fn '),
]
adapters = ['jig/tests/conformance/src/lib.rs', 'jig/tests/ops/src/conformance/mod.rs',
            'jig/tests/ops/src/joint/engine.rs', 'temper/tests/conformance/src/lib.rs']

for rev in ['422be3b57f', 'HEAD']:
    print(rev)
    methods = set()
    for root in roots:
        body = '\n'.join(t for _, t in source(rev, root))
        direct = set(re.findall(r'\bcore\.([a-z_]\w*)\s*\(', body))
        methods.update(direct)
        print(root, len(direct),
              len(re.findall(r'\bcore\.(?:' + '|'.join(children) + r')\b', body)),
              len(re.findall(r'\b(?:jig_domain_)?(?:tasks|fleet|people)::Event::\w+', body)),
              [len(re.findall(r'\b(?:jig_domain_)?tasks::Event::'+n+r'\b', body))
               for n in ('HoldFrom', 'WakeProcedure')])
    print('method union', len(methods))
    print('hub-link LOC', sum(t.count('\n') for _, t in source(rev, 'jig/crates/jig-domain-hub-link/src')))
    for path, decl, pattern in counts:
        body = re.search(r'^pub '+decl+r' \{\n(.*?)^\}', read(rev, path), re.M|re.S)[1]
        print(decl, len(re.findall(pattern, body, re.M)))
    for p in adapters:
        block = re.search(r'^impl Application for \w+ \{.*?^\}', read(rev,p), re.M|re.S)[0]
        print(p, len(block.splitlines()))
    core = git('grep','-l','-E',r'^jig-domain-core[. =]',rev,'--','*Cargo.toml').splitlines()
    dependents = git('grep','-l','-E',
                     r'^jig-domain-('+'|'.join(children)+r')[. =]',rev,'--','*Cargo.toml').splitlines()
    paths = [p.split(':',1)[1] for p in dependents]
    print('core manifests', len(core)-1, 'broad child manifests',
          sum(p != 'Cargo.toml' and not re.search(r'/jig-domain-core(?:-vocabulary)?/',p) for p in paths))
    production = 0
    for p in paths:
        if p == 'Cargo.toml' or re.search(r'/(tests|testing|benchmarks)/',p):
            continue
        doc = tomllib.loads(read(rev,p))
        if doc.get('package', {}).get('name') not in excluded and child_names & doc.get('dependencies', {}).keys():
            production += 1
    print('production child manifests', production)
PY
```

The core's and the connectors' cases `ops`'s root names:

```bash
rg -o '(core|jig_domain_core|connector)::(Ask|Held|Now|Request|Write|Event)::[A-Z][A-Za-z0-9_]+' \
  jig/examples/ops/domain/src -g '*.rs' -g '!tests*.rs' -g '!**/tests/**' --no-filename |
  sed 's/^jig_domain_core::/core::/' | sort -u
```

The roots' lines and temper's wiring files (count 8):

```bash
python3 -B - <<'PY'
import re, subprocess as s

roots = ['jig/examples/ops/domain', 'jig/testing/jig-test-domain',
         'temper/crates/temper-engine-domain']
wiring = re.compile(
    r'^(admission|assemble|boundary|deadline|decision|domain|journal_stop|lib|limits|link|'
    r'loads|push|restart|settling|start_wait|stopping|translate|wire|route)\.rs$|'
    r'^route/(asks|held|now|parts|opened|public|host_route|applications|document|see)\.rs$|'
    r'^route/(parts|document)/[^/]+\.rs$')
for rev in ['422be3b57f','HEAD']:
    for root in roots:
        prefix = root + '/src/'
        paths = s.check_output(['git','ls-tree','-r','--name-only',rev,'--',prefix],
                               text=True).splitlines()
        lines = {p[len(prefix):]: s.check_output(['git','show',rev+':'+p]).count(b'\n')
                 for p in paths if p.endswith('.rs') and
                 not re.search(r'/(tests|fuzzy)/|/(tests[^/]*|fuzzy)\.rs$',p)}
        print(rev, root, sum(lines.values()))
        if root == roots[-1]:
            chosen = {p:n for p,n in lines.items() if wiring.search(p)}
            print(len(chosen), sum(chosen.values()), chosen)
PY
```

The edge's enums, case by case (the draft's section 2):

```bash
python3 -B - <<'PY'
import re, subprocess as s

v = 'jig/crates/jig-domain-core-vocabulary/src/'
specs = {
    v+'vocabulary.rs': 'Event Request Ask Held Now LinkNow Write NamedAction',
    v+'effects/connector.rs': 'Ask Event',
    'jig/crates/jig-domain-tasks/src/boundary.rs': 'Event',
    'jig/crates/jig-domain-fleet/src/boundary.rs': 'Event',
    'jig/crates/jig-domain-people/src/boundary.rs': 'Event Ask',
}
def variants(rev, path, name):
    t = s.check_output(['git','show',rev+':'+path], text=True)
    body = re.search(r'^pub enum '+name+r' \{\n(.*?)^\}',t,re.M|re.S)[1]
    return set(re.findall(r'^    ([A-Z]\w*)\s*(?=[({,]|$)',body,re.M))

for path, names in specs.items():
    for name in names.split():
        old, new = [variants(r,path,name) for r in ('422be3b57f','HEAD')]
        print(path, name, len(old), len(new), 'added', sorted(new-old), 'gone', sorted(old-new))
PY
```
