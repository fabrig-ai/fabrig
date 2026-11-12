# Configuration

Kind: reference

The configuration document's top level and application members, their bounds and what temper-box init --box writes.

Every member is read at startup; a change to any member needs a restart.

Objects and lists of objects name their shapes below; init values show objects and list lengths, with scalar values written as JSON.
Address objects use the engine page's address shape; see temper help configuration-engine. Engine-only application members are absent for a worker alone.

## document

Shape: `document`. Paths: `document`. The top-level members of the configuration document.

- `engine`: engine, absent for a worker alone; optional; init writes object. The engine’s configuration; see temper help configuration-engine.
- `workers`: list of worker, one in standalone or worker alone; empty for engine alone; required; init writes list (1 entry). The workers’ configurations; see temper help configuration-engine.
- `application`: application; required; init writes object. The machine, listeners and application limits.

## application

Shape: `application`. Paths: `application`. The application’s machine parts, read before startup.

- `machine`: machine, engine shapes only; required; init writes object. The machine places offered when a forge is made.
- `demo`: demo or null, engine shapes only; required; init writes object. The playground’s scripted model, or null for none.
- `welcome`: boolean; optional; init writes true. Whether the document shows the welcome and offers the playground.
- `limits`: application-limits; required; init writes object. The application limits profile and named overrides.
- `workers`: list of application-worker, one entry per top-level worker, in the same order; required; init writes list (1 entry). The workers’ workspace directories and git programs.
- `clients`: application-clients, engine shapes only; required; init writes object. The clients’ listener, browser bundle and public origin.
- `sign_in`: sign-in or null, engine shapes only; required; init writes null. The people’s sign-in registration, or null for none.
- `landing`: landing, engine shapes only; optional; init leaves it out. The projects’ landing ways, checks and tiers.
- `webhooks`: webhooks or null, engine shapes only; required; init writes object. The Forgejo webhook listener, or null for none.
- `charter`: refused: the forge's charters live in the store; read them with temper ctl settings show forge-charter 1

## application.limits

Shape: `application-limits`. Paths: `application.limits`. The application’s limits profile and named overrides.

- `profile`: text, small or standard; required; init writes "standard". The limits profile used before applying overrides.
- `overrides`: named limits, see temper help limits; required; init writes {"checkout.message_bytes":10662,"checkout.workspaces":4,"forge.client.op_bytes":10660,"forge.client.repositories":3,"forge.judge_projects":8,"forge.output":82,"forge.repositories":3,"forge_component.body_bytes":21341,"git_component.operations":4,"root.journal.transcript_bytes":16803814,"root.loads.bytes":16804058,"root.loads.read_bytes":33607620,"root.loads.reply_bytes":16804058,"root.loads.transcript_bytes":16803810}. The dotted limit names whose values replace the profile’s.

## application.workers[]

Shape: `application-worker`. Paths: `application.workers[]`. One worker’s workspace and git program.

- `root`: text, data path; at most 1,048,576 bytes; required; init writes "workspaces". The private directory holding the worker’s checkouts.
- `git`: text or null, absolute executable path; at most 1,048,576 bytes; required; init writes "/usr/bin/git". The git program used when a repository names none, or null for none.

## application.clients

Shape: `application-clients`. Paths: `application.clients`. The clients’ listener and browser presentation.

- `listener`: address, socket address; loopback when printed links or approvals are enabled; required; init writes {"socket":"127.0.0.1:8367"}. The clients’ listener and the address of printed sign-in links.
- `bundle`: text or null, absolute directory; at most 1,048,576 bytes; required; init writes null. The browser bundle directory, or null for the binary’s bundle.
- `public`: text or null, origin; at most 267 bytes; required; init writes null. The public browser origin, or null for the listener’s address.

## application.webhooks

Shape: `webhooks`. Paths: `application.webhooks`. The Forgejo deliveries listener.

- `listener`: address, loopback socket, unique among listeners; required; init writes {"socket":"127.0.0.1:8371"}. The listener receiving Forgejo webhook hints.

