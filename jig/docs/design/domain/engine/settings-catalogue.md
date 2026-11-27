# The engine's settings: the catalogue and provider lists

## 1. The catalogue

The current settings of kind `model`, in number order, are the
deployment's **catalogue**: what a task's LLM choice, an authority's
models and a charter name.

- **A number is a position.** Model `i` of a task's choice and bit `i` of
  an authority's models are the setting numbered `i + 1`
  (`../tasks/README.md`, section 3; `../authority.md`, section 3), so the store's and
  the wire's indices hold across a change and a release. The bound is
  `MODELS`, 64, for the deployment's life; a number no setting has is a
  hole, which nothing offers or resolves: a template's choice or an
  authority's bit naming one is refused as a number past the highest
  is. The binary's defaults take low
  numbers, 1 to 32, and keep them; a model a person or a provider's list
  adds takes the highest free number from 64 down to 33, a deployment's
  own range (1), so a release's new default never meets one. With none
  free there, an owner's add is refused as *count*, as a full catalogue
  is, and a list's names past it are left out (12); a number below 33 is
  never given to an added model.
- **A number keeps its name.** A set naming another name than its
  default's or its stored change's is out of shape; so are a dialect or
  program its endpoint doesn't have, a name out of `valid_name`, and a
  zero window, output or price unit. An unknown provider or endpoint is a
  reference fault. Nonzero window and output values no agent's bounds
  hold make a run wait, as a charter's does (`fleet.md`).
- **A model's speeds.** A model may declare the faster speeds its
  provider serves it at, as it declares its efforts, and name one of
  them its default speed (`speed`), which a run of it takes where
  nothing chooses another (`executors.md`, 1). One that declares none
  runs at its provider's standard speed alone. Each speed has its name,
  as the provider takes it and sends it (`fast`, `ultrafast`); its
  prices, the four of the model's (input, cached input, output, and
  cache writes where priced apart), in the model's price unit; and at
  most two other names (`also`) the provider may report it served
  under, as OpenAI reports Fast as `priority`. The model's own prices
  stay its standard speed's. Out of shape, beside the above:
  - a name, a speed's or an `also`, empty, past 32 bytes, or holding a
    byte but a lower-case letter, a digit, `-` or `_`; a name held twice
    in the model, across its speeds and their `also` names; `standard`
    or `default`, the names the dialects give their standard speed;
  - a fifth speed, or a third `also`;
  - a default speed the model doesn't declare;
  - a speed's price of zero, or below the model's own, a cache write
    compared as each is priced (its own rate, else its input rate),
    since a faster speed never costs less;
  - on the Anthropic dialect or Claude Code's program, a speed but
    `fast`; on another program's route, any speed, since that program
    chooses its own.

  A model's speeds count in its bound and in a charter's size with its
  other members, their names and prices (`settings.md`, 8). An owner's
  add by name and a provider's list give no speeds and no default speed
  (below; 2), whatever the model they take their figures from declares:
  an owner gives them by a set.
- **A model stays for the tasks that hold it.** A *removed* item keeps the
  setting, *withdrawn*: out of the offer (`views.md`, 1) and of every charter's
  resolution, with its number and name. A *reset* returns one with a
  default to it, and one with no default is out of shape. A run reads the
  charter version of its revision (3), whose records are the models as
  they were, so a task keeps the model it has, and no change of a model
  moves one.
- **A charter's models are names, resolved.** A charter names its first
  choices, in order, and whether **the rest** follow
  (`../../protocol/configuration-engine.md`, section 1). Its resolution is each
  name's record in the catalogue, at the effort the charter selects for
  it where it selects one (`../tasks/README.md`, section 3), and at the
  speed the charter names for it where it names one, `standard` for the
  provider's standard speed, else at the model's default speed, withdrawn ones dropped, the main the
  first left; then, with the rest, every other model in number order,
  while it stays within `settings.alternatives` and `run_bytes`. A
  resolution with no model is a *reference* fault. What a run reads, what
  the fleet needs and what the tasks child checks a choice against is the
  resolution; a setting's value, a read and a proposal keep the charter as
  set, so that a new model needs no charter's change. Each record holds
  the model's speeds with its other members, and as its speed the one
  just given, none for standard. A charter's speed for a model is one
  the model declares, by its own name, or `standard`, or none: a
  decision that leaves a charter naming another for one of its models,
  its overlay's included, is refused as a *reference* fault naming the
  charter, as the whole settings are checked at each decision
  (`settings.md`, 8), so that a resolution always finds it.
- **A change of a resolution is a change of its charter.** A decision or
  a release whose models change a charter's resolution makes that charter's
  next version at its revision, and holds the old one (3 and 8),
  though no item names the charter and its stored value stays. A change of
  a model's speeds or its default speed is one, for each charter whose
  resolution includes the model, and for no other. A change of
  a model's `picker` alone changes no resolution, and so no version.
