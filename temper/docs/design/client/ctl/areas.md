# The tools' areas

The table grows area by area, each with its stories (`testing.md`).

1. **A chat to a landed change** (the first milestone's story,
   `../../README.md`, section 2): the host's tools of `tools.md`, 5; every read
   of `tools.md`, 2 other than the general proposal read of area 2; the watched
   reads' waits (`tools.md`, 4); every act of 5.3.
2. **Goals and plans:** `goal start --project PROJECT [--template TEMPLATE]
   [--charter CHARTER] [--priority PRIORITY] [--budget AMOUNT | --as-allowed] [--deadline TIME] TEXT`
   (`shapes::set_goal`, keyed), `proposal show PROPOSAL`, `task amend
   TASK [--rev REV] CHANGE... REASON`, `proposal withdraw PROPOSAL`,
   `seen`.

   `goal start` defaults to stock template 2; `--template` names an
   offered template, including its charter, contract, tracking, label,
   flags, LLM choice and symbolic resources. The chosen executor needs
   `--charter CHARTER`. Missing controls are collected before returning usage,
   exit 1, nothing sent: `template 2 needs: --charter CHARTER; --budget AMOUNT or --as-allowed; --deadline TIME; help goal start`.
   Only controls required by the selected template and absent from the call
   appear; fixed budget and deadline defaults are kept.
   A fixed executor takes its template's charter; also giving `--charter`
   fails as usage: `template 4's executor is charter 5; leave out --charter`
   (the numbers name that template and charter). A charter not offered
   in the project's creating bounds sends nothing, naming the template,
   the required charter and the offered charters. A budget or deadline
   past the offered ceiling has a separate diagnostic; a refusal beyond
   the direct bounds retains jig's reason. If the project has no creating
   offer, the diagnostic points to the role and the receiving bound
   `documents.lines` for charter and procedure lists. The fill and authority/proposal
   judgement are jig's (`jig/docs/design/client/domain/pages.md`, 6.2).
   Absent controls keep template defaults. A given budget requires
   `--budget` or `--as-allowed`; a given deadline requires `--deadline`,
   UTC to the minute as `2026-10-24T17:00Z`. Explicit budget or deadline
   controls replace the template's corresponding default before filling;
   `--priority` replaces a tracked template's priority. Model controls
   are area 5's; absent sub-agent flags preserve the template's list.

   The general `proposal show PROPOSAL` reads every proposal kind. For
   settings it prints the engine's current/origin/would-be review and its
   newest revision; retained ended and withdrawn records keep their state.
   An automatically replaced task proposal is unknown (7). It
   uses the same read vocabulary independently of the first area's
   settings proposal act.

   - **`task amend`'s changes,** one or more, each a control of the task,
     the rest kept as the task's document shows them (jig's
     `domain/tasks/changes.md`), drafted by the client domain's
     amendment editor (`../README.md`, 3.7):

     | Change | Sets |
     |---|---|
     | `--priority PRIORITY` \| `--untracked` | its tracking: tracked at PRIORITY, or untracked |
     | `--deadline TIME` \| `--no-deadline` | its authority's deadline, TIME as `2026-10-24T17:00Z` (UTC, to the minute), the rest of its authority restated as shown |
     | `--instructions FILE` | its instructions, whole: FILE's bytes, less a final newline; empty is none, said explicitly |
     | `--spec FILE` | its spec's words, whole; its parameters and inputs kept |
     | `--dependency TASK`... \| `--no-dependencies` | its dependencies, whole: those named, in order; what it names anew is added, what it leaves out removed |
     | `--model MODEL [--effort EFFORT]` | its main model and effort (area 5) |
     | `--budget AMOUNT` | its authority's spend, AMOUNT in the unit the client shows budgets in, the rest of its authority restated as shown |
     | `--tasks N` | the tasks its subtree may make over its life, the rest of its authority restated as shown |

     FILE is a path, or `-` for stdin, used once, as TEXT is.
   - **A budget or tasks raised is a widening,** funded by the party who
     gives it, who becomes the task's funder (jig's `domain/authority.md`,
     section 7, "Whoever widens a task funds it"): from their role's pool,
     within what the policy lets them give alone, or proposed past it, as
     the next bullets say. It is the way out a hold for a task's own
     budget or tasks names (jig's `domain/tasks/lifecycle.md`, 6; `../../domain/forge/change/repairs.md`
     ): `task amend 41 --rev 8 --budget 2000 "room for its review"`,
     then `task release 41 --hold 3`; for tasks, the hold prints
     `task amend 41 --rev 8 --tasks N REASON`, then the release. N is
     a replacement lifetime allowance with room for a repair and review;
     release never erases tasks already made. A hold whose lack the task's
     authority does not hold (a project's period, a pool) names `project
     fund` instead, and a hold no amendment lifts names the cancel.
   - **`--rev REV` is the spec's revision** that `task show TASK`
     prints, as the task's offered `amend` act carries it: `task show
     TASK`'s acts print it as `task amend 41 --rev 8 CHANGE... REASON
     --key NAME`. A change of a tracked task's priority alone names none
     (jig's `domain/tasks/changes.md`; `../../domain/agent/tools.md`);
     every other amendment does, and one without it exits 1, `temper
     ctl: task amend needs --rev REV`, nothing sent. The tool reads the
     task's document, drafts the change on it, and sends nothing when
     the document's revision is no longer REV: `temper ctl: task 41
     moved: its spec revision is 9, not 7; read again: task show 41`,
     exit 1.
   - **What the document must hold whole:** a deadline, a budget or
     tasks restates the task's authority, a spec's words its spec. Where the document cut
     it to the client's bounds, the tool sends nothing and exits 1,
     `temper ctl: task 41's authority is cut in this read; amend its
     deadline or tasks from a client with wider bounds` (or `spec`, `words`).
   - **Beyond the party's bounds,** an amendment follows `tools.md`, 3, compared
     by jig's admission for the task's offered `amend` act: within,
     sent; past what the party holds but proposable, a deadline later
     than its funder allows among them, sent as `Propose`, and
     `proposed: t41/3, to …`; past the bounds, refused, nothing sent.
     Each change is proposable (jig's `domain/tasks/README.md`, 4 and `jig/docs/design/domain/tasks/changes.md`); what
     fails another check than authority (a dependency out of reach, a
     circle, a goal past the project's goals) is the engine's refusal,
     exit 2.
   - **The same flag twice, or both of a pair,** exits 1 as usage;
     `--dependency` alone may repeat.
3. **Several parties:** `party make --project PROJECT --role ROLE --out
   FILE` (jig's `MakeService`, keyed; `README.md`, section 7), `sign-in --service
   FILE`, `deployment show`, `task move TASK REASON`, a person task's
   `task take`, `task hand-back` and `task answer TASK --rev REV`, and
   the project's own operations, `policy`, `role`, `pool` (jig's
   `domain/people/acts.md`, 2), past the spend per period that `project
   fund` sets (`tools.md`, 3).

   `task show TASK` joins a person task's document to its generic task:
   its question, result contract, choice codes and bounds, person revision,
   holder, and each offered act with its admission. Its holder prints as
   `taken by: PARTY`, or `taken by: none`. Once an answer has ended the
   task, `answered by: PARTY` takes that line's place, from the retained
   result's `chosen_by`. A closing person task with no holder prints
   neither: its live document does not say who answered. `--json` keeps
   `taken_by` as jig gives it. The inbox's person-task
   line prints `read: task show TASK`. Role-addressed work offers
   `task take TASK` and `task hand-back TASK`; it must be taken before an
   answer. These commands and person answers are optionally keyed.

   A person answer keeps the shown revision: `task answer TASK --rev REV
   CHOICE TEXT` for a choice and its reason, `task answer TASK --rev REV
   TEXT` for words, or `task answer TASK --rev REV --approve STATE [TEXT]`
   for the exact approval state, STATE as lowercase hexadecimal. Every
   contract also accepts `task answer TASK --rev REV --cannot REASON`.
   A stale revision reaches the engine and is refused; a retained keyed
   answer reaches it for replay even after the task ends. The numbered
   question form of `tools.md`, 3 keeps its separate question read and answer.

   An answer past what its contract asks, a report's words or the
   reason of the choice it names, reaches the engine and is refused as
   past its contract (exit 2). The act's read gave the contract, so the
   refusal names the bound: `task answer: Limit reached: task 44's reason
   for choice 1 is at most 128 bytes`, or `task 44's words are at most 128
   bytes`, and JSON's `bound` beside the refusal. The same refusal for any
   other cause (a choice the contract lacks, an answer of another form, a
   `--cannot` or an `--approve` past the deployment's result bound) has no
   bound the task asks for, and prints as it is; so does a restored
   request, which read nothing. TEXT's bound in the tool's line is the most
   any call takes: the tool's notes say a person task may ask for less.

4. **jig's plumbing whole:** `create`, `propose`, `compound --project
   PROJECT FILE`, a batch, a proposal or a compound request as JSON
   (keyed).
5. **LLM accounts** (jig's `domain/accounts.md`, section 5): the party's
   accounts and, for an owner, the deployment's, as temper's accounts
   page shows them (`../pages/README.md`). Each act is one of jig's
   requests for accounts (jig's `client/domain/contract/requests.md`, 2).

   | Tool | What it does | jig's | K |
   |---|---|---|---|
   | `account list` | the party's accounts, then, for an owner, the deployment's: each with its number, LLM provider, display name and holder; its sources in order, each with its kind, its name for a program's, and its state, per host for a program's, with the reset where the provider gave one; the runs on it and the tasks waiting for it; the sign-ins and checks in progress; its acts | the `accounts` watch (`tools.md`, 4) | |
   | `model list [--project PROJECT]` | the project's offer (jig's `client/domain/contract/documents.md`, 1), by LLM provider: each model with its efforts, its default effort, what it is good at, its prices and the accounts that can run it; a provider with no model there, its name and no models, never a model of its | `me`'s offer and providers | |
   | `account connect PROVIDER` | starts a sign-in with an LLM provider: prints the address to open in a browser on the engine's machine, and the call that follows it, `account list --until settled`; exits 0 once the request is durable | sign in | yes |
   | `account disconnect ACCOUNT [--source SOURCE]` | signs a source out: a signed-in source's record revoked at the provider, a program's login signed out on each host; the source removed. Unconfirmed, the answer says to disconnect temper in the provider's settings, or names the hosts that did not sign out | sign out | |
   | `account add-key PROVIDER FILE [--account ACCOUNT] [--deployment]` | adds a key read from FILE: to ACCOUNT, or as a new key-only account; the deployment's with `--deployment`, by an owner | add a key | yes |
   | `account replace-key ACCOUNT FILE` | replaces the account's key with the one read from FILE | replace a key | |
   | `account forget ACCOUNT [--source SOURCE]` | removes a source, or the account with every source, each signed out as `account disconnect` does | remove | |
   | `account declare PROVIDER ADAPTER NAME [--account ACCOUNT]` | declares a program's login under the source's NAME (1 to 32 of `a` to `z`, `0` to `9` and `-`): a new account, or a source of ACCOUNT; the answer names the command to run on each host with its directories, `temper-box program-login --config CONFIG --data DATA --cache CACHE NAME`; CONFIG, DATA and CACHE stand for that box's directories, and skips it where the programs directory's NAME is already signed in, then `account verify ACCOUNT --source program`, ACCOUNT as `account list` shows it (`../../protocol/credentials.md`, section 3) | declare a program's login | yes |
   | `account verify ACCOUNT [--source SOURCE]` | has each host check a program source's login now; answered once asked, then `account list --until settled` | check | |
   | `account rank ACCOUNT POSITION [--source SOURCE]` | the account's place among its holder's accounts of its LLM provider, from 1; with `--source`, the source's place in the account's order | order | |
   | `account retry ACCOUNT [--source SOURCE]` | resumes a limited source, so that the runs it holds start again | resume | |
   | `account export ACCOUNT FILE` | moves a signed-in account out, by its holder: FILE made new and private before anything is sent; the signed-in source withdrawn, so that no run takes it and no refresh follows; its record written to FILE and synced; then *written*, and the engine erases it. Prints `moved out: account N to FILE`. FILE already there, exit 1, nothing sent. Refused, exit 2, FILE removed: "not signed in" (no signed-in source: a key is added again, a program's login is per host), "moving", "cancelled" (*written* past the move's deadline). A record not written, or lost on its way, is not erased: exit 1, FILE removed, and the engine cancels the move at its deadline | move out, then *written* | |
   | `account import FILE` | moves a record in from FILE, read as a key's file is: taken into the secrets' store, checked as this application's and its subject held by no other holder, the account made or the one of its subject renewed; prints `moved in: account N`, `, renewed` after it for a renewal, then removes FILE. FILE refused (not private, a link, empty, past the bound), exit 1, nothing sent. Refused, exit 2, FILE kept: "not ours" (another application's or LLM provider's record), "held" (its subject held by another holder), "moving" (its subject's account moving out here) | move in | yes |

   - **The verbs are plain verbs,** as the other tools' are. Eight are
     not jig's words: `sign-in` and `sign-out` are the host's (`tools.md`, 5), so
     a sign-in with a provider is `account connect` and its end `account
     disconnect`; `account forget`, `account rank`, `account verify` and
     `account retry` are jig's remove, order, check and resume; `account
     export` and `account import`, its move out and in.
   - **Offers and readiness are supplied facts.** In JSON, `me` prints
     `providers`, `providers_cut` and `key_bytes`; `account list` prints the
     party's `readiness` and `readiness_cut`, and an owner's
     `deployment_facts` holds the deployment's offers and readiness beside
     its `deployment` account rows. In text, `me` prints each offer as
     `provider P · NAME · sign-in, key, programs X · manage usage URL`,
     including only the offered methods and supplied usage address, then
     keeps its `cut:` line. Account text starts with `current: true|false`
     and prints `account N · provider P · NAME · DISPLAY · yours|party N|deployment`.
     Under it, sources stay in supplied order:
     `  source S · KIND · NAME · STATE · usable|not usable`, omitting NAME
     except for a named source, and retaining supplied limits and resets.
     A program source has `    host H: usable|not signed in|signed in otherwise (M)|failed R`
     for its supplied host observations (unknown while no state is supplied),
     and, after the fields, its `account verify N --source program` call when
     that source is unique. A complete account held by the caller also
     offers `account forget N`. Account payers, usage
     addresses, runs and waiting tasks follow their supplied facts; absent
     party activity is unavailable. `signing:` and `last end:` print when
     present. `readiness: provider P · party N|project N · usable|unavailable|unknown`
     prints each party observation once, with settled when supplied; an
     owner's `deployment:` holds its accounts and offers, with
     `deployment readiness:` only for observations different from the party's.
     A supplied readiness cut is named `readiness cut: true (jig)` (with
     `deployment` for that list); `cut: true` prints only on loss, before
     the digest. Nested loss contributes to the document's `cut`. Each
     readiness list has its own window, preserving both supplied observations.
     Account and catalogue takes charge structural bytes within `README.md`, section 9's
     result text ceiling; nested sources, hosts, programs and offers do not
     consume an account list's row count. Text presents the bounded JSON
     projection, sharing its omissions and digest for reads and waits.
   - **Model text** names `models for project N`, then `provider P · NAME`
     and each `model NAME`. Each model names its description, window and
     output in tokens, efforts, and input, cached input, cache write and
     output prices per the supplied token unit, in deployment spend units.
     Missing provider numbers, names and descriptions print `unknown`;
     an empty effort list prints `none listed`. The accompanying accounts
     use the account text above, preserving supplied source and readiness
     states without inferring provider readiness. Loss prints `cut: true`
     and the result keeps its `models` digest; JSON retains its records.
   - **One result per call.** `account connect` prints the address and
     returns; the sign-in goes on at the engine, and `account list
     --until settled` follows it (`tools.md`, 4). It never opens a browser, and
     the provider's redirect reaches the engine's loopback, so the
     browser that approves runs on the engine's machine
     (`../../protocol/credentials.md`, section 3). `account verify` alike:
     asked, then waited for.
   - **A key is read from FILE,** never an argument and never printed:
     a private file, the user's own with no group or other bits and not
     reached through a link, as the engine reads a secret's file
     (`../../protocol/configuration/README.md`, section 5); its bytes, less a final
     newline, within the bound of `README.md`, section 9. The tool reads it, sends
     it once, and keeps no copy (`README.md`, section 5).
   - **`PROVIDER` is an LLM provider's number,** as `model list` and
     `account list` print it (jig's `protocol/configuration.md`, section
     3); `ADAPTER` is the program's name in smith's agent configuration,
     `claude_code` (smith's `protocol/agent.md`, section 4).
   - **`SOURCE` names one of the account's sources** by its kind,
     `sign_in`, `key` or `program`, as `account list` prints it (`README.md`, section 12
     ).
   - **A declaration after forgetting** may use the same NAME for its
     original holder (jig's `domain/accounts.md`, 3.4). Use a fresh
     request key for the new declaration, since a key keeps its first
     answer.

   - **A program's login is never the tools'.** `account declare` makes
     the source; the person, or the harness, runs `temper-box program-login`
     on each host, outside `temper ctl`, since it hands the terminal to
     the program's own sign-in (`../../protocol/credentials.md`, section 3).
     The printed act names `--config CONFIG --data DATA --cache CACHE`,
     placeholders for that box's three directories: ctl reads no box
     configuration. Skip it where the programs directory's NAME is
     already signed in. Then `account verify ACCOUNT --source program`,
     ACCOUNT as `account list` shows it.
   - **A refused account act says why and what next,** where jig's
     refusal of that act has one reason a person can act on (jig's
     `domain/accounts.md`, section 5), rather than the refusal's word
     alone. Text prints, exit 2, `TOOL: WHY`, then the next step on a
     line of its own; JSON keeps the refusal and adds the next step as
     `next`:

     | Act, refused | WHY | Next step |
     |---|---|---|
     | `account declare`, *name taken* | `program name NAME is taken` | `Declare under another name. If account list shows a source of yours named NAME, check its login instead: account verify ACCOUNT --source program` |
     | `account declare`, *not offered* | `provider P offers you no program login` | `me prints each provider's ways; declare with a provider whose line names programs` |
     | `account retry`, *not offered* | `the source is not limited, so there is nothing to resume` | `account list shows its state and what it needs` |

     NAME and P are the declaration's own. A forgotten program source's
     name is declared again by the person who held it, as jig's
     `domain/accounts.md`, 3.4, has it, and is answered as a first
     declaration is.
   - **The task's model** is chosen with this area: `chat start` and area
     2's `goal start` take `--model MODEL [--effort EFFORT]`, the task's
     LLM choice, the template's choice or what each run's check takes without them (jig's
     `domain/tasks/README.md`, section 3), and area 2's `task amend` changes it.
     Both take `--sub-agent MODEL[:EFFORT]` too, repeatable, each one of
     the choice's sub-agents in the order given (jig's
     `protocol/configuration.md`, 3.1), a model at most once, its effort its
     default where left out as `--model`'s is; given,
     they replace the template's sub-agents whole, and they need
     `--model`. Without them the choice keeps the template's sub-agents,
     or names none.
     Left out, the effort is the model's default in the offer; given, it
     names one of the model's efforts. A model the offer in `me` leaves out
     is refused before anything is sent, `model not offered: MODEL`. An
     amendment replaces the main choice whole; its effort is
     the caller's, never inherited from the task's old main.
   - **An account's uses** have no tool: each takes jig's default (jig's
     `domain/accounts.md`, 4.2; `README.md`, section 12).
   - **An account moved,** `account export` and `account import`, is
     jig's move (jig's `domain/accounts.md`, section 6) through
     `jig-client`'s `move_out`, `written` and `move_in` (jig's
     `client/domain/client/requests.md`): a fresh deployment takes a
     signed-in account in and gives it back, as a host with no browser
     takes a record signed in on a computer with one. The record is never printed, never an argument, and in no
     request or answer kept (`README.md`, section 5). `task move` and `task take` are
     area 3's names.
   - **The file's form is jig's,** opaque to temper: a moved record,
     its version, then the application's name, the issuer, the client
     id, the subject, the display name and skein's record (jig's
     `protocol/llm.md`, 5.6). The application's name is
     `temper`, temper's root's constant, the same in every deployment
     and never configured, so a file moves between any two temper
     deployments whose LLM provider has the same registration (issuer
     and client id, as ChatGPT's is: `../../protocol/setup/init.md`, 1), and
     is refused "not ours" by another application built on jig.
   - **The file's versions** (`../../draft/upgrades.md`, 4.2 to 4.4): it
     carries two families, each with its version, a version naming one
     layout: jig's moved form (form 1) and, inside it, skein's
     OAuth record (its record version). It holds one value, not a
     store, so no store's levels; the nesting fixes a patch's order,
     the form's patches first, then skein's on the record inside it.
     `account export` writes the form the engine's jig makes. A newer
     `account import` reads every older form and patches it, the form's
     patches and then the record's, before the engine's checks (`README.md`, section 11
     ). An older `account import` refuses a newer form in its own
     words before anything is sent (below), since the older one is
     whatever is deployed: the words must be in form 1's `account
     import`.
   - **`account export`, step by step,** in the shell
     (`temper-client-shell-cli`), the tools' root deciding each step:
     1. FILE is made new before anything is sent: an exclusive create,
        so that no name already there, a link among them, is opened,
        with no group or other bits (0600). FILE there: `temper ctl:
        FILE exists`, exit 1.
     2. `move_out` is sent once, under a name the table draws and never
        keeps (`README.md`, section 5).
     3. Answered *moving out*, the record is written to FILE whole,
        FILE synced and then its directory, and only then is `written`
        sent with the move's number, an ordinary request, kept in
        DIRECTORY until answered (`README.md`, 6.2). Answered done, the tool prints
        `moved out: account N to FILE`, FILE as given, and exits 0:
        FILE is the account's only holder.
     4. Until `written` is sent, every failure removes FILE, which
        then holds no record the engine has let go: a refusal ("not
        signed in", "moving"), exit 2; the record not written (a write
        or a sync failed), `temper ctl: FILE not written: WHY; the
        engine keeps the account and cancels the move at its
        deadline`, exit 1; the answer lost (jig-client's *lost*: the
        exchange ended with no whole answer, or the answer came with no
        record), `temper ctl: the record of the move was lost on its
        way; the engine keeps the account and cancels the move at its
        deadline; export again after it`, exit 1; *unsent*, as every
        call's, exit 1; a signal, 128 plus its number.
     5. `written` refused "cancelled" (the move's deadline passed, or
        the engine restarted, and its record was refreshed once at the
        engine, so FILE's is dead): FILE is removed, exit 2.
     6. Once `written` is sent and its answer has not come (the link
        down past its deadline, a signal), FILE is kept: it is the only
        holder if the engine took `written`, and a dead record if the
        move was cancelled. The call exits as `results.md`, 3 says, and the next
        call sends the kept `written` (`README.md`, section 12).
   - **`account import`, step by step:** FILE is read as a key's file is
     (above), its bytes whole, nothing trimmed, within the bound of
     `README.md`, section 9; refused, exit 1, nothing sent, the line naming FILE's
     fault and never its bytes. Its form version, the file's first two
     bytes, is read against the form this temper's jig reads: a newer
     form, `temper ctl: FILE holds a moved record of form vN; this temper
     reads up to vM: upgrade temper`, exit 1, nothing sent, FILE kept.
     Any other file is sent, and the engine refuses one it cannot read
     "not ours", as it does a form newer than the engine's jig reads
     when ctl's and the engine's jig differ (`README.md`, section 12). `move_in` carries the bytes once under
     `--key NAME`, never kept (`README.md`, section 5). Answered *moved in*, the tool
     prints the account, then removes FILE, whose record is dead once
     the engine refreshes it; a removal that fails exits 1, `temper
     ctl: FILE not removed: WHY; its record is spent`, after the
     account's line. Every other end keeps FILE, the only holder of the
     record: a refusal, exit 2; *unsent*, *lost* or the link down,
     exit 1, and the agent asks again with the same `--key` and FILE,
     answered with the first answer within the engine's retention.
6. **Forges** (`../forge/repositories.md`, 1 and `../forge/deployment.md` and `../forge/repositories.md`, 4): each forge the store
   keeps as temper's `Forges` page shows it, its set ups and connects,
   which make a forge when no FORGE is named (`../../protocol/forge/engine.md`,
   2), its removal, the playground's start and reset, and the
   adoption form's picker. Each act is one of the FORGE requests (`../engine.md`
   ), made through `jig-client` as the forge family's bytes,
   as `repository adopt` is.

   | Tool | What it does | temper's | K |
   |---|---|---|---|
   | `forge list` | each forge the store keeps by its number, kind and `web` address: its connection, "connected as LOGIN (ID)", connecting and what it waits for, stopped and why, not set up, setting up and at which step, set up failed and why, or not connected and why; how its last connect ended, a GitHub App's lacking permissions and events each named; its mode, and "admin's token kept" for managed and owned; on GitHub, the App's slug and its installations, each with its id, its account's login, all or selected repositories, suspended, or not accepted (a stray), "not accepted · kept", with its act, `forge accept FORGE INSTALLATION`; a job's log on a host it does not read, with how to add it; its load; its acts | the forge's deployment part, read and watched (`../forge/served.md`) | |
   | `forge connect [FORGE] github` | starts GitHub's manifest flow, FORGE a kept GitHub forge, or, with none, a new one, the App made under the person's own account and installed on any account GitHub offers them: prints the link to open in a browser, the manifest's page on the clients' listener, and the call that follows it, `forge list --until settled`; exits 0 once the request is durable | `Connect { forge, how: GitHubManifest { back: none } }` | yes |
   | `forge connect [FORGE] github-app APP FILE [--read-only]` | connects an App of the person's own, FORGE a kept GitHub forge, or, with none, a new one: APP its id, FILE its private key, PEM as GitHub gives it; no webhook secret; temper points the App's hook at `https://SLUG.invalid/` and says so in its output, unless `--read-only` leaves it as read; then `forge list --until settled` | `Connect { forge, how: GitHubApp { id, read_only?, key } }` | yes |
   | `forge connect [FORGE] forgejo FILE [--api HOST:PORT --git URL [--tls NAME] [--web URL] [--git-program FILE]]` | connects a Forgejo through its bot's tokens, FILE a tokens file (below) naming `api`, `git` and, where the person made hooks to temper's listener, `webhook_secret`: FORGE a kept connected Forgejo, not connected or stopped, or, with none, a new one, connected, at `--api` and `--git`, both required without FORGE and refused with it, as are the three that follow, as `forgejo-admin`'s are; the setup host checks that both tokens answer as one user, not a site admin, and on a kept forge as its kept user, then keeps them and connects as that user; prints `connecting` and the call that follows, `forge list --until connected:FORGE`, and exits 0 once the request is durable; refused, exit 2, in `ForgeRefusal`'s words (`../forge/records.md`) | `Connect { forge, how: ForgejoTokens }`, `forge` the kept one or a `connected` `ManagedForge`, the tokens and the secret carried as `ApiToken`, `GitToken` and `WebhookSecret` | yes |
   | `forge connect [FORGE] forgejo-admin FILE [--first ORG/REPOSITORY [--project PROJECT] [--label LABEL]] [--api HOST:PORT --git URL [--tls NAME] [--web URL] [--git-program FILE]]` | connects a Forgejo as its site admin, FILE the admin's token: FORGE a kept managed Forgejo, or, with none, a new one, managed, at `--api`, its API's host and port, and `--git`, its remotes' base, both required without FORGE and refused with it, as are the three that follow, with `--tls`, its TLS server name, plaintext without it, `--web`, its pages, and `--git-program`, the git its workers run, theirs without it (`../forge/records.md`, `ManagedForge`); Reconcile makes or finds its marked bot, never TakeOver; with `--first`, makes ORG/REPOSITORY with the starter workflow, unadopted, PROJECT a number or name as for `forge setup`, default a new project named REPOSITORY, held in the call's own run and never sent in `first`, runner LABEL default `docker`; the token is kept only after Reconcile and Create are Done, and every end erases the carried value; without `--first`, prints `connecting` and `forge list --until connected:FORGE`; with `--first`, prints `connecting`, waits for Made and Connected, then shares `forge setup`'s MakeProject and adoption continuation, as PROJECT's home unless it has one, with key `first-repository-F`; prints the project number and adoption (JSON: `project` and `outcome`) and exits 0; a stop or failed Create exits 1 naming the reason or step; an ORG equal to the bot's login, case-folded, is refused before any write and exits 2, `first_is_bot`, "the organization is the bot's login: name another organization" | `Connect { forge, how: ForgejoAdmin { first? } }`, `forge` the kept one or a `ManagedForge`, token carried as AdminToken | yes |
   | `forge install FORGE` | prints a fresh link to install temper's App on another account, GitHub alone; the installation shows on `forge list` once the person's browser comes back | `Install { forge, back: none }` | |
   | `forge accept FORGE INSTALLATION` | accepts INSTALLATION, an installation of temper's App that exists already on GitHub and that `forge list` shows not accepted, with no browser: read on GitHub as the App's, then kept as an installation accepted through the setup route is (`../../protocol/setup/github.md`, 4); prints `accepted: installation I on forge F` and the call that follows, `forge repositories FORGE --installation INSTALLATION`, exit 0, and the same when it is accepted already; refused, exit 2, as `forge install` is, or, its run stopped, in the forge's words for its reason (`../forge/records.md`): "GitHub has no installation I of this App", "GitHub refused the App's read of installation I", "GitHub did not answer: try again", "temper keeps at most N installations" | `Accept { forge, installation }` | |
   | `forge replace FORGE FILE` | replaces what a person pasted, on a forge connected or stopped: on GitHub, FILE the App's new key, PEM as GitHub gives it, which must be a key of the same App; on a connected Forgejo, a tokens file naming `api`, `git` or both, and `webhook_secret` when the hooks' secret changes, which must answer as the forge's user; each new value is kept over the old, and the forge connects with them; prints `connecting` and `forge list --until connected:FORGE`, and exits 0 once the request is durable; refused, exit 2, in `ForgeRefusal`'s words, `not connected` on a forge not connected and `busy` while it connects | `Replace { forge }`, the key or the tokens carried | yes |
   | `forge forget FORGE` | erases a managed forge's admin's token; the forge is connected, its bot and tokens kept | `Forget { forge }` | |
   | `forge setup [FORGE] [--org ORG] [--repository REPOSITORY] [--project PROJECT] [--playground]` | sets up an owned Forgejo here, under the box, with its first repository, FORGE a kept owned Forgejo, or, with none, a new one, made as the request is admitted (`../../protocol/forge/engine.md`, 2): the box fetches its binaries, prepares it and starts Forgejo and its runner, then temper takes its handover, makes `ORG/REPOSITORY` (default `work/hello`) with the starter workflow, or with `--playground` makes `playground/widget` as the playground's copy, its files beside the starter workflow, taking no `--org` or `--repository` (a usage error, exit 1, with either), and connects (`../../protocol/setup/owned.md`, 5 and 7); prints `setting up`, waits as `forge list --until made:FORGE` does, then adopts `ORG/REPOSITORY` into PROJECT, a project's number or name, a name no project holds made a new project first by jig's `MakeProject`, and with no `--project` a new project named REPOSITORY (`../../draft/archive/first-run.md`, 2.4); PROJECT is the call's own, held in its run and never sent with `SetUp`, which names no project (`../../protocol/setup/owned.md`, 7), so that a call restored from its kept `SetUp` takes it from its own arguments; it adopts as `repository adopt --project PROJECT --forge FORGE --owner ORG --name REPOSITORY --landing main` would, as PROJECT's home unless PROJECT already has one, under the same sign-in and with the key `first-repository-F`, so that the call run again adopts nothing twice; prints the adoption, without `../forge/family.md`, 6's line on protection it could not read, and exits 0. That line is true there, temper's bot being a writer as it is anywhere (`../../domain/forge/adoption.md`), but it asks nothing a person can do on a repository temper has just made; `--json` keeps `protection`, and `repository adopt` keeps the line. Refused, exit 2: "not offered" (an owned forge already exists when FORGE is absent, or the forge is not an owned one whose setup says `"start": "asked"`, or it is set up, setting up or connecting), "busy" (the box is setting up another forge), "invalid" (ORG or REPOSITORY not a name Forgejo takes, or ORG, REPOSITORY or PROJECT's name empty or too long, PROJECT's checked by the call before it sends), or, without the box, "not under the box", saying to start temper with `temper box` (`../forge/deployment.md`; `../../protocol/setup/box.md`, 4). A not-offered refusal in text and JSON points to `forge list`, `project make NAME --key KEY`, then `repository make --project PROJECT --forge FORGE ORG/NAME --until adopted --key KEY`; the single-owned-forge rule remains. Set up failed, or its first repository failed, exit 1, naming the step. With `--playground`, ORG and REPOSITORY above are `playground` and `widget`, and the default project a new one named `widget` | `SetUp { forge, org, repository, playground }`; then `MakeProject` when PROJECT names no project, and `Adopt` | |
   | `playground start` | starts the playground (`../../protocol/setup/playground.md`): the engine makes its fake forge, the box starts its fakes, and the engine adds the scripted model's account and adopts its repository into its project, in the caller's name; waits for the answer, then prints `running` and exits 0, the same when it already runs. Refused, exit 2: "not offered" (a real forge kept). Start failed, exit 1, naming why: the fakes did not start (the box's why), the account refused, the adoption refused, or the forge not connected in time; run again, it makes what is not done | `StartPlayground` | |
   | `playground end` | resets the playground (`../../protocol/setup/playground.md`), as the banner's "set up temper" does (`../forge/family.md`, 7): its project deleted, its fake forge removed and the scripted model's account removed, each a kept act the engine resumes at its next start if one is cut; waits for the answer, then prints `reset` and exits 0, once the playground is gone. Refused, exit 2: "not offered" (no playground) | `EndPlayground` | |
   | `forge remove FORGE` | removes forge FORGE (`../../domain/forge/adoption.md`, 1): ended at once, then, by kept acts, its live work cancelled, its box group stopped and its directory removed, its records erased; on the forge itself only what temper ran is undone, a managed Forgejo's bot and hooks and a GitHub App staying the person's. Prints `removed` and exits 0 once the forge is ended. Refused, exit 2, in `ForgeRefusal`'s words: not the deployment's owner, no such forge, or not taken by this forge (the playground's, which `playground end` removes) | `RemoveForge { forge }` | |
   | `change show TASK [--project PROJECT]` | the panel of change task TASK (`../forge/served.md`, 2; `../forge/records.md`), read as the change's page watches it, in the scope of PROJECT, the task's project when none is named: its repository by forge, owner and name; its branch and landing branch; its head; its pull request's number and address on the forge; its step in `../forge/README.md`, 2.1's words, with its place in its queue, a repair's reason or, landed, its merge commit, whole; its hold; the task its step runs; CI at its head; its gates, each with its kind, whether it blocks and its verdict; its repairs, resolutions and updates, each used of its limit; its questions (`../forge/served.md`, 2): its route, the questions taken used of its bound, and one line per open question, its asker's task and the question's number, where it stands (`taken`, `with foreman F`, `asked of requester`, `answered, held for dependencies`) and the first line of its words; then the call that follows it, `change show TASK --until landed` while it is live and has not landed, `change show TASK --until cleared` while it is verifying or red. An ended step points to `task show TASK` for its report; a successful no-change report adds `completed, no change`. A landed or ended change keeps its last panel. `task show TASK` of a change task, live or ended, prints `change show TASK` among the calls that follow it. A task that is no change into a repository of PROJECT, or a change the forge has not registered yet, prints `no change panel for task N`, exit 2 | the forge's `change` subject, read once (`../forge/served.md`, 2) | |
   | `forge repositories FORGE [--installation INSTALLATION] [--page PAGE]` | the picker's PAGE, from 1 (`../forge/family.md`, 4): a line per row, `OWNER/NAME`, `private` where it is, and `R/U`, its repository id and owner id for `repository adopt --ids`; then the call that reads page PAGE plus one while more follow, or that none do. `--installation INSTALLATION` is an accepted installation's id, as `forge list` prints it, GitHub alone | `Reachable { forge, installation?, page }` | |

   - **`forge list`'s text,** one line per forge, in number order: `forge N
     · host: HOST · KIND · MODE · STATE`, HOST the resource host used in
     charter documents (`../../protocol/configuration/settings.md`, 6), the state
     in the row's words (`../forge/deployment.md`), such as `forge 3 ·
     host: git.example:3000 · Forgejo · owned · Setting up: preparing`,
     then, when it has one, its first
     repository in the row's words, such as `· work/hello made, in
     Default`; then the digest (`tools.md`, 4). Its JSON is the
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
     settled` follows it (`tools.md`, 4); its end, connected or why not, is the
     forge's row in `forge list`, never the call's status. No tool opens
     a link: the person opens it in a browser where the clients'
     listener reaches (`../forge/served.md`, 1).
   - **A pasted value is read from FILE,** never an argument, never
     typed and never printed: a private file, as area 5's key is (the
     user's own, no group or other bits, not reached through a link),
     within the bound of `README.md`, section 9. Group or other permissions are refused
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
     keeps no copy (`README.md`, section 5).
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

7. **Landed drafts** (`../../domain/forge/drafts.md`): a writer saying
   that a draft landed in one of a project's repositories is ready to
   start a goal, or no longer is, and the project's landed drafts listed,
   as Work draws the ready ones (`../pages/work.md`, 1). The group
   `draft`. Each act is the FORGE request `SetDraft`
   (`../forge/records.md`), made through `jig-client` as `repository
   adopt`'s is; OWNER and NAME name a repository adopted into PROJECT,
   both or neither, the project's home with neither.

   | Tool | What it does | temper's | K |
   |---|---|---|---|
   | `draft ready --project PROJECT [--owner OWNER --name NAME] [--chat TASK] PATH` | says the draft at PATH is ready to start a goal: temper reads it at the head of the repository's landing branch and keeps it, with TASK the chat it was written in; prints `ready: OWNER/NAME PATH at COMMIT on BRANCH`, COMMIT its first 12 digits, then its title where it has one, exit 0 | `SetDraft { change: ready { chat } }` | yes |
   | `draft not-ready --project PROJECT [--owner OWNER --name NAME] PATH` | says it no longer is; Work stops listing it; prints `not ready: OWNER/NAME PATH`, exit 0 | `SetDraft { change: not_ready }` | yes |
   | `draft list --project PROJECT` | the project's landed drafts, a watched read of the subject `drafts`: a line each, newest first, `OWNER/NAME PATH`, then `ready`, `not ready` or `goal G`, COMMIT's first 12 digits, BRANCH, when it was said ready in the clock form, `chat T` where it names one, and its title; "No landed drafts." with none, exit 0 | the subject `drafts` | |

   - **The refusals,** exit 2, each saying what to do: `no repository`,
     "no repository OWNER/NAME in PROJECT", or "PROJECT has no home
     repository: name one with --owner and --name"; `path`, "PATH is no
     draft's path: a relative path ending .md"; `missing`, "no PATH at
     the head of the landing branch: land it first"; `not a file`, "PATH
     is a directory"; `unreachable`, "the forge did not answer: try
     again"; `unknown`, "no landed draft at PATH"; `full`, "PROJECT keeps
     64 landed drafts, every one listed: say one not ready first". A
     refusal at admission is jig's, as for `repository adopt`.
   - **`--json`** prints the outcome's row, or the panel's rows, as the
     wire has them, a commit whole.

- **Areas 1, 5, 6 and 7 are designed here in full.** Areas 2 to 4 name their
  tools and jig's operations; each area's arguments are checked against
  jig's operations as they stand when the area is built.
- **A tool no area names** is a design question, and joins the table as
  a row of an area.
