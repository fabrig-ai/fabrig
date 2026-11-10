# Helpers

- **Sugar over the public client.** jig's helpers are modules of
  `jig-client`, built only from plumbing (the API's operations, subjects
  and reads, ../contract/README.md) and from the client's public surface, as an
  application's own code is. A helper an application ignores costs it
  nothing, and any application may write its own instead.
- **`shapes`:** functions composing plumbing requests from templates,
  the application's data that `me` serves (../contract/requests.md, 6;
  `../../../protocol/configuration.md`, 3.1):
  - `shapes::chat(me, project, words)` fills the chat template into a
    `Create`, and `shapes::chat_from(me, project, template, words)`
    fills a chat's template named by its number the same way, for an
    application offering more than one (temper's Ask temper,
    template 5): a template `me` does not offer the project, or one
    whose flags make no conversation (../contract/requests.md, 2), gives
    `NotOffered::Template`;
  - `shapes::close_chat(task, disposals)` gives one compound request:
    each live delegate cancelled or moved to the party, then the chat
    cancelled with the grace of the chat template `me` offers now, or
    with none once a change of settings removed it: closing is an act,
    and reads the newest settings as every act does
    (`../../../domain/model.md`, section 13), so a chat can always be closed;
  - `shapes::remove_project(me, project)` gives a project's
    `RemoveProject` with the same grace, the chat template's as `me`
    offers it now, or none once a change of settings removed it: the
    grace each of the project's live conversations is asked to finish
    within as its deletion closes it (`../../../domain/people/acts.md`, 2);
    a project not in `me` gives `NotOffered::Project`, and nothing is
    sent;
  - `shapes::set_goal(..)` fills the stock goal template;
    `shapes::set_goal_from(me, project, template, draft, reason)` fills
    an offered template by number; `shapes::set_agreed_goal_from`, the same with
    its specification's `asks`, a task its requester agrees before it starts
    changing. A task keeps the creating bound's note scopes, a goal's as a chat's. Each compares the filled draft
    with the project's creating bounds in `me` (../contract/documents.md, 3), a
    deadline given as a span within both, for the engine to judge, and
    gives one of four: *made,* a `Create`, within the first bound;
    *proposable,* a `Propose` to the roles the offer names, between the
    two bounds where the offer says so; *refused,* the offer's refusal,
    between them where it says so; or *not offered,* past the second
    bound or with an executor not listed. Refused and not offered send
    nothing;
  - each fills what a template does not give from `me` and its own
    inputs: the authority `me` offers for creating (../contract/documents.md, 3),
    one level below the role, made one level shallower again for each
    level of the template's `shallower` past the first, its spend the
    budget named (any for `AsAllowed`, whose amount the engine gives)
    and its deadline the task's: the template's span sent as a span,
    which the engine makes a wall (../contract/requests.md, 2), or the creator's
    wall where the template has it given, so that no shape reads a
    clock; no wake policy, so the executor's own;
    the template's LLM choice, or the goal draft's where it names one,
    by configured names (../contract/requests.md, 2), none being the charter's default
    taken at each run's check (`../../../domain/accounts.md`, 4.3). Shapes copy
    configured intent, never `resolved_default`, the separate preview served
    in `me`;
    no parameters, inputs or dependencies; symbolic resources copied
    from the template for the engine to resolve;
  - a template it cannot fill gives `NotOffered::Template`, and nothing
    is sent: a contract no `Create` names (an approval, or a verdict
    whose choices its document cut), a value its helper takes no input
    for (a chat's executor, budget or deadline the creator's to give),
    or a `shallower` of 0, which no task's authority may be.