- **What the charter left out:** a model past a charter's bounds stays
  in the catalogue and off that charter. `me` says how many, for a
  project's chats (`../../client/domain/contract/documents.md`, 1).
- **Who hears.** A decision or a start's restore that changes the
  catalogue asks the root to tell the clients' component and the store,
  before its commit (`Ask::Catalogue`; `../../protocol/store/records.md`).
  Both keep each model at its own number, whatever the gaps: the store's
  names are one per number up to the highest, a hole's name and efforts
  empty, as its names at open are, and the clients' copy is one row per
  number, so a stored index or bit is the number less one. The
  tasks child hears the efforts with the charters (2), the fleet its
  needs, and the views mark every `me`.
- **An owner's act.** `ChangeModel` (`../../client/domain/contract/requests.md`, 2)
  turns one model's picker on or off, or adds a model by name, a provider
  and a name, as one change at the newest revision, running work staying,
  decided as 2's. Adding checks the name's shape alone (`valid_name`),
  since only a first run shows whether the provider serves it. It takes the
  highest free number and the provider's dearest stock model's endpoint,
  window, output and prices, marked estimated, so that a budget stops
  early rather than late, with no efforts, no speeds and on the picker. A name the
  catalogue holds is out of shape, unless withdrawn, which it brings back.
  A provider with no stock model takes its `endpoint` and the catalogue's
  dearest stock model's figures.
- **A person's removal of a list's model** makes it a person's: the item keeps
  the setting withdrawn, with `source` person, so that a list naming it again
  doesn't bring it back (12). Adding it by name does.

## 2. A provider's list

A read of an account's list (../accounts-lists.md, 1) comes to the core as an answer.
The core alone turns it into settings, as the one decider that's neither a party,
the release nor a first start. It moves no running work.

- **Placing a read.** The core gives the fleet the account's grant and the
  endpoint its provider names (`../../protocol/configuration-engine.md`, section 1). The
  fleet holds it until a connected worker's bounds hold that endpoint, sends it to
  the one with the most free slots, one read in flight per worker, and holds it
  again if that worker's channel is lost before it answers. A renewal reaches a
  read still held. Host rule is not asked (../accounts-sources.md, 1): no tool runs on a
  read. With no worker connected, a read waits and nothing says it failed.
- **What it keeps.** Of the rows, in the provider's order: a name that a `keep`
  pattern matches and no `drop` pattern does, and that `valid_name` holds. The rest
  never reach the store.
- **A name new to the catalogue** takes `new_model` (11), `source` list, off the
  picker, and the highest free number, in the provider's order, each seeing the
  last. Its `first_read` is this read's time and never changes, so it's "new" once.
  Its figures are those of `new_model`, and where the list says them, its shown
  name, its window and output, no larger than `new_model`'s, since the binary's
  defaults bound what a run is given, whatever a provider allows, and its
  efforts, its default effort being `high` where it lists that, else its first.
  Its prices stay estimated.
- **The catalogue's room.** With no number free from 64 down to 33 (11),
  a name takes no number, and the account's `left_out` counts it, so the
  provider's first are kept.
- **A name gone from every list:** a model of this provider with `source` list
  that no account's last good read holds, this one's included, is withdrawn: out of
  the offer and every charter's resolution, its number and name kept for the tasks
  that hold it. A withdrawn one with `source` list that a read names again comes
  back. A read never touches a model of the binary's or a person's, nor another
  provider's, nor a kept row's picker, prices or figures; the accounts' names narrow
  those (../accounts-lists.md, 1).
- **What a read decides.** The new, gone and returned models are one change at
  the newest revision, `Running::Stays`, decided as 2's, by the provider's list
  (`Decider::List`, naming the provider, in the revision's record). A read that
  changes nothing makes no revision. Like any revision it withdraws stale proposals
  (6) and makes the versions of charters whose resolution it changes (11).
  Refused, for held versions past their limits or a whole set out of shape, it makes
  none, and the attempt fails as the catalogue's room.
- **One commit.** The account's record, the revision and its versions commit
  together, so that a crash leaves both or neither; `settings::writes_bound` prices
  the record's one write.
- **What it marks.** A read marks the `accounts` document of whoever holds the
  account, and the deployment's, since its times moved. `me` is marked only when
  what it says moved: the decision changed the current settings, or the account's
  `names` changed, which narrows its offer, or its displayed `left_out` changed
  (`../../client/domain/contract/documents.md`, 1). A day's alarm alone marks nothing, nor
  does a failed read, a repeat, a refused act or a read still held.
- **A failed read** changes no setting and no name (../accounts-lists.md, 1).
