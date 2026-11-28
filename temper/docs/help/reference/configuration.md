# Configuration

Kind: reference

The configuration file's keys, their shapes, bounds and defaults.

Every key is read at a start; changing any key needs a restart.

S: `temper-box`; E: `temper-engine`; W: `temper-worker`. Help lists public keys alone.

## Keys

| Key | Shapes | Set | Form and bound | Default | What it says |
|---|---|---|---|---|---|
| `format` | S E W | public | a whole number from 1 to the newest format | 1 | the format the file is written in (written only above 1) |
| `listen` | S E | public | a socket address on loopback, such as "127.0.0.1:7100" | 127.0.0.1:7100 | where the client connects |
| `public` | S E | public | an origin within 267 bytes; the start checks its form | the listener's origin | where people reach it |
| `webhooks` | S E | public | a socket address on loopback, such as "127.0.0.1:7100" | listen's next port (0 stays 0) | the forge's webhooks' listener |
| `owner` | S E | public | an address of 3 to 254 visible ASCII bytes with one @ | the user's login at localhost | the first owner, read at the first start alone |
| `workers` | E | public | a socket address on loopback, such as "127.0.0.1:7100" | required | where workers connect, on loopback |
| `engine` | W | public | a socket address on loopback, such as "127.0.0.1:7100" | required | where its engine listens for workers |
| `name` | W | public | 1 to 32 bytes | the host's name | the name its engine knows it by, if not the host's |
| `[sign_in]` | S E | public | a table | no Google sign-in | Google sign-in and its members |
| `[sign_in.google]` | S E | public | a table | no Google sign-in | Google's sign-in |
| `sign_in.google.client` | S E | public | 1 to 256 visible ASCII bytes | required with Google | Google's client |
| `sign_in.google.secret` | S E | public | a nonempty data path, no NUL or relative .. component, at most 4096 bytes | required with Google | in the data directory, mode 0600 |
| `[[sign_in.members]]` | S E | public | an array of 1 to 64 member tables | required with Google | one for each member |
| `sign_in.members[].address` | S E | public | an address of 3 to 254 visible ASCII bytes with one @ | required | the member's address |
| `sign_in.members[].login` | S E | public | a login of 1 to 40 ASCII letters, digits, -, _ or ., starting and ending alphanumeric | required | the member's login |
| `[[sign_in.forges]]` | S E | public | an array of tables | none | each Forgejo whose sign-in goes through temper |
| `sign_in.forges[].forge` | S E | public | a forge number of 1 to 65535 | required | the forge's number |
| `sign_in.forges[].client` | S E | public | 1 to 255 visible ASCII bytes | required | the provider's client |
| `[openid]` | S E | public | a table | the box's provider, where provider.key exists; none on an engine | temper as an OpenID provider for Forgejo's sign-in |
| `openid.key` | S E | public | a nonempty data path, no NUL or relative .. component, at most 4096 bytes | provider.key | the provider's signing key in the data directory |
| `openid.addresses` | S E | public | a list containing only 1 (printed link) or 3 (Google) | [3] with Google, else [1] | the providers whose subjects are given as email |
| `[[openid.clients]]` | S E | public | an array of tables | none | each client of the provider |
| `openid.clients[].id` | S E | public | 1 to 255 visible ASCII bytes | required | the client's id |
| `openid.clients[].secret` | S E | public | a nonempty data path, no NUL or relative .. component, at most 4096 bytes | public client | the client's secret's file in the data directory |
| `openid.clients[].redirect` | S E | public | a redirect URI within 2048 bytes; the start checks its form | required | the client's redirect URI |
| `[push]` | S E | public | a table | no push | the deployment's push |
| `push.key` | S E | public | a nonempty data path, no NUL or relative .. component, at most 4096 bytes | required | the push key's file in the data directory |
| `push.subject` | S E | public | a mailto: or https: subject, at most 256 visible ASCII bytes | required | where a push service may reach the operator |
| `[programs]` | S W | public | a table | found or fetched | programs by their paths |
| `programs.git` | S W | public | an absolute path, no NUL, at most 1048576 bytes | found on PATH | the git executable |
| `programs.claude` | S W | public | an absolute path, no NUL, at most 1048576 bytes | found on PATH or fetched | the Claude executable |
| `programs.forgejo` | S | public | an absolute path, no NUL, at most 1048576 bytes | fetched at set-up | the Forgejo executable |
| `programs.runner` | S | public | an absolute path, no NUL, at most 1048576 bytes | fetched at set-up | the Forgejo runner executable |
| `[environment]` | S W | public | a table of strings; names nonempty without = or NUL, values without NUL | PATH and HOME appended where absent | one line for each variable |
| `[internal]` | S E | internal | a table | the binary's own | internal build inputs |
| `internal.bundle` | S E | internal | an absolute path, no NUL, at most 1048576 bytes | the binary's own bundle | a client bundle of another build |
| `[test]` | S E | test | a table | the real services | test services |
| `[test.github]` | S E | test | a table | github.com's | a fake GitHub |
| `test.github.api` | S E | test | a socket address or HOST:PORT | required | the fake GitHub's API |
| `test.github.transport` | S E | test | "plaintext" on loopback, or { server_name = "...", trust = "..." } | required | the fake GitHub's transport |
| `test.github.transport.server_name` | S E | test | 1 to 4096 visible ASCII bytes | required with TLS | the TLS server name |
| `test.github.transport.trust` | S E | test | a nonempty configuration path, no NUL or relative .. component, at most 4096 bytes | the machine's roots | the TLS trust file in the configuration directory |
| `test.github.remote` | S E | test | an HTTP(S) URL base with no final slash, at most 4096 bytes | required | the fake's remote URL base |
| `test.github.web` | S E | test | an HTTP(S) URL base with no final slash, at most 4096 bytes | required | the fake's web URL base |
| `test.github.git` | S E | test | a nonempty data path, no NUL or relative .. component, at most 4096 bytes | the worker's git | the fake's git in the data directory |
| `[test.google]` | S E | test | a table | Google's | a fake issuer for Google sign-in |
| `test.google.issuer` | S E | test | an HTTP(S) URL within 2048 bytes; HTTP only to loopback | required | the fake issuer's URL |
| `test.google.authorization` | S E | test | an HTTP(S) URL within 2048 bytes; HTTP only to loopback | required | the fake's authorization URL |
| `test.google.token` | S E | test | an HTTP(S) URL within 2048 bytes; HTTP only to loopback | required | the fake's token URL |
| `test.google.keys` | S E | test | an HTTP(S) URL within 2048 bytes; HTTP only to loopback | required | the fake's keys URL |
| `test.google.transport` | S E | test | "plaintext" on loopback, or { server_name = "...", trust = "..." } | required | the fake issuer's transport |
| `test.google.transport.server_name` | S E | test | 1 to 4096 visible ASCII bytes | required with TLS | the TLS server name |
| `test.google.transport.trust` | S E | test | a nonempty configuration path, no NUL or relative .. component, at most 4096 bytes | the machine's roots | the TLS trust file in the configuration directory |
| `[test.push]` | S E | test | a table | the real push services | fake push services |
| `test.push.services` | S E | test | 1 to 8 origins within 1024 bytes; HTTP only to loopback | required | the fake push services' origins |