## application.sign_in

Shape: `sign-in`. Paths: `application.sign_in`. The registration and people allowed to sign in.

- `google`: google; required; init leaves it out. The sign-in registration and its provider.
- `members`: list of sign-in-member, 1 to sign_in.members; required; init leaves it out. The people allowed to sign in through this registration.
- `forges`: list of forge-sign-in, each forge and client unique; optional; init leaves it out. The Forgejo forges signing their people in through the deployment.

## application.sign_in.google

Shape: `google`. Paths: `application.sign_in.google`. The sign-in provider’s public registration and secret path.

- `provider`: integer, application provider number; at most 65,535; required; init leaves it out. The identity provider number used for sign-in.
- `client`: text, 1 to 256 visible ASCII bytes; required; init leaves it out. The provider’s client id.
- `secret`: text, data path; at most 1,048,576 bytes; required; init leaves it out. The private file holding the client secret.
- `endpoints`: sign-in-endpoints or null; required; init leaves it out. The issuer’s endpoints, or null for Google’s own.

## application.sign_in.members[]

Shape: `sign-in-member`. Paths: `application.sign_in.members[]`. One person allowed to sign in.

- `address`: text, 3 to 254 visible ASCII bytes; one @; unique ignoring ASCII case; required; init leaves it out. The verified email address used as the identity’s subject.
- `login`: text, 1 to 40 bytes; letters, digits, dash, underscore or dot; unique ignoring ASCII case; required; init leaves it out. The login minted for the person.

## application.sign_in.forges[]

Shape: `forge-sign-in`. Paths: `application.sign_in.forges[]`. One Forgejo client of the deployment’s sign-in provider.

- `forge`: integer, 1 to 65,535; unique; required; init leaves it out. The forge’s number in the deployment.
- `client`: text, visible ASCII; unique; a configured provider client; required; init leaves it out. The client id used by the forge to sign people in.

## application.sign_in.google.endpoints

Shape: `sign-in-endpoints`. Paths: `application.sign_in.google.endpoints`. The sign-in issuer’s endpoints and transport.

- `issuer`: text, HTTPS URL, or HTTP on loopback; at most sign_in.url_bytes; required; init leaves it out. The issuer identifier checked in sign-in tokens.
- `authorization`: text, HTTPS URL, or HTTP on loopback; at most sign_in.url_bytes; required; init leaves it out. The endpoint where the browser authorizes sign-in.
- `token`: text, HTTPS URL, or HTTP on loopback; at most sign_in.url_bytes; required; init leaves it out. The endpoint exchanging the authorization code.
- `keys`: text, HTTPS URL, or HTTP on loopback; at most sign_in.url_bytes; required; init leaves it out. The endpoint supplying the issuer’s verification keys.
- `transport`: application-transport or text, plaintext on loopback, or TLS; required; init leaves it out. The transport used for the issuer’s endpoints.

## application.sign_in.google.endpoints.transport

Shape: `application-transport`. Paths: `application.sign_in.google.endpoints.transport`. A TLS transport object, or plaintext as text.

- `tls`: application-tls; required; init leaves it out. The issuer’s TLS server name and trust roots.

## application.sign_in.google.endpoints.transport.tls

Shape: `application-tls`. Paths: `application.sign_in.google.endpoints.transport.tls`. The sign-in issuer’s TLS identity and trust.

- `server_name`: text, at most 1,048,576 bytes; required; init leaves it out. The name checked during the TLS handshake.
- `trust`: text or null, configuration path; at most 1,048,576 bytes; required; init leaves it out. The trust roots file, or null for the machine’s roots.

## application.landing

Shape: `landing`. Paths: `application.landing`. The projects’ landing ways and checks.

- `projects`: list of landing-project, at most authority.projects; unique project numbers; required; init leaves it out. The projects’ landing configurations.
- `deployment`: refused: landing rules live in the store; read them with temper ctl settings show landing-rule

## application.landing.projects[]

Shape: `landing-project`. Paths: `application.landing.projects[]`. One project’s landing way, merge command and checks.