- **`facts`:** pure functions over the documents the client holds, the
  same for an application's own presentation as for the reference UI:
  - *a chat's status,* from its chip's phase and its host's last report,
    what it waits on, and `me`, for the chat's project:

    | Status | The chat is |
    |---|---|
    | starting | waiting to start, due, preparing or claimed; or its host preparing or starting its run |
    | waiting for its run | due or preparing, and its run can't start yet: for an account, a credential or a host (1) |
    | writing | running, writing |
    | a tool | running a tool its host names: which, and since when |
    | working | running, neither writing nor in a tool its host names, or nothing reported yet; and since when |
    | waiting for you | waiting on the signed-in party: a question, a proposal or a person task in its tree; and for what |
    | waiting for them | waiting on another party: whom |
    | parked | idle: its run waiting, holding its slot, or parked |
    | retrying | backing off: until when, and why |
    | held | held: why |
    | closing; ended | closing; then ended, and when |

    The first that applies, in this order: held, closing and ended, each
    a phase that says why it waits; then a party it waits on, in any
    active stage; then its stage, and, while its run is claimed or
    running, what its host last reported. Writing and a tool are read
    only from such a report, never from a run being placed
    (`../../../domain/hosts/hub.md`, 5). A run that waits holds its slot only
    until it parks, and is idle meanwhile (`../../../domain/engine/executors.md`,
    4). The party it waits on is the signed-in one where it names
    them, or a role `me` says they hold in the chat's project; any other
    is another party. The helper reads whatever chip it is given and
    never aborts on one: a record missing what its phase names is the
    link's to refuse (`../../../protocol/client/wire.md`, section 1), and a chip
    built past the link with no end time ends at its phase's `since`,
    and one backing off with no notice says until when alone. How each
    status is said is the presentation's (the reference UI's: ../pages.md,
    section 4);
  - *an act's offer and effect:* the engine's admission of the act, as
    the document offers it (made, proposable and to whom, bounded, or
    refused; ../contract/documents.md, 3), and what a decision will do, where the
    act is offered with it (../contract/documents.md, 4); read, never inferred;
  - *a refusal and why:* an offer's refusal, with its facts
    (../contract/README.md, section 3); a request's comes with them in its answer
    (`requests.md`);
  - *whether an act is dangerous:* a cancel, a proposal's rejection and
    a note's deletion are, and no other act. A presentation that
    confirms marks a dangerous act so (the reference UI: ../objects.md,
    section 2);
  - *a decision's facts:* what a deciding act on an object would do, as
    data read from its document: whether the act is dangerous; its
    effect, where the act is offered with one; and, for accepting a
    proposal, what it needs and who funds it, with what the funder has
    left and will have after (../contract/documents.md, 4). jig's check holds a
    tree a person decided on to them (../testing.md, 6.2).
- **`thread`:** a conversation's thread (`conversations.md`).
- **`words`:** jig's English for its enums (phases, holds, refusals, what
  a task waits on) and for a decision's facts, as phrases with holes an
  application overrides phrase by phrase, built with skein-ui's
  phrasebook types, `jig-client`'s one use of skein-ui (`README.md`, section 3); the
  reference UI says them through skein-ui's phrasebook (../view.md,
  section 7), and jig's check says a decision's facts in them, with the
  application's overrides (../testing.md, 6.2).
  `held` says what a held task waits on when it awaits nothing else:
  `WaitsOn` names no hold, so a client that reads the task's hold chooses
  the word, and no `WaitsOn` maps to it.
  `an account` says what a task waits on when no account serves its next
  run and it awaits nothing else: `WaitsOn` names no run wait either, so a
  client that reads the task's `run_wait` chooses the word, and no
  `WaitsOn` maps to it.
- **What jig adopts.** Sugar starts in an application. jig adopts it
  when a second consumer uses it, or early when it is hard to interpret
  consistently (what "waiting for you" means, how committed turns and
  provisional text interleave), and once its shape has settled.
  Correctness (revisions, once, atomicity) is never sugar: it is
  plumbing.

## 1. Refused starts

`chat_status` also reads the task document's optional `run_wait`: a
chat due or preparing whose run wait is `Refused` is refused by its
host (`HostRefused`), with why. Its place in the order: held, closing
and ended first, as ever; then, in an active stage, a party it waits
on, and then a dependency it waits on, which is parked; then a refused
start; and only then its stage, so a chat backing off is retrying and
one claimed or running reads its host's report. A summary row, which
holds no run wait, passes none and is never refused. A refusal is not a
reply. The reference view shows the refusal's observed host kind, time
and cause; it promises no automatic source switch.

A chat due or preparing whose run wait is any other is waiting for its
run (`RunWaiting`, naming an account, a credential or a host), in the same
place in the order: *starting* says a run is about to start, never that
it can't. The exception is an account wait about to end by itself, the
account `Refreshing` or its source `Asking` or `Granted`, which stays
starting. A summary row, holding no run wait, says starting: the task's
document says why. A run waiting for an account has replied, as far as a
caller waiting for a reply goes, since nothing moves it but an act; one
waiting for a credential or a host has not.