## Where each member of temper.json went

- `application.clients.listener` → `listen`.
- `application.clients.public` → `public`.
- `application.webhooks` → `webhooks`.
- `engine.owners` → `owner`.
- `engine.hosts.listener` → `workers`.
- `workers[0].engine` → `engine`.
- `workers[0].name` → `name`.
- `application.sign_in` → `[sign_in]`.
- `application.sign_in.google` → `[sign_in.google]`.
- `application.sign_in.google.client` → `sign_in.google.client`.
- `application.sign_in.google.secret` → `sign_in.google.secret`.
- `application.sign_in.members` → `[[sign_in.members]]`.
- `application.sign_in.members[].address` → `sign_in.members[].address`.
- `application.sign_in.members[].login` → `sign_in.members[].login`.
- `application.sign_in.forges` → `[[sign_in.forges]]`.
- `application.sign_in.forges[].forge` → `sign_in.forges[].forge`.
- `application.sign_in.forges[].client` → `sign_in.forges[].client`.
- `engine.clients.provider` → `[openid]`.
- `engine.clients.provider.key` → `openid.key`.
- `engine.clients.provider.addresses` → `openid.addresses`.
- `engine.clients.provider.clients` → `[[openid.clients]]`.
- `engine.clients.provider.clients[].id` → `openid.clients[].id`.
- `engine.clients.provider.clients[].secret` → `openid.clients[].secret`.
- `engine.clients.provider.clients[].redirect_uri` → `openid.clients[].redirect`.
- `application.push` → `[push]`.
- `application.push.key` → `push.key`.
- `application.push.subject` → `push.subject`.
- `application.environment` → `[environment]`.
- `application.clients.bundle` → `internal.bundle`.
- `application.test.github` → `[test.github]`.
- `application.test.github.api` → `test.github.api`.
- `application.test.github.transport` → `test.github.transport`.
- `application.test.github.transport.tls.server_name` → `test.github.transport.server_name`.
- `application.test.github.transport.tls.trust` → `test.github.transport.trust`.
- `application.test.github.remote` → `test.github.remote`.
- `application.test.github.web` → `test.github.web`.
- `application.test.github.git` → `test.github.git`.
- `application.sign_in.google.endpoints` → `[test.google]`.
- `application.sign_in.google.endpoints.issuer` → `test.google.issuer`.
- `application.sign_in.google.endpoints.authorization` → `test.google.authorization`.
- `application.sign_in.google.endpoints.token` → `test.google.token`.
- `application.sign_in.google.endpoints.keys` → `test.google.keys`.
- `application.sign_in.google.endpoints.transport` → `test.google.transport`.
- `application.sign_in.google.endpoints.transport.tls.server_name` → `test.google.transport.server_name`.
- `application.sign_in.google.endpoints.transport.tls.trust` → `test.google.transport.trust`.
- `application.push.services` → `test.push.services`.
- `engine.llm`: moved: LLM providers are settings; see them with temper ctl settings show provider.
- `application.demo`: moved: the playground's scripted model is the box's own.
- `application.landing`: moved: a project's landing is settings, project-landing and project-gates; set them with temper ctl settings apply.
- `engine.limits`: moved: jig's limits are the setting limits; ask temper, or use temper ctl settings apply.
- `engine.memory_bytes`: moved: the root's budget is transcript_memory, in the setting temper-limits.
- `application.limits`: moved: temper's limits are the setting temper-limits; ask temper, or use temper ctl settings apply.
- `workers[].memory_bytes`: moved: nothing read a worker's memory_bytes; remove it.
- `application.machine`: moved: a forge's place is chosen when it's set up, and kept with the forge.
- `application.welcome`: moved: the welcome is a setting; change it with temper ctl settings.
- `workers[].graces`: moved: temper fixes a worker's graces (temper help server).
- `workers[].programs`: moved: temper fixes the login checks' interval.
- `application.workers`: moved: a worker's workspaces are the data directory's; its git is found on PATH.
- `engine.deployment`: moved: temper keeps the store in the data directory's state.
- `engine.providers`: moved: temper fixes its identity providers: 1 link, 2 deployment, 3 Google's.
- `engine.connectors`: moved: temper fixes its connector, the forge's, 1.
- `engine.acts`: moved: temper fixes the forge connector's acts.
- `engine.projects`: moved: projects are made at runtime; make one from the client or temper ctl.
- `engine.clients.heartbeat_ms`: moved: temper fixes the clients' heartbeat, printed link and approval.
- `engine.clients.link`: moved: temper fixes the clients' heartbeat, printed link and approval.
- `engine.clients.approval`: moved: temper fixes the clients' heartbeat, printed link and approval.
- `workers[].slots`: a worker's places are its setting's; temper ctl worker places NAME N.
- `workers[].secret`: a worker's secret is secrets/worker.secret in the data directory.
- `workers[].agent`: temper writes the agent's configuration from the engine's figures.