- `project`: integer, at most 4,294,967,295; unique; required; init leaves it out. The project number whose landing way is configured.
- `rules`: refused: landing rules live in the store; read them with temper ctl settings show landing-rule --project
- `by`: text, queue or turn; queue when absent; optional; init leaves it out. The project’s way of landing changes.
- `combine`: landing-combine, stock merge for main, squash for other; optional; init leaves it out. How each train combines its member.
- `merge_job`: text, relative repository path without parent segments; 1 to 4,096 bytes; optional; init leaves it out. The merge command’s path from the repository root; see The merge command, below.
- `close_job`: text, relative repository path without parent segments; 1 to 4,096 bytes; optional; init leaves it out. The close command’s path from the repository root; see temper help configuration’s close command.
- `gates`: list of repository-checks, at most forge.repositories; optional; init leaves it out. Each repository’s test gates and tiers.
- `check_timeout_ms`: integer, 1,000 to 86,400,000; none when absent; optional; init leaves it out. Delivery checks’ wall bound for change producers, repairs and resolutions, including goal landings; read each attempt, or the agent’s bound when absent.
- `gate_retries`: integer, 0 to 255; 2 when absent; optional; init leaves it out. How many runner faults a landing gate may retry.
- `gate_timeout_ms`: integer, 60,000 to 86,400,000; 3,600,000 when absent; optional; init leaves it out. How long a landing gate may run, in milliseconds.
- `tier_retries`: integer, 0 to 255; 2 when absent; optional; init leaves it out. How many times a failing tier may be retried.
- `verification_bound_ms`: integer, at most 18,446,744,073,709,551,615; 7,200,000 when absent; optional; init leaves it out. The verification time bound in milliseconds.

## application.landing.projects[].combine

Shape: `landing-combine`. Paths: `application.landing.projects[].combine`. How trains combine their members.

- `main`: text, merge or squash; merge when absent; optional; init leaves it out. Trains into the default branch.
- `other`: text, merge or squash; squash when absent; optional; init leaves it out. Trains into other branches.

## application.landing.projects[].gates[]

Shape: `repository-checks`. Paths: `application.landing.projects[].gates[]`. One repository’s test gates and after-landing tiers.

- `forge`: integer, at most 65,535; required; init leaves it out. The forge containing the repository.
- `repository`: integer, at most 4,294,967,295; required; init leaves it out. The repository number on the forge.
- `gates`: list of gate-check, at most forge.gates; optional; init leaves it out. The test gates used for landing by turn.
- `tiers`: list of tier-check, at most forge.tiers.tiers; optional; init leaves it out. The tiers checked after landing.

## application.landing.projects[].gates[].gates[]

Shape: `gate-check`. Paths: `application.landing.projects[].gates[].gates[]`. A test gate required before landing by turn.

- `name`: text, 1 to 32 bytes; ASCII letters, digits, dash or underscore; unique; required; init leaves it out. The gate’s name in reports.
- `workflow`: text, 1 to 255 bytes; no slash; required; init leaves it out. The workflow file dispatched for the gate.
- `ignored`: list of text, at most forge.ignored_paths; each at most forge.name_bytes; optional; init leaves it out. Whole paths the check ignores, with no empty, dot or parent segment.
- `blocking`: true or false; required; init leaves it out. Whether a failing gate blocks landing.
- `base`: true or false, false when absent; optional; init leaves it out. Whether the gate reads the round’s base, given as temper_base and included in its key.

## application.landing.projects[].gates[].tiers[]

Shape: `tier-check`. Paths: `application.landing.projects[].gates[].tiers[]`. A tier checked after a landing.

