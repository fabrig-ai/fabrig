# Forges through ctl

This document holds forges through ctl, split from [`areas.md`](areas.md).

## 1. Forges
6. **Forges** (`../forge/repositories.md`, 1 and `../forge/deployment.md` and `../forge/repositories.md`, 4): each forge the store
   keeps as temper's `Forges` page shows it, its set ups and connects,
   which make a forge when no FORGE is named (`../../protocol/forge/engine.md`,
   2), its removal, the playground's start and reset, and the
   adoption form's picker. Each act is one of the FORGE requests (`../engine.md`
   ), made through `jig-client` as the forge family's bytes,
   as `repository adopt` is.

   | Tool | What it does | temper's | K |
   |---|---|---|---|
   | `forge list` | each forge the store keeps by its number, kind and `web` address: its connection, "connected as LOGIN (ID)", connecting and what it waits for, stopped and why, not set up, setting up and at which step, set up failed and why, or not connected and why; how its last connect ended, a GitHub App's lacking permissions and events each named; its mode, and "admin's token kept" for managed and owned; on GitHub, the App's slug and its installations, each with its id, its account's login, all or selected repositories, suspended, or not accepted (a stray), "not accepted · kept", with its act, `forge accept FORGE INSTALLATION`; a job's log on a host it does not read, with how to add it; its load; its acts. Before the forges, a line for each table the forges' document carries full past its report bound, `the forge's outbox full for AGE: WORDS` or `the forge's read admission full for AGE: WORDS`, as the engine's log says it (`../../protocol/service-process.md`, 1.3); JSON carries each as `full`, its `table`, `since` and `words` | the forge's deployment part, read and watched (`../forge/served.md`) | |
   | `forge connect [FORGE] github` | starts GitHub's manifest flow, FORGE a kept GitHub forge, or, with none, a new one, the App made under the person's own account and installed on any account GitHub offers them: prints the link to open in a browser, the manifest's page on the clients' listener, and the call that follows it, `forge list --until settled`; exits 0 once the request is durable | `Connect { forge, how: GitHubManifest { back: none } }` | yes |
   | `forge connect [FORGE] github-app APP FILE [--read-only]` | connects an App of the person's own, FORGE a kept GitHub forge, or, with none, a new one: APP its id, FILE its private key, PEM as GitHub gives it; no webhook secret; temper points the App's hook at `https://SLUG.invalid/` and says so in its output, unless `--read-only` leaves it as read; then `forge list --until settled` | `Connect { forge, how: GitHubApp { id, read_only?, key } }` | yes |
   | `forge connect [FORGE] forgejo FILE [--api HOST:PORT --git URL [--tls NAME] [--web URL] [--git-program FILE]]` | connects a Forgejo through its bot's tokens, FILE a tokens file (below) naming `api`, `git` and, where the person made hooks to temper's listener, `webhook_secret`: FORGE a kept connected Forgejo, not connected or stopped, or, with none, a new one, connected, at `--api` and `--git`, both required without FORGE and refused with it, as are the three that follow, as `forgejo-admin`'s are; the setup host checks that both tokens answer as one user, not a site admin, and on a kept forge as its kept user, then keeps them and connects as that user; prints `connecting` and the call that follows, `forge list --until connected:FORGE`, and exits 0 once the request is durable; refused, exit 2, in `ForgeRefusal`'s words (`../forge/records.md`) | `Connect { forge, how: ForgejoTokens }`, `forge` the kept one or a `connected` `ManagedForge`, the tokens and the secret carried as `ApiToken`, `GitToken` and `WebhookSecret` | yes |
   | `forge connect [FORGE] forgejo-admin FILE [--first ORG/REPOSITORY [--project PROJECT] [--label LABEL]] [--api HOST:PORT --git URL [--tls NAME] [--web URL] [--git-program FILE]]` | connects a Forgejo as its site admin, FILE the admin's token: FORGE a kept managed Forgejo, or, with none, a new one, managed, at `--api`, its API's host and port, and `--git`, its remotes' base, both required without FORGE and refused with it, as are the three that follow, with `--tls`, its TLS server name, plaintext without it, `--web`, its pages, and `--git-program`, the git its workers run, theirs without it (`../forge/records.md`, `ManagedForge`); Reconcile makes or finds its marked bot, never TakeOver; with `--first`, makes ORG/REPOSITORY with the starter workflow, unadopted, PROJECT a number or name as for `forge setup`, default a new project named REPOSITORY, held in the call's own run and never sent in `first`, runner LABEL default `docker`; the token is kept only after Reconcile and Create are Done, and every end erases the carried value; without `--first`, prints `connecting` and `forge list --until connected:FORGE`; with `--first`, prints `connecting`, waits for Made and Connected, then shares `forge setup`'s MakeProject and adoption continuation, as PROJECT's home unless it has one, with key `first-repository-F`; prints the project number and adoption (JSON: `project` and `outcome`) and exits 0; a stop or failed Create exits 1 naming the reason or step; an ORG equal to the bot's login, case-folded, is refused before any write and exits 2, `first_is_bot`, "the organization is the bot's login: name another organization" | `Connect { forge, how: ForgejoAdmin { first? } }`, `forge` the kept one or a `ManagedForge`, token carried as AdminToken | yes |
   | `forge install FORGE` | prints a fresh link to install temper's App on another account, GitHub alone; the installation shows on `forge list` once the person's browser comes back | `Install { forge, back: none }` | |
   | `forge accept FORGE INSTALLATION` | accepts INSTALLATION, an installation of temper's App that exists already on GitHub and that `forge list` shows not accepted, with no browser: read on GitHub as the App's, then kept as an installation accepted through the setup route is (`../../protocol/setup/github.md`, 4); prints `accepted: installation I on forge F` and the call that follows, `forge repositories FORGE --installation INSTALLATION`, exit 0, and the same when it is accepted already; refused, exit 2, as `forge install` is, or, its run stopped, in the forge's words for its reason (`../forge/records.md`): "GitHub has no installation I of this App", "GitHub refused the App's read of installation I", "GitHub did not answer: try again", "temper keeps at most N installations" | `Accept { forge, installation }` | |
   | `forge replace FORGE FILE` | replaces what a person pasted, on a forge connected or stopped: on GitHub, FILE the App's new key, PEM as GitHub gives it, which must be a key of the same App; on a connected Forgejo, a tokens file naming `api`, `git` or both, and `webhook_secret` when the hooks' secret changes, which must answer as the forge's user; each new value is kept over the old, and the forge connects with them; prints `connecting` and `forge list --until connected:FORGE`, and exits 0 once the request is durable; refused, exit 2, in `ForgeRefusal`'s words, `not connected` on a forge not connected and `busy` while it connects | `Replace { forge }`, the key or the tokens carried | yes |
   | `forge forget FORGE` | erases a managed forge's admin's token; the forge is connected, its bot and tokens kept | `Forget { forge }` | |
   | `forge setup [FORGE] [--org ORG] [--repository REPOSITORY] [--project PROJECT] [--playground]` | sets up an owned Forgejo here, under the box, with its first repository, FORGE a kept owned Forgejo, or, with none, a new one, made as the request is admitted (`../../protocol/forge/engine.md`, 2): the box fetches its binaries, prepares it and starts Forgejo and its runner, then temper takes its handover, makes `ORG/REPOSITORY` (default `work/hello`) with the starter workflow, or with `--playground` makes `playground/widget` as the playground's copy, its files beside the starter workflow, taking no `--org` or `--repository` (a usage error, exit 1, with either), and connects (`../../protocol/setup/owned.md`, 5, and `../../protocol/setup/owned-repositories.md`, 1); prints `setting up`, waits as `forge list --until made:FORGE` does, then adopts `ORG/REPOSITORY` into PROJECT, a project's number or name, a name no project holds made a new project first by jig's `MakeProject`, and with no `--project` a new project named REPOSITORY (`../../draft/archive/first-run.md`, 2.4); PROJECT is the call's own, held in its run and never sent with `SetUp`, which names no project (`../../protocol/setup/owned-repositories.md`, 1), so that a call restored from its kept `SetUp` takes it from its own arguments; it adopts as `repository adopt --project PROJECT --forge FORGE --owner ORG --name REPOSITORY --landing main` would, as PROJECT's home unless PROJECT already has one, under the same sign-in and with the key `first-repository-F`, so that the call run again adopts nothing twice; prints the adoption, without `../forge/family.md`, 6's line on protection it could not read, and exits 0. That line is true there, temper's bot being a writer as it is anywhere (`../../domain/forge/adoption.md`), but it asks nothing a person can do on a repository temper has just made; `--json` keeps `protection`, and `repository adopt` keeps the line. Refused, exit 2: "not offered" (an owned forge already exists when FORGE is absent, or the forge is not an owned one whose setup says `"start": "asked"`, or it is set up, setting up or connecting), "busy" (the box is setting up another forge), "invalid" (ORG or REPOSITORY not a name Forgejo takes, or ORG, REPOSITORY or PROJECT's name empty or too long, PROJECT's checked by the call before it sends), or, without the box, "not under the box", saying to start temper with `temper box` (`../forge/deployment.md`; `../../protocol/setup/box.md`, 4). A not-offered refusal in text and JSON points to `forge list`, `project make NAME --key KEY`, then `repository make --project PROJECT --forge FORGE ORG/NAME --until adopted --key KEY`; the single-owned-forge rule remains. Set up failed, or its first repository failed, exit 1, naming the step. With `--playground`, ORG and REPOSITORY above are `playground` and `widget`, and the default project a new one named `widget` | `SetUp { forge, org, repository, playground }`; then `MakeProject` when PROJECT names no project, and `Adopt` | |
   | `playground start` | starts the playground (`../../protocol/setup/playground.md`): the engine makes its fake forge, the box starts its fakes, and the engine adds the scripted model's account and adopts its repository into its project, in the caller's name; waits for the answer, then prints `running` and exits 0, the same when it already runs. Refused, exit 2: "not offered" (a real forge kept). Start failed, exit 1, naming why: the fakes did not start (the box's why), the account refused, the adoption refused, or the forge not connected in time; run again, it makes what is not done | `StartPlayground` | |
   | `playground end` | resets the playground (`../../protocol/setup/playground.md`), as the banner's "set up temper" does (`../forge/family.md`, 7): its project deleted, its fake forge removed and the scripted model's account removed, each a kept act the engine resumes at its next start if one is cut; waits for the answer, then prints `reset` and exits 0, once the playground is gone. Refused, exit 2: "not offered" (no playground) | `EndPlayground` | |
   | `forge remove FORGE` | removes forge FORGE (`../../domain/forge/adoption-connection.md`, 1): ended at once, then, by kept acts, its live work cancelled, its box group stopped and its directory removed, its records erased; on the forge itself only what temper ran is undone, a managed Forgejo's bot and hooks and a GitHub App staying the person's. Prints `removed` and exits 0 once the forge is ended. Refused, exit 2, in `ForgeRefusal`'s words: not the deployment's owner, no such forge, or not taken by this forge (the playground's, which `playground end` removes) | `RemoveForge { forge }` | |
   | `change open --project PROJECT [--repository OWNER/NAME] --branch BRANCH [--at COMMIT] [--into BRANCH] --title TEXT [--body TEXT] (--budget AMOUNT \| --as-allowed)` | opens a change from BRANCH, a branch the person pushed, as their own change task, which temper lands as any change (`../../domain/forge/change/opened.md`): in OWNER/NAME, the project's only repository when none is named; at COMMIT, the branch's head when the change takes it where none is named; into BRANCH, the repository's landing branch, the one its adoption names (`../../domain/forge/adoption.md`, 12), where none is named; TEXT its pull request's title, and its body's words after the opening line; AMOUNT what its reviews, resolutions and repairs may spend, or what the person's role allows. Prints `task N`, exit 0, which `change show` and `landing history --change` then read. Usage, exit 1, nothing sent: the controls missing, named together, `change open needs: --branch BRANCH; --title TEXT; --budget AMOUNT or --as-allowed`, only those absent; `--budget` and `--as-allowed` both given; `--repository` not `OWNER/NAME`; `--at` not 40 or 64 hexadecimal digits, sent lowercase; a title past 256 bytes, a body past 16,384, a branch or `--into` past 1,024, each naming its bound. Refused, exit 2, nothing made, one line `temper ctl: change open: WHY`: `project P has several repositories, R1, R2: name one with --repository`; `project P has no repository a change lands into`; `R is not a repository of project P`; `BRANCH is under R's prefix PREFIX: a change lands into one of its landing branches`; `BRANCH is past the forge's N-byte name bound`; and the core's refusals, the builder's among them, in jig's words as `goal start` prints them (area 2). A replay under its key answers the first. A change that fails later was opened, and `change show TASK` says why | the forge family's `OpenChange` (`../forge/records.md`) | yes |
   | `change show TASK [--project PROJECT]` | the panel of change task TASK (`../forge/served.md`, 2; `../forge/records.md`), read as the change's page watches it, in the scope of PROJECT, the task's project when none is named: its repository by forge, owner and name; its branch and landing branch; its head; for a change opened from a pushed branch, its source: the branch, the commit named or `its head`, and the head taking kept; its pull request's number and address on the forge; its step in `../forge/README.md`, 2.1's words, with its place in its queue, a repair's reason or, landed, its merge commit, whole; its hold; the task its step runs; CI at its head; its gates, each with its kind, whether it blocks and its verdict; its repairs, resolutions and updates, each used of its limit; its questions (`../forge/served.md`, 2): its route, the questions taken used of its bound, and one line per open question, its asker's task and the question's number, where it stands (`taken`, `asked of requester`, `answered, held`) and the first line of its words; then the call that follows it, `change show TASK --until landed` while it is live and has not landed, `change show TASK --until cleared` while it is verifying or red. An ended step points to `task show TASK` for its report; a successful no-change report adds `completed, no change`. A landed or ended change keeps its last panel. `task show TASK` of a change task, live or ended, prints `change show TASK` among the calls that follow it. A task that is no change into a repository of PROJECT, or a change the forge has not registered yet, prints `no change panel for task N`, exit 2 | the forge's `change` subject, read once (`../forge/served.md`, 2) | |
   | `forge repositories FORGE [--installation INSTALLATION] [--page PAGE]` | the picker's PAGE, from 1 (`../forge/family.md`, 4): a line per row, `OWNER/NAME`, `private` where it is, and `R/U`, its repository id and owner id for `repository adopt --ids`; then the call that reads page PAGE plus one while more follow, or that none do. `--installation INSTALLATION` is an accepted installation's id, as `forge list` prints it, GitHub alone | `Reachable { forge, installation?, page }` | |

   - **`forge list`'s text,** one line per forge, in number order: `forge N
     · host: HOST · KIND · MODE · STATE`, HOST the resource host used in
     charter documents (`../../protocol/configuration/settings.md`, 6), the state
     in the row's words (`../forge/deployment.md`), such as `forge 3 ·
     host: git.example:3000 · Forgejo · owned · Setting up: preparing`,
     then, when it has one, its first
     repository in the row's words, such as `· work/hello made, in
     Default`; then the digest (`tools-waiting.md`, 1). Its JSON is the
     rows' fields, including `host`, the state by its name with its step,
     its why, or its user.
   - **The names are the forge's and the page's.** The group `forge`
     keeps them apart from area 5's `account connect`, `account forget`
     and `account replace-key`, which are an LLM account's; `forge list`
     and `forge repositories` are reads, as the others are.
   - **Who may:** a party whose role holds the deployment's connectors;
     in standalone, the deployment's owner (`../engine.md`). Any
     other call of this area is refused by the engine, exit 2, in its
     words, before a forge is called; `repository adopt` without `--ids`
     stays the way of a party that may adopt and not read the picker.
   - **One result per call.** A connect or an install prints the link
     as text and returns; a pasted connect returns once the request is
     durable. The run goes on at the engine, and `forge list --until
     settled` follows it (`tools-waiting.md`, 1); its end, connected or why not, is the
     forge's row in `forge list`, never the call's status. No tool opens
     a link: the person opens it in a browser where the clients'
     listener reaches (`../forge/served.md`, 1).
   - **A pasted value is read from FILE,** never an argument, never
     typed and never printed: a private file, as area 5's key is (the
     user's own, no group or other bits, not reached through a link),
     within the bound of `bounds.md`, section 1. Group or other permissions are refused
     before reading, naming the file and its mode in octal. An admin's
     token is a bundle of one `AdminToken` item, within 4096 bytes in
     all; it is never restored as an RSA key or replayed from a carried
     request.
     `--project` and `--label` require `--first`; ORG and REPOSITORY are
     nonempty within the wire's bounds, and LABEL is 1 to 64 letters,
     digits, dots, dashes or underscores. A key's file and an admin's
     token's are their bytes, less a final newline. A tokens file is
     one item a line, `NAME VALUE`, NAME one of `api`, `git` and
     `webhook_secret`, each at most once, VALUE its bytes to the line's
     end, and no other line; a file that breaks it is a usage error,
     exit 1, nothing sent, the line's number named and never its value.
     The tool makes temper's bundle of the items
     (`../../protocol/credentials.md`, pasted values), sends it once and
     keeps no copy (`requests.md`, section 1).
   - **No pasted value is echoed:** not in a result, a `kept:` line,
     `--json`, an error or the help. `forge list` says that a value was
     given, and what the forge answered of it, never the value; an act
     it offers that takes one prints its call with `FILE`, the agent's
     own path to give.
   - **The refusals** are the engine's, exit 2, in `../forge/repositories.md`, 4's
     words: no such forge; the forge's kind or mode does not take the
     act (an install on Forgejo, a forget on a forge not managed, a
     replacement on a managed or owned forge, a connect of an owned
     one, a removal of the playground's forge); busy, since a forge runs one writing plan, a connect, an
     install or a reconcile, and one picker's read beside it, a second
     of either kind refused, as is one past `setup.runs`, which counts
     both; and, for `forge repositories`, the forge's reason (not
     connected, not installed, unreachable) and, on a managed or owned
     Forgejo, the admin's token's, missing or not a site admin's (skein's
     `forge/setup/forgejo.md`, 1.3 and 1.8).
   - **A page** is at most `picker.page` rows
     (`../../protocol/configuration/README.md`, its limits), clipped to the
     forge's own page cap, and says more follow when it is full; so a
     page past the last, the one after a full last page among them,
     answers no rows and no more, exit 0.
   - **"Not listed"** is `repository adopt` by `--owner` and `--name`
     without `--ids`; a repository temper cannot reach is refused, exit
     2, with what would make it reachable, the installation's page among
     it (`../forge/family.md`, 6; `../forge/repositories.md`, 4).

## 2. Open questions

- **The forge's read family's tools,** beside `forge list` (area 6), as the
  engine serves each of the forge's other parts (`../forge/served.md`).

- **Protection on a forge temper owns** (the `forge setup` row, `areas.md`): temper's
  bot stays a writer, so a rule a person later sets there stays unread, and
  `repository adopt` on that forge still prints the line. Granting the bot
  admin would let whatever holds its tokens change that rule, the second net
  (`../../domain/forge/adoption.md`), so it isn't done. Reading with the site admin's
  token, which temper holds there, would widen nothing, at the price of a
  path from the setup's host to adoption and to its later reads. Default: as it
  stands, until a person on an owned forge sets a rule temper should know.

- **A tokens file's layout** (`areas.md`, area 6): temper's bundle is items by
  number (`../../protocol/credentials.md`, pasted values), which no person
  writes. Default: `NAME VALUE` lines, as `areas.md` has them; JSON, or a file
  per token, if an agent needs one.

- **An agent and GitHub's browser steps:** a manifest's or an install's
  link needs a person's browser signed in on GitHub. Default: the tools
  print the link for a person and open nothing; tests drive its steps as
  plain requests on skein's fake forge (`testing.md`, 2, 3); an agent alone
  connects a Forgejo, or an App of its own with `github-app`.

- **The picker's read as a keyed request:** `forge repositories` draws
  a key as the other unmarked acts do, and the engine keeps its answer
  for the retention (`../README.md`, section 5, the picker's page as a
  read).
  Default: kept as any keyed request; a read of the forge's family once
  the engine serves one.

- **Waiting on one forge:** `forge list`'s wait watches every forge's
  row, so another forge's change answers `--until changed`. Default:
  `--until settled` or `--until connected:FORGE` names what is waited
  for; a watched read of one forge, `forge show FORGE`, if an agent
  needs one.