## Formats

Format 1 is the newest temper-box reads; a file without format is format 1.

Format 1 is the newest temper-engine reads; a file without format is format 1.

Format 1 is the newest temper-worker reads; a file without format is format 1.

## The merge command

The command is the repository's `.temper/hooks/merge`, or the project's `merge_job` in a repository without `.temper/hooks/`. It settles the conflicts its project's rules know, with no model, before an agent is asked. temper takes it from the merge in progress: a command file left in conflict, or absent from the merge, is its failure.

- **What it is given:** its arguments, `merge` and the path of `request.json`, which holds `{"version":1,"head":"HEAD","base":"BASE"}`: the head merged into, and the commit merged in. Its environment is `PATH` (`/usr/local/bin:/usr/bin:/bin`), `LANG` (`C.UTF-8`), and `HOME` and `TEMPER_JOB`, both the job's directory, which is its working directory too. No credential reaches it. It has 600 seconds.
- **Where it works:** the repository is the directory of its own name under `TEMPER_JOB`, at the head with the base merged in and not committed; the conflicting files are marked and unmerged in the index (`git diff --name-only --diff-filter=U`).
- **What it must do:** edit the conflicting files it knows and `git add` each it settles, without committing, aborting the merge or writing another branch; then write `result.json` in `TEMPER_JOB` as compact JSON, at most 512 bytes including its newline. Fit `left`'s records first, in index order, then `settled`'s paths in that order: each whole or omitted, with every one after it in its list omitted too. Write members `settled`, `left`, then `more`, omitting empty `left` and, unless anything was omitted, `more`. Where anything was omitted, end with `"more":{"left":N,"settled":M,"read":"git status --short; git log --name-status --diff-filter=DR HEAD...MERGE_HEAD"}`, N and M the counts omitted. The same head and base must give the same tree and result.
- **Exit 0:** `{"settled":[PATH]}`, no file left unmerged: temper commits the tree as the merge, with both parents, and pushes it. A file left unmerged or still marked fails the job.
- **Exit 3:** `{"settled":[PATH],"left":[LEFT]}`, each LEFT `{"path":PATH,"kind":KIND}`, KIND `modified`, `added`, `deleted`, `renamed` or `unknown`, read from the index stages and each side's history, not a missing stage alone. A deletion or one-sided rename adds `by` (`head` or `base`), `from` or `to` for a rename, and `commit`, the first twelve hexadecimal digits of the commit in that side's history that deleted or renamed it. A rename on both sides has `by` (`both`) alone. The head is the branch merged into; the base is the commit merged in. It settles no file one side deleted or renamed: the agent judges it. Nothing is committed or pushed; an agent resolves in a merge made afresh, temper carrying this result into its words as it is, never parsing it.
- **Any other exit,** or a result missing or past its bound, is the command's failure. A job that fails for good holds the change, as a resolution that fails does.