- `name`: text, 1 to 32 bytes; ASCII letters, digits, dash or underscore; unique; required; init leaves it out. The tier’s name in reports.
- `workflow`: text, 1 to 255 bytes; no slash; required; init leaves it out. The workflow file dispatched for the tier.
- `ignored`: list of text, at most forge.ignored_paths; each at most forge.name_bytes; optional; init leaves it out. Whole paths the check ignores, with no empty, dot or parent segment.
- `artifact`: text, 1 to 32 bytes; no slash; required; init leaves it out. The artifact listing the tier’s failing tests.
- `format`: text, junit or lines; required; init leaves it out. The format of the failing-tests artifact.
- `branch`: text, nonempty; goal, a branch name, or absent for the landing branch; optional; init leaves it out. The branch whose tier this follows.
- `span_ms`: integer, at most 18,446,744,073,709,551,615; 0 when absent; optional; init leaves it out. The interval in milliseconds between tier runs.
- `description`: text, 1 to 120 bytes of UTF-8; no control character; optional; init leaves it out. What the tier checks, shown on its card and by see settings gates.

## application.demo

Shape: `demo`. Paths: `application.demo`. The playground’s scripted model.

- `llm`: demo-llm; required; init writes object. The fake model listener, provider, key path and scripts.

## application.demo.llm

Shape: `demo-llm`. Paths: `application.demo.llm`. The scripted provider’s fake model.

- `listen`: address, loopback socket, unique among listeners; required; init writes {"socket":"127.0.0.1:8370"}. The fake model’s listener.
- `provider`: integer, configured LLM provider; at most 4,294,967,295; required; init writes 3. The provider whose account the playground makes.
- `key`: text, data path; at most 4,096 bytes; required; init writes "demo/llm.key". The private file holding the scripted provider’s key.
- `asked`: list of demo-script, at most 16 scripts; required; init writes list (3 entries). The scripts offered as playground prompts.
- `cued`: list of demo-script, at most 16 scripts; optional; init writes list (4 entries). The scripts matched by their cues without being offered as prompts.
- `otherwise`: list of demo-turn or text, nonempty; at most 16 turns; required; init writes list (1 entry). The fallback script for a prompt matching no cue.

## application.demo.llm.asked[]

Shape: `demo-script`. Paths: `application.demo.llm.asked[]` `application.demo.llm.cued[]`. One cue and its scripted turns.

- `cue`: text, 1 to 256 bytes; unique across scripts; required; init writes "Add CSV export to widget" | "Add a dark theme to widget" | "Fix widget's empty search state" | "## Task\n\nExport widget's table as CSV" | "Review this change at the named head" | "Write this up as a draft beside us." | "Add what happens to a running goal.". The prompt cue matching this script.
- `turns`: list of demo-turn or text, nonempty; at most 16 turns; required; init writes list (3 entries) | list (1 entry) | list (1 entry) | list (3 entries) | list (1 entry) | list (2 entries) | list (2 entries). The model turns played for this cue.

## application.demo.llm.asked[].turns[]

Shape: `demo-turn`. Paths: `application.demo.llm.asked[].turns[]` `application.demo.llm.cued[].turns[]` `application.demo.llm.otherwise[]`. One scripted model turn, or text for a plain answer.

- `lines`: list of demo-line, 1 to 16 lines; required; init writes list (2 entries) | list (1 entry) | list (2 entries) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry) | list (1 entry). The text or tool call making up this turn.
- `finish`: text, stop, tool_calls, length or content_filter; required; init writes "tool_calls" | "stop". The reason this model turn ends.
- `tokens`: integer, 1 to 18,446,744,073,709,551,615; required; init writes 270 | 2 | 30 | 68 | 70 | 7 | 116 | 47 | 24 | 242 | 19 | 317 | 17 | 51. The number of tokens charged for the turn.

## application.demo.llm.asked[].turns[].lines[]

Shape: `demo-line`. Paths: `application.demo.llm.asked[].turns[].lines[]` `application.demo.llm.cued[].turns[].lines[]` `application.demo.llm.otherwise[].lines[]`. One text answer or tool call.

