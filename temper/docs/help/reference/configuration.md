# Configuration

Kind: reference

The configuration document's top level and application members, their bounds and what temper-box init --box writes.

Configuration is read at startup; changing a permitted member needs a restart.

Objects and lists of objects name their shapes below; init values show objects and list lengths, with scalar values written as JSON.
Address objects use the engine page's address shape; see temper help configuration-engine. Engine-only application members are absent for a worker alone.

## document

Shape: `document`. Paths: `document`. The top-level members of the configuration document.

- `engine`: engine, absent for a worker alone; optional; init writes object. The engine’s configuration; see temper help configuration-engine.
- `workers`: list of worker, one in standalone or worker alone; empty for engine alone; required; init writes list (1 entry). The workers’ configurations; see temper help configuration-engine.
- `application`: application; required; init writes object. The machine, listeners and application limits.

## application

Shape: `application`. Paths: `application`. The application’s machine parts, read before startup.

- `machine`: refused: moved: a forge's place is chosen when it's set up, and kept with the forge
- `demo`: refused: moved: the playground's scripted model is the box's own
- `welcome`: refused: moved: the welcome is a setting; change it with temper ctl settings
- `limits`: refused: moved: temper's limits are the setting temper-limits; ask temper, or use temper ctl settings apply
- `environment`: list of environment-entry, standalone and worker alone only; absent or empty inherits HOME then PATH; entries keep their order, adding HOME then PATH where omitted; optional; init leaves it out. The agents’ variables; only missing HOME and PATH come from the worker’s process.
- `workers`: refused: moved: a worker's workspaces are the data directory's; its git is found on PATH
- `clients`: application-clients, engine shapes only; required; init writes object. The clients’ listener, browser bundle and public origin.
- `sign_in`: sign-in or null, engine shapes only; required; init writes null. The people’s sign-in registration, or null for none.
- `push`: push or null, absent or null for none; engine shapes only; optional; init leaves it out. The deployment’s push key, contact and services.
- `landing`: refused: moved: a project's landing is settings, project-landing and project-gates; set them with temper ctl settings apply
- `webhooks`: webhooks or null, engine shapes only; required; init writes object. The Forgejo webhook listener, or null for none.
- `charter`: refused: the forge's charters live in the store; read them with temper ctl settings show forge-charter 1

## application.clients

Shape: `application-clients`. Paths: `application.clients`. The clients’ listener and browser presentation.

- `listener`: address, socket address; loopback when printed links or approvals are enabled; required; init writes {"socket":"127.0.0.1:8367"}. The clients’ listener and the address of printed sign-in links.
- `bundle`: text or null, absolute directory; at most 1,048,576 bytes; required; init writes null. The browser bundle directory, or null for the binary’s bundle.
- `public`: text or null, origin; at most 267 bytes; required; init writes null. The public browser origin, or null for the listener’s address.

## application.environment[]

Shape: `environment-entry`. Paths: `application.environment[]`. One variable given to the agents’ programs.

- `name`: text, at most 1,048,576 bytes; unique names; smith also refuses empty names, = and NUL; required; init leaves it out. The variable’s name.
- `value`: text, at most 1,048,576 bytes; smith also refuses NUL; required; init leaves it out. The variable’s value.

## application.push

Shape: `push`. Paths: `application.push`. The deployment’s push key and services.

- `key`: text, private data file, 0o600; PKCS#8 PEM P-256; at most 1,024 bytes; required; init leaves it out. The kept VAPID signing key; its public half is the clients’.
- `subject`: text, mailto: or https: URI; 1 to 256 printable ASCII bytes, no space, quote or backslash; required; init leaves it out. Where the push services may reach the operator.
- `services`: list of text or null, 1 to 8 origins; https, or http on loopback; null for https://fcm.googleapis.com, https://updates.push.services.mozilla.com and https://web.push.apple.com; required; init leaves it out. The allowed push services; hosts resolve at each send, HTTPS uses machine roots.

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

## The merge command

`merge_job` names this command, a path in the repository being merged. It settles the conflicts its project's rules know, with no model, before an agent is asked. temper takes it from the merge in progress: a command file left in conflict, or absent from the merge, is its failure.

- **What it is given:** its arguments, `merge` and the path of `request.json`, which holds `{"version":1,"head":"HEAD","base":"BASE"}`: the head merged into, and the commit merged in. Its environment is `PATH` (`/usr/local/bin:/usr/bin:/bin`), `LANG` (`C.UTF-8`), and `HOME` and `TEMPER_JOB`, both the job's directory, which is its working directory too. No credential reaches it. It has 600 seconds.
- **Where it works:** the repository is the directory of its own name under `TEMPER_JOB`, at the head with the base merged in and not committed; the conflicting files are marked and unmerged in the index (`git diff --name-only --diff-filter=U`).
- **What it must do:** edit the conflicting files it knows and `git add` each it settles, without committing, aborting the merge or writing another branch; then write `result.json` in `TEMPER_JOB` as compact JSON, at most 512 bytes including its newline. Fit `left`'s records first, in index order, then `settled`'s paths in that order: each whole or omitted, with every one after it in its list omitted too. Write members `settled`, `left`, then `more`, omitting empty `left` and, unless anything was omitted, `more`. Where anything was omitted, end with `"more":{"left":N,"settled":M,"read":"git status --short; git log --name-status --diff-filter=DR HEAD...MERGE_HEAD"}`, N and M the counts omitted. The same head and base must give the same tree and result.
- **Exit 0:** `{"settled":[PATH]}`, no file left unmerged: temper commits the tree as the merge, with both parents, and pushes it. A file left unmerged or still marked fails the job.
- **Exit 3:** `{"settled":[PATH],"left":[LEFT]}`, each LEFT `{"path":PATH,"kind":KIND}`, KIND `modified`, `added`, `deleted`, `renamed` or `unknown`, read from the index stages and each side's history, not a missing stage alone. A deletion or one-sided rename adds `by` (`head` or `base`), `from` or `to` for a rename, and `commit`, the first twelve hexadecimal digits of the commit in that side's history that deleted or renamed it. A rename on both sides has `by` (`both`) alone. The head is the branch merged into; the base is the commit merged in. It settles no file one side deleted or renamed: the agent judges it. Nothing is committed or pushed; an agent resolves in a merge made afresh, temper carrying this result into its words as it is, never parsing it.
- **Any other exit,** or a result missing or past its bound, is the command's failure. A job that fails for good holds the change, as a resolution that fails does.

## More slots

`workers[0].slots` bounds the runs a worker hosts at once, an agent process each. The host's agents follow it; the `agents` setting does not raise them. Each of these must be at least the slots, and a start names the one that's short:

- `hub.slots`, in the `limits` setting. Workers take their limits from the engine.
- `fleet.slots`, in the `limits` setting. An engine alone can't see a worker's slots: it uses them up to this, and turns away a hello listing more runs.
- `checkout.workspaces`, in the `temper-limits` setting, and `git_component.operations` there, at least the workspaces.
