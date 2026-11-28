# The engine's settings: in the store and at a start

## 1. In the store, and at a start

- **What the store keeps** (the core's settings family,
  `../../protocol/store/keys.md`, 1.4): the defaults the newest revision was
  made with, setting by setting; the changes, each a setting's value whole or its
  removal, with the revision that made it, and of an application's
  kind its state and size alone, its value the connector's (5); a
  record per revision, with
  its number, the revision it was made against, when, who decided it (a
  party, the release, the first start, a provider's list), the proposal it came from,
  whether it moved running work, and its items with the values they
  set and what each replaced (10), an application's items with
  their sizes alone; each charter
  version a held revision reads and the newest does
  not; and pending proposals' sets. The tasks family keeps each
  task's revision, live and ended, and each claim's.
- **History** is the revision records, kept for every revision held
  and for the latest ones within the history limit (8); the decision
  that makes a revision, or that ends the last holder of one, erases
  those that fall out. A version is restored by a proposal of the
  settings as it left them (10). History is read from
  the store, a revision by its number and the records a page at a
  time, newest first, each with what it changed; a revision erased is
  answered as forgotten, naming the oldest kept
  (`../../client/domain/contract/reads.md`, section 1). The store keeps the
  history's extent, the oldest revision it holds, among the live
  settings. Each commit that makes a revision, and a start whose history
  limit was lowered, erases from the oldest up to the limit's cutoff,
  within the settings commit's write budget, and records the new
  oldest. A revision below the cutoff is answered as forgotten even while
  a row that commit had no room for remains; later revisions' commits
  erase the rest.
- **Settings writes** are checked at configuration against the store's
  commit edge (`../../protocol/store/records.md`, 4).
  `jig_domain_core::settings::writes_bound` bounds the first start's or
  a revision's writes; it must fit `JournalSizes.writes`, which supplies
  the store's `commits.writes`. An overflow or a bound past that edge
  refuses the configuration.
- **A start** reads the settings right after the deployment's
  numbers, before any other record of the core's (`restart.md`, step 1):
  the stored defaults, the changes, the held revisions' records and the
  charter versions. It then compares the
  defaults it is given with those stored:
  - *a first start* makes revision 1 from its defaults, recorded as the
    first start's;
  - *defaults that differ* are stored, and when they change the current
    settings (a setting no change is over), they make a revision
    recorded as the release's, with no proposal, once the core's
    records are restored and every connector has said which of its
    kinds' defaults differ (`restart.md`, steps 1 and 2; 5), with the
    connectors' new defaults saved in it; like any revision, it
    withdraws stale proposals of settings (6); new work takes it,
    running work keeps its own. A model a change is over stays as that
    change left it, so one a person turned off stays off, and a release's
    new model comes in at its default. An overlay masks nothing: a
    charter's default that differs under its overlay changes the
    current settings, the overlay laid over the new default
    (`overlays.md`, 4);
  - *a release's new model a deployment already holds by name*
    (`settings-catalogue.md`, 1): a default model the stored defaults
    lack, whose name a stored change gives a model at any number, a
    withdrawn one's too, is left out of the defaults the start is given,
    before every check below. The deployment's model keeps its number
    and its value, the tasks, templates and charters that name it go on
    naming it, and the default's number stays unused there while the
    name is held, since a catalogue holds a name once. A later start
    leaves it out the same way, so nothing of it is stored;
  - *the numbers first* (1): every default of a charter, template,
    policy or model out of its range, and every stored change of one
    past 64, refuses the
    start in one refusal naming them all, ascending by kind and number,
    each a default or a change, before the whole check, so that an
    owner moves them all at once (`upgrade --check` rehearses the start
    and names them before a redeploy). The core's `start::Failure`
    carries it as `Numbers { settings: Box<[Numbered]> }`, each
    `Numbered { setting: Setting, changed: bool }`, `changed` false
    for a default, the list no longer than the defaults and changes the
    start holds;
  - *then a release's default on a kept number* (1): a default the start
    is given that the stored defaults lack, at the number of a stored
    change that sets a value (a withdrawn model's too), refuses the start
    in one refusal naming them all, ascending by kind and number, before
    the whole check. Laid under that change, the release's item would be
    masked by the deployment's own, unseen. The core's `start::Failure`
    carries it as `Taken { settings: Box<[Setting]> }`; its words name
    each and both ranges, and say to move each to 33 to 64 on the build
    that runs, then start the new one; `upgrade --check` names it before
    a redeploy;
  - *kept items are warned of* (1): every stored change setting a value
    at 1 to 32 that no default the start is given has is kept where it
    is and checked whole as any setting is. The core names them,
    ascending by kind and number, through `Core::settings_kept(&self) ->
    Box<[Setting]>` (`check::kept(&Current)` in its vocabulary), and the
    application warns of them in one line once the start has ended,
    naming each and both ranges and saying that a release whose stock
    brings one of their numbers refuses its start;
  - *the current settings are checked whole* (1): a start whose
    settings fail is refused, naming the setting, the change over it if
    any, and what fails, as a document failing its check always was;
  - *past limits:* held revisions or charter versions past limits
    lowered since, or a release's revision that would pass them, refuse
    the start, saying which, as a live set past lowered limits does. A
    task or a claim naming a revision the store holds no record of is a
    link that cannot be made, and refuses the start (`restart.md`).

  A saved project policy above the current policy byte budget refuses the
  start naming its project and policy; the named restore failure is retained
  until the shell reports it.

  All of it, the release's revision included, is committed before the
  core admits anything.

  The release's withdrawals are in its decision (6), so a crash
  leaves the revision and its withdrawals both or neither: a start after
  the commit finds the revision made and the proposals withdrawn, and
  makes no second one; a start after a commit that was lost makes them
  both. They run at `Phase::Link`, with every link made. The decision
  holds their records: the tasks child's, the core's set erased, and a
  connector's dropping of an application's values, a settings ask like
  the release's others. The people child's projections of them follow
  in memory: it takes a projection event once its census is whole, at
  its link and again once its start has ended, never while loading or
  sweeping, and a restart rebuilds them from the records. What acts on
  them waits, by `restart.md`: a proposer left due is activated when the
  tasks child is told it is restored, and a word left
  for a live run is relayed after that run's next turn, as any restored
  task's are. Nothing is activated or relayed from the decision itself,
  so a first start and a second after a crash do the same.
- **The document's members as defaults.** An application that passes
  no defaults gives the document's `llm` models, `charters`, `templates`, `policies`
  and `rules` (`../../protocol/configuration-engine.md`, section 1), and the
  store's changes lie over them as over any defaults. A deployment that
  restarts with the same document and no change behaves as it did; a
  document edited between starts gives the release's new defaults
  (above); a stored change wins over the document's member; a reset
  returns the setting to the document's value.