- `text`: text, text alone, or call with arguments; optional; init writes "I’ll delegate this change in playground/widget. It will push a branch, open a pull request, check CI and land into main on Playground forge." | "CSV export is available from widget’s table. The change landed on Playground forge. This was a scripted change." | "I would give widget a dark theme that follows the system's preference, with its colours as variables in one place so both themes stay in step. I am the scripted model and play this from a script, so nothing is changed: connect a model from Accounts to have the work done." | "I would have widget say when a search finds nothing, name the words searched for and offer to clear them, in place of an empty table that looks broken. I am the scripted model and play this from a script, so nothing is changed: connect a model from Accounts to have the work done." | "I wrote an Away mode draft with its purpose, return flow and open questions." | "I added how a running goal continues and waits for your decisions." | "I am the scripted model, the playground’s: I answer only the suggested requests, from a script. To ask for other work, connect a model from Accounts: a ChatGPT plan, a Claude subscription or an API key.". The text returned by a plain answer.
- `call`: text, at most 64 bytes; call with arguments; optional; init writes "delegate" | "wait" | "read" | "edit" | "finish" | "note". The tool called by this scripted answer.
- `arguments`: JSON value, tool input schema; optional; init writes object. The tool call’s input, interpreted by the named tool.

## application.machine

Shape: `machine`. Paths: `application.machine`. The machine places offered when a forge is made.

- `owned`: machine-owned; optional; init writes object. The box’s owned Forgejo place, absent when none is offered.
- `fake`: machine-fake; optional; init writes object. The playground’s fake forge place, absent when none is offered.
- `github`: machine-github or null; optional; init writes null. GitHub’s place, null for the public service.

## application.machine.owned

Shape: `machine-owned`. Paths: `application.machine.owned`. The owned Forgejo and its runner on this machine.

- `sign_in`: text or null, a client of engine.clients.provider; at most 256 bytes; not with public; optional; init writes "forgejo". The client the box’s owned Forgejo signs its person in through, null for none.
- `remote`: text, at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash; required; init writes "http://127.0.0.1:8368". The base of the owned forge’s repository remotes.
- `forgejo`: text, cache path or fetched; at most 4,096 bytes; required; init writes "fetched". The Forgejo binary, fetched at the pinned version when requested.
- `runner`: machine-runner or null; required; init writes object. The runner to prepare, or null for none.
- `listen`: address, loopback socket, unique among listeners; required; init writes {"socket":"127.0.0.1:8368"}. The owned Forgejo listener.
- `person`: machine-account; required; init writes object. The person account made at setup.
- `mirror`: machine-mirror; optional; init leaves it out. Alternate HTTPS origins for fetching the pinned binaries.
- `public`: text, HTTPS origin; at most 4,096 bytes; optional; init leaves it out. The browser origin of the owned forge behind its proxy.

## application.machine.fake

Shape: `machine-fake`. Paths: `application.machine.fake`. The playground’s fake forge on this machine.

- `listen`: address, loopback socket, unique among listeners; required; init writes {"socket":"127.0.0.1:8369"}. The fake forge listener.
- `remote`: text, at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash; required; init writes "http://127.0.0.1:8369". The base of the fake forge’s repository remotes.
- `git`: text, data path; at most 4,096 bytes; required; init writes "demo/bin/git". The fake git program for the playground’s repositories.

## application.machine.github

Shape: `machine-github`. Paths: `application.machine.github`. A GitHub test peer’s addresses and transport.

- `api`: address, socket or host and port; required; init leaves it out. The GitHub API address.
- `transport`: machine-transport or text, plaintext on loopback, or TLS; required; init leaves it out. The transport to the GitHub API.
- `remote`: text, at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash; required; init leaves it out. The base of the GitHub repository remotes.
- `web`: text, at most 4,096 bytes; HTTP on loopback or HTTPS; no final slash; required; init leaves it out. The base of GitHub’s browser pages.
- `git`: text, data path; at most 4,096 bytes; optional; init leaves it out. The git program for repositories on the test peer.

## application.machine.owned.runner

Shape: `machine-runner`. Paths: `application.machine.owned.runner`. The owned Forgejo’s runner program and capacity.

