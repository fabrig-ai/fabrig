# LLM accounts through ctl

This document holds LLM accounts through ctl, split from [`areas.md`](areas.md).

## 1. LLM accounts
5. **LLM accounts** (jig's `domain/accounts.md`, section 5): the party's
   accounts and, for an owner, the deployment's, as temper's accounts
   page shows them (`../pages/README.md`). Each act is one of jig's
   requests for accounts (jig's `client/domain/contract/requests.md`, 2).

   | Tool | What it does | jig's | K |
   |---|---|---|---|
   | `account list` | the party's accounts, then, for an owner, the deployment's: each with its number, LLM provider, display name and holder; its sources in order, each with its kind, its name for a program's, and its state, per host for a program's, with the reset where the provider gave one; the runs on it and the tasks waiting for it; the sign-ins and checks in progress; its acts | the `accounts` watch (`tools-waiting.md`, 1) | |
   | `model list [--project PROJECT]` | the project's offer (jig's `client/domain/contract/documents.md`, 1), by LLM provider: each model with its efforts, its default effort, what it is good at, its prices and the accounts that can run it; a provider with no model there, its name and no models, never a model of its | `me`'s offer and providers | |
   | `account connect PROVIDER` | starts a sign-in with an LLM provider: prints the address to open in a browser on the engine's machine, and the call that follows it, `account list --until settled`; exits 0 once the request is durable | sign in | yes |
   | `account disconnect ACCOUNT [--source SOURCE]` | signs a source out: a signed-in source's record revoked at the provider, a program's login signed out on each host; the source removed. Unconfirmed, the answer says to disconnect temper in the provider's settings, or names the hosts that did not sign out | sign out | |
   | `account add-key PROVIDER FILE [--account ACCOUNT] [--deployment]` | adds a key read from FILE: to ACCOUNT, or as a new key-only account; the deployment's with `--deployment`, by an owner | add a key | yes |
   | `account replace-key ACCOUNT FILE` | replaces the account's key with the one read from FILE | replace a key | |
   | `account forget ACCOUNT [--source SOURCE]` | removes a source, or the account with every source, each signed out as `account disconnect` does | remove | |
   | `account declare PROVIDER ADAPTER NAME [--account ACCOUNT]` | declares a program's login under the source's NAME (1 to 32 of `a` to `z`, `0` to `9` and `-`): a new account, or a source of ACCOUNT; the answer names the command to run on each host with its directories, `temper-box program-login --config CONFIG --data DATA --cache CACHE NAME`; CONFIG, DATA and CACHE stand for that box's directories, and skips it where the programs directory's NAME is already signed in, then `account verify ACCOUNT --source program`, ACCOUNT as `account list` shows it (`../../protocol/credentials-llm-accounts.md`, section 1) | declare a program's login | yes |
   | `account verify ACCOUNT [--source SOURCE]` | has each host check a program source's login now; answered once asked, then `account list --until settled` | check | |
   | `account rank ACCOUNT POSITION [--source SOURCE]` | the account's place among its holder's accounts of its LLM provider, from 1; with `--source`, the source's place in the account's order | order | |
   | `account uses ACCOUNT [--party N]... [--project N]...` | sets exactly the named parties and projects, at most 16 together; `account uses ACCOUNT --default` restores jig's defaults; prints `done: uses` (`requests.md`, section 1) | set uses | |
   | `account retry ACCOUNT [--source SOURCE]` | resumes a limited source, so that the runs it holds start again | resume | |
   | `account export ACCOUNT FILE` | moves a signed-in account out, by its holder: FILE made new and private before anything is sent; the signed-in source withdrawn, so that no run takes it and no refresh follows; its record written to FILE and synced; then *written*, and the engine erases it. Prints `moved out: account N to FILE`. FILE already there, exit 1, nothing sent. Refused, exit 2, FILE removed: "not signed in" (no signed-in source: a key is added again, a program's login is per host), "moving", "cancelled" (*written* past the move's deadline). A record not written, or lost on its way, is not erased: exit 1, FILE removed, and the engine cancels the move at its deadline | move out, then *written* | |
   | `account import FILE` | moves a record in from FILE, read as a key's file is: taken into the secrets' store, checked as this application's and its subject held by no other holder, the account made or the one of its subject renewed; prints `moved in: account N`, `, renewed` after it for a renewal, then removes FILE. FILE refused (not private, a link, empty, past the bound), exit 1, nothing sent. Refused, exit 2, FILE kept: "not ours" (another application's or LLM provider's record), "held" (its subject held by another holder), "moving" (its subject's account moving out here) | move in | yes |

   - **The verbs are plain verbs,** as the other tools' are. Eight are
     not jig's words: `sign-in` and `sign-out` are the host's (`host.md`), so
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
     present. `readiness: provider P · party payer N|project payer N · usable|unavailable|unknown`
     prints each party observation once, with settled when supplied; an
     owner's `deployment:` holds its accounts and offers, with
     `deployment readiness:` only for observations different from the party's.
     A row names whose payment it is for. A project payer's row is the
     project paying for its own work, which a plan account covers only
     as jig's `domain/accounts-uses.md`, 1.2 allows; work paid from a party's
     pool uses the party payer's row. So an unavailable project payer
     beside a usable party payer is no fault: the first readiness list
     that prints a project payer's row is followed, once in the text, by
     `readiness payers: a project payer's row is for work the project pays for itself, while a party's pool work uses the party payer's row`,
     and a text with no project payer's row has no such line. The
     readiness values, the `uses:` lines (`party N|project N`) and JSON
     are unchanged. `account list`'s reference says the same in its
     notes: `Each readiness row names its payer: a project payer's row is for work the project pays for itself, while your party's pool work uses its party payer's row.`
     A supplied readiness cut is named `readiness cut: true (jig)` (with
     `deployment` for that list); `cut: true` prints only on loss, before
     the digest. Nested loss contributes to the document's `cut`. Each
     readiness list has its own window, preserving both supplied observations.
     Account and catalogue takes charge structural bytes within `bounds.md`, section 1's
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
     --until settled` follows it (`tools-waiting.md`, 1). It never opens a browser, and
     the provider's redirect reaches the engine's loopback, so the
     browser that approves runs on the engine's machine
     (`../../protocol/credentials-llm-accounts.md`, section 1). `account verify` alike:
     asked, then waited for.
   - **A key is read from FILE,** never an argument and never printed:
     a private file, the user's own with no group or other bits and not
     reached through a link, as the engine reads a secret's file
     (`../../protocol/configuration/README.md`, section 5); its bytes, less a final
     newline, within the bound of `bounds.md`, section 1. The tool reads it, sends
     it once, and keeps no copy (`requests.md`, section 1).
   - **`PROVIDER` is an LLM provider's number,** as `model list` and
     `account list` print it (jig's `protocol/configuration-engine.md`, section
     1); `ADAPTER` is the program's name in smith's agent configuration,
     `claude_code` (smith's `protocol/agent.md`, section 4).
   - **`SOURCE` names one of the account's sources** by its kind,
     `sign_in`, `key` or `program`, as `account list` prints it (`README.md`, section 12
     ).
   - **A declaration after forgetting** may use the same NAME for its
     original holder (jig's `domain/accounts-sources.md`, 1). Use a fresh
     request key for the new declaration, since a key keeps its first
     answer.

   - **A program's login is never the tools'.** `account declare` makes
     the source; the person, or the harness, runs `temper-box program-login`
     on each host, outside `temper ctl`, since it hands the terminal to
     the program's own sign-in (`../../protocol/credentials-llm-accounts.md`, section 1).
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
     `domain/accounts-sources.md`, 1, has it, and is answered as a first
     declaration is.
   - **The task's model** is chosen with this area: `chat start` and area
     2's `goal start` take `--model MODEL [--effort EFFORT]`, the task's
     LLM choice, the template's choice or what each run's check takes without them (jig's
     `domain/tasks/README.md`, section 3), and area 2's `task amend` changes it.
     Both take `--sub-agent MODEL[:EFFORT]` too, repeatable, each one of
     the choice's sub-agents in the order given (jig's
     `protocol/configuration-templates.md`, 1), a model at most once, its effort its
     default where left out as `--model`'s is; given,
     they replace the template's sub-agents whole, and they need
     `--model`. Without them the choice keeps the template's sub-agents,
     or names none.
     Left out, the effort is the model's default in the offer; given, it
     names one of the model's efforts. An
     amendment replaces the main choice whole; its effort is
     the caller's, never inherited from the task's old main.
   - **A start's flags win over its template** for every run of the task
     (`../../draft/goal-start-flags.md`, 2). Given `--model`, a start's
     choice is the flags' main, the `--sub-agent` flags' sub-agents or,
     without them, the template's first choice's sub-agents, and no
     later choice: a template's `later` (`tools-acts.md`, 1) goes with the
     rest of its choice, so no run after the task's first activation
     takes the template's models in place of the flags'. Without
     `--model` the task takes the template's choice whole, its `later`
     with it. `help goal start` and `help chat start` say which wins: a
     flag wins over the template, whose choice holds only where no flag
     is given; and that a model flag a bound refuses is named, with
     nothing made.
   - **A model flag refused names itself:** its flag, its value as given
     and the bound that refused it, and nothing is made. Before anything
     is sent, every act taking these flags (`chat start`, `goal start`,
     `task amend`) checks the offer in `me`, the models the party may
     choose in the project as its authority allows them, and each
     model's declared efforts, refusing as usage, exit 1, with JSON's
     `error` the same words:

     | Refused | Words |
     |---|---|
     | `--model`'s model, not offered | `--model MODEL: not among the models you may choose in this project` |
     | `--effort`, not the model's | `--effort EFFORT: not an effort MODEL declares; it declares E1, E2` |
     | a `--sub-agent`'s model, not offered | `--sub-agent VALUE: MODEL is not among the models you may choose in this project` |
     | a `--sub-agent`'s effort, not its model's | `--sub-agent VALUE: EFFORT is not an effort MODEL declares; it declares E1, E2` |
     | a `--sub-agent`'s model named again | `--sub-agent VALUE: MODEL is already a sub-agent` |

     VALUE is the flag's whole value, its effort included; a model
     declaring no effort `it declares none`; the efforts in the model's
     order. Each name or value echoed is cut as a name in an error is,
     at 256 bytes or its first control character, `...` marking the cut.

     jig checks the rest when the start is asked: each model one the
     task's charter allows and within the task's authority (jig's
     `domain/tasks/README.md`, section 4), refusing the batch whole as
     `Llm`, naming the model and its fault. A start, `goal start` or
     `chat start`, prints that refusal, exit 2, as `TOOL: NAMED: WHY`.
     NAMED is `--model MODEL` where the call's `--model` named the
     refused model, `--sub-agent VALUE` where one of its `--sub-agent`
     flags did (`--model` first where both did), and otherwise the part
     of the choice that holds it, as `task show` names the parts: `main
     model MODEL` or `sub-agent model MODEL`. That is a model the
     template gave, or any model of a request restored after a client
     restart, whose kept ask holds the choice and not the call's flags.
     WHY is the fault's:

     | Fault | WHY |
     |---|---|
     | `Charter` | `not one of charter C's models`, C the task's charter |
     | `Authority` | `outside the authority the task is given` |
     | `Effort` | `its effort is not one the model declares` |
     | `Unconfigured` | `not a configured model` |
     | `Procedure` | `the task's executor runs no model` |
     | `Repeated` | `named twice among the sub-agents` |

     As `goal start: --model claude-sonnet-5-5: not one of charter 17's
     models`. JSON keeps the refusal's record and, where a flag named
     the model, adds `flag` (`--model` or `--sub-agent`) and `value`, as
     a refused account act adds `next`. Other acts keep jig's words for
     the refusal, `This model cannot be chosen: MODEL`.
   - **An account moved,** `account export` and `account import`, is
     jig's move (jig's `domain/accounts.md`, section 6) through
     `jig-client`'s `move_out`, `written` and `move_in` (jig's
     `client/domain/client/requests.md`): a fresh deployment takes a
     signed-in account in and gives it back, as a host with no browser
     takes a record signed in on a computer with one. The record is never printed, never an argument, and in no
     request or answer kept (`requests.md`, section 1). `task move` and `task take` are
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
        keeps (`requests.md`, section 1).
     3. Answered *moving out*, the record is written to FILE whole,
        FILE synced and then its directory, and only then is `written`
        sent with the move's number, an ordinary request, kept in
        DIRECTORY until answered (`command-line.md`, 1.2). Answered done, the tool prints
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
     `bounds.md`, section 1; refused, exit 1, nothing sent, the line naming FILE's
     fault and never its bytes. Its form version, the file's first two
     bytes, is read against the form this temper's jig reads: a newer
     form, `temper ctl: FILE holds a moved record of form vN; this temper
     reads up to vM: upgrade temper`, exit 1, nothing sent, FILE kept.
     Any other file is sent, and the engine refuses one it cannot read
     "not ours", as it does a form newer than the engine's jig reads
     when ctl's and the engine's jig differ (`README.md`, section 12). `move_in` carries the bytes once under
     `--key NAME`, never kept (`requests.md`, section 1). Answered *moved in*, the tool
     prints the account, then removes FILE, whose record is dead once
     the engine refreshes it; a removal that fails exits 1, `temper
     ctl: FILE not removed: WHY; its record is spent`, after the
     account's line. Every other end keeps FILE, the only holder of the
     record: a refusal, exit 2; *unsent*, *lost* or the link down,
     exit 1, and the agent asks again with the same `--key` and FILE,
     answered with the first answer within the engine's retention.

