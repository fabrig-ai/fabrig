# The client and the engine: answers

## 1. Answers

Every request has one answer, of one of these kinds:

- **done,** with its outcome: what was made (the first and the last of
  the tasks' numbers, consecutive in the batch's order, and the
  `AsAllowed` task, if any, with what it was given; a word's message
  number; a proposal's number and the role it was first admitted to, in the
  request's project; its present recipient is in its document. A replay keeps
  the original outcome's role; an account moved out, with its record, once: `requests.md`, 1), or what
  was decided; for a compound request, its
  operations made, in order, and the first of the message numbers they
  took, one each (`requests.md`, 4); and, with a retained decision visible to the party,
  who decided first, how and when. A task's automatically replaced settings
  proposal has no retained decision: deciding it answers `Unknown`, naming
  nothing and changing nothing (`../../../domain/tasks/proposals.md`).
  Accepting an ended settings proposal with a retained decision takes the
  running-work choice as a live acceptance does; that choice does not make
  a missing proposal malformed;
- **refused,** with its facts, and for a compound request the place of
  the first operation that failed:
  - *role:* the party's role here does not allow it;
  - *authority:* what was lacking, typed (a budget and how much is left,
    a grant, a kind of task, a depth, a deadline, tasks left, or a
    ceiling a policy edit passes: its holder, the deployment's rules, the
    project's policy or a role by number, the dimension (a spend per
    period, a spend, tools, delegation's tasks, depth or executors,
    grants, a deadline or notes), and, for a scalar, its value beside
    the edit's; a list is named without its contents,
    `../../../domain/authority.md`, section 6); and, where
    the request was judged against the party's role and pool and found
    beyond them (`../../../domain/authority-checks.md`, 1.4), whether a proposal of
    it could be accepted (`requests.md`, 5). A refusal that another check decided
    (the request's shape, a reference, an uncertain effect's choice, its
    funding, the project's spend per period, an `AsAllowed` that would
    give nothing) makes no claim either way. Who would accept a proposal
    is the offer's to say (`documents.md`, 3), and the proposal's document's once it
    is made, not the refusal's;
  - *no one to escalate to:* a party's root task whose escalations would
    have no holder (`requests.md`, 2);
  - *limit:* which one (the deployment's projects, the project's live
    tasks, its goals, the operations per request, a grace past its bound,
    the inbox full, a party's words too long, an amendment too long, a
    key, a name or a pasted address past its bytes, the LLM accounts'
    bounds: accounts in the deployment, accounts per holder, sources per
    account, sign-ins at once, program sources per worker; and those that
    are no number: a duplicate, an empty batch, a reserved or duplicate
    role, a resource held, a hold's kind, a record after ready, a counter
    exhausted) and, where a number was met, its bound: a configured one
    with its configuration key, or a fixed one, so that a client names
    it, prints the bound, and says to raise the key only where one can;
    a client's own limits are the client's to say;
  - *provider's rule:* an LLM provider's terms forbid it, naming the
    rule (a plan's uses naming another person, or a project with
    another party; another account for a task its account limits:
    `../../../domain/accounts-uses.md`, sections 1.2 and 1.3);
  - *account preference:* the account's number and why it cannot fund the
    prospective task: unknown, another provider, uses that do not serve the
    financial payer, or a different account while the kept account of that
    provider is limited. An otherwise valid unavailable account is accepted
    and waits; acceptance checks a proposal against its accepter's funding;
  - *moved:* the object's revision now, and what changed;
  - *ended:* the object has ended, and how;
  - *unknown:* nothing by that name the party may see;
  - *name taken:* another project holds the name a `MakeProject` or a
    rename gives, under simple case folding (`../../../domain/model.md`,
    6.1); it names no project, which the party may not see; or a live
    source holds the program source's name a declaration gives, or
    another holder's reservation does (`../../../domain/accounts-sources.md`, 1),
    naming neither;
  - *not offered:* the deployment does not offer it (an executor it does
    not have, a charter its newest revision removed; for an account's
    act, what `../../../domain/accounts.md`, section 5, says: a declaration
    to an LLM provider offering the holder no program's login, a resume
    of a source that is not limited);
  - *stale:* a change or a proposal of settings names a revision other
    than the newest, which it names (`../../../domain/engine/settings.md`, 2);
  - *judged:* an application's connector refused an item of its kind of
    settings, naming the item and with its reason, in its own words
    (`../../../domain/engine/settings-application.md`, 1);
  - *full:* the settings' revisions or charter versions held would pass
    their limits, naming the oldest revision held and its live tasks,
    cut at their limit, which a `MoveToNewest` may move
    (`../../../domain/engine/settings.md`, 8);
  - *circle:* the batch or the dependencies an amendment adds would close
    a circle of waits, with the live tasks or among its own: the circle's
    tasks, in order, each waiting for the next, cut at its limit
    (`../../../domain/tasks/holds.md`, 7);
  - *live tasks:* an owner's `RemoveProject` of a project with live work:
    the project's live roots, by number, lowest first, at most 32, and
    how many more (`../../../domain/people/acts.md`, 2), in jig's words
    "This project has live tasks; end them first:", each client listing
    the numbers after them;
  - *dependency:* a dependency a party's batch names or an amendment adds has failed or was
    cancelled, naming it and how it ended (`../../../domain/tasks/changes.md`
    ; `../../../domain/tasks/README.md`, section 4);
  - *concluded:* a `Delegate` beneath a gathering its requester has
    concluded, naming when (`../../../domain/tasks/gatherings.md`,
    section 3);
  - *kept:* a person task's answer while it is kept, naming its pending
    dependencies (`../../../domain/tasks/lifecycle-endings.md`, 1);
  - *not well formed:* which task or operation, and why (two tasks of a
    batch giving `AsAllowed`; the conversation flag on a task that is not
    an agent's; a `Conclude` of a task that is not a gathering, and a
    gathering made out of its shape, `EndedBy`
    (`../../../domain/tasks/gatherings.md`, section 2); a proposed batch giving `AsAllowed`; a project's name out
    of shape, saying which rule; a change of settings whose settings
    would fail their check, naming the setting and its fault, one of
    the check's closed list (`../../../domain/engine/settings.md`, 1), or that
    changes nothing). A client words each refusal of a change from
    these facts, checking no setting itself;
  - *sign-in redirect:* a finish refused, saying why: *not waiting*
    (no sign-in of the party's under that number waits for its
    redirect: none, another party's, one that has taken one, ended,
    or begun before the engine's restart), *address* (not the
    sign-in's redirect: another scheme, host, port or path, a
    fragment, a field twice, past its bounds, or without its code or
    issued client id), *state* (another sign-in's: the sign-in still
    waits), or *too many* (it has refused a few for their state, and
    takes no more pastes, still waiting for its browser: skein's
    `oauth/accounts.md`, 5; `../../../domain/accounts.md`, section 5);
  - *key conflict:* the key names another ask, which is a client's bug
    where the client drew the key (`requests.md`, 1);
  - *account move:* an account's move refused, saying why: *not signed
    in* (no signed-in source holding a record that reads), *not ours*
    (a moved file of another application, or of no configured LLM
    provider's registration), *unreadable* (a moved file that does not
    decode, of a form or version this build does not read, or damaged),
    *held* (its subject held by another
    holder), *cancelled* (a *written* for a move no longer under way:
    the client removes the file it wrote), or *moving* (the source is
    out with a move, or back from one and not yet refreshed:
    `../../../domain/accounts.md`, section 6);
- **busy** or **not ready:** nothing was made and nothing kept; send it
  again, under its key, after a backoff. A busy says why when it is the
  table of retained answers that is full (`../../../domain/people/acts.md`,
  1): *answers full*, with how many it holds and when its oldest
  answer expires, so a client says so and when it will have room, never
  only that the request was not sent;
- **no answer** (the exchange lost: its deadline passed, or the
  transport failed before an answer): the outcome is unknown, so the
  request is sent again under its key after the same backoff; its key
  makes a repeat safe, since a request already taken answers with what
  was retained, and a sign-in by link repeats until the link's own
  expiry ends it;
- **signed out:** the sign-in has ended; sign in, then send it again;
- **version not served:** the engine does not serve the version the
  request's owner is encoded in. Nothing was read, made or kept. It
  names the owner, its ranges keeping their bytes, which no client
  reads (`README.md`, section 6).

A client hears one more, which no engine sends: **no answer**, when the
connection failed before one came; whether the engine has the request is
unknown, and the key makes sending it again safe.

How the client's link hands each answer to its request table is
`../../../protocol/client/README.md`'s, section 3; a request answered version not
served is held, kept and unsent, for its owner's version (../client/watches.md,
2).