- `binary`: text, cache path or fetched; at most 4,096 bytes; required; init writes "fetched". The runner binary, fetched at the pinned version when requested.
- `labels`: list of text, 1 to 8 labels, each at most 128 bytes; required; init writes ["temper-host:host"]. The labels offered to the forge’s jobs.
- `capacity`: integer, 1 to 64; required; init writes 1. The number of jobs the runner may execute at once.
- `path`: text, at most 4,096 bytes; nonempty without NUL; required; init writes "/usr/local/bin:/usr/bin:/bin". The PATH the runner gives its jobs.

## application.machine.owned.person

Shape: `machine-account`. Paths: `application.machine.owned.person`. The person account made on the owned forge.

- `login`: text, valid forge login; at most 4,096 bytes; required; init writes "person". The person’s forge login.
- `email`: text, valid forge email; at most 4,096 bytes; required; init writes "person@localhost". The person’s forge email address.

## application.machine.owned.mirror

Shape: `machine-mirror`. Paths: `application.machine.owned.mirror`. HTTPS origins used to fetch pinned binaries.

- `forgejo`: text, HTTPS origin; at most 4,096 bytes; required; init leaves it out. The origin serving the pinned Forgejo binary.
- `runner`: text, HTTPS origin; at most 4,096 bytes; required; init leaves it out. The origin serving the pinned runner binary.

## application.machine.github.transport

Shape: `machine-transport`. Paths: `application.machine.github.transport`. A TLS transport object, or plaintext as text.

- `tls`: machine-tls; required; init leaves it out. The API’s TLS server name and trust roots.

## application.machine.github.transport.tls

Shape: `machine-tls`. Paths: `application.machine.github.transport.tls`. TLS identity and trust for the GitHub test peer.

- `server_name`: text, at most 4,096 bytes; required; init leaves it out. The server name checked during the TLS handshake.
- `trust`: text or null, at most 1,048,576 bytes; required; init leaves it out. The trust roots, or null for the machine’s roots.

## The merge command

`merge_job` names this command, a path in the repository being merged. It settles the conflicts its project's rules know, with no model, before an agent is asked. temper takes it from the merge in progress: a command file left in conflict, or absent from the merge, is its failure.

- **What it is given:** its arguments, `merge` and the path of `request.json`, which holds `{"version":1,"head":"HEAD","base":"BASE"}`: the head merged into, and the commit merged in. Its environment is `PATH` (`/usr/local/bin:/usr/bin:/bin`), `LANG` (`C.UTF-8`), and `HOME` and `TEMPER_JOB`, both the job's directory, which is its working directory too. No credential reaches it. It has 600 seconds.
- **Where it works:** the repository is the directory of its own name under `TEMPER_JOB`, at the head with the base merged in and not committed; the conflicting files are marked and unmerged in the index (`git diff --name-only --diff-filter=U`).
- **What it must do:** edit the conflicting files it knows and `git add` each it settles, without committing, aborting the merge or writing another branch; then write `result.json` in `TEMPER_JOB`, at most 512 bytes. Where the paths do not fit, it lists those that do, in index order, and adds `"more":N`, the count left out. The same head and base must give the same tree.
- **Exit 0:** `{"settled":[PATH]}`, no file left unmerged: temper commits the tree as the merge, with both parents, and pushes it. A file left unmerged or still marked fails the job.
- **Exit 3:** `{"settled":[PATH],"left":[PATH]}`, the conflicts it could not settle: nothing is committed or pushed, and an agent resolves them in a merge made afresh, this result in its words.
- **Any other exit,** or a result missing or past its bound, is the command's failure. A job that fails for good holds the change, as a resolution that fails does.

## More slots

`workers[0].slots` bounds the runs a worker hosts at once, an agent process each. The host's agents follow it, and nothing in `agent.json` raises them. Each of these must be at least the slots, and a start names the one that's short:

- `hub.slots`, in `workers[0].limits.overrides`.
- `fleet.slots`, in `engine.limits.overrides`. An engine alone can't see a worker's slots: it uses them up to this, and turns away a hello listing more runs.
- `checkout.workspaces`, in `application.limits.overrides`, and `git_component.operations` there, at least the workspaces.