## 2. Open questions

- **Other account-act refusal reasons** (`areas.md`, area 5): a key's add or
  replace and a sign-in's start or finish can answer not offered for
  several core causes. Giving each its reason and next step needs those
  causes distinguished in jig's account refusals and client contract
  (jig's `client/domain/contract/README.md`, section 9). Default: their generic
  words; declaration's name taken and not offered, and retry's not
  offered, have the specific words of 5.5.

- **Model catalogue changes** (`areas.md`, area 5): `model list` reads the
  offer; it fetches no provider list and adds no model. Catalogue settings,
  adding by name and refreshing an account's list follow the store and list
  steps (`../../draft/model-catalogue.md`, section 7).

Each is named here and designed when it comes.

- **An older moved form patched at `account import`** (`areas.md`, area 5, the
  file's versions): jig's form patches, then skein's record patches,
  before the engine's checks, when a form 2 exists.

- **Naming a source** in area 5's `--source SOURCE`: by its kind, which
  names one source while an account holds one of each kind. Default:
  by kind, and by a program source's name once an account may hold two
  of a kind.

- **An export's `written` answered to a later call** (`areas.md`, area 5): when
  `account export` ends before `written` is answered, the next call
  that sends it reports its answer as any kept request's, and nothing
  removes FILE when it is refused "cancelled", though FILE's record is
  then dead (an import of it makes an account needing sign-in at its
  first refresh). Default: so; the
  alternative keeps FILE's path with the kept request, a stored format.
  Until then the agent tells which by `account list`, which sends the
  kept `written` first: the account's `sign_in` source gone (or the
  account), FILE is its only holder; the source still listed, signed in
  or moving, the move was cancelled and FILE's record is dead.

- **Why a moved record is "not ours"** (`areas.md`, area 5): jig's refusal
  carries no reason, so a form newer than the engine's jig reads, another
  application's record and another registration's read alike. Default:
  `account import`'s own check of the form (`areas.md`) covers ctl's jig; a
  reason on jig's refusal (form, application, registration) is jig's,
  when a form 2 exists.
