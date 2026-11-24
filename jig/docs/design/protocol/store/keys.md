# The store: keys

## 1. Keys

### 1.1 The encoding

- **Bytes sort as the domain's keys do.** Every key is written with
  skein-kv's key writer (skein's `data/kv.md`, 2.1) so that byte order equals
  the order Rust derives for it: a variant's index as a tag, then its
  fields in order; numbers big-endian at fixed width, narrower ones
  widened; an absent option before a present one; a byte string escaped
  and terminated, so a shorter one sorts before its extensions; a list of
  byte strings as each string marked, then an end mark.
- **Keys never hold payload:** numbers, small fixed identities, and
  names within their limits.
- **Written out, not generated.** skein's generator writes values, whose
  length-prefixed bytes do not sort (skein's `data/codec/README.md`, section 3), so
  each family's key has a small encoder and decoder of its own, beside
  its codec. A property test holds each encoder to the derived order on
  generated keys, and each decoder to the encoder.

### 1.2 Tags

The first byte of every key names its owner:

| Tag | Owner | Then |
|---|---|---|
| 0 | the component's own rows | its family: the format row (`records.md`, 3), transcripts' places (`transcripts.md`, 1), task logs' extents and places (`logs.md`, 1) |
| 1 | jig | the child's tag, in the order of the core's key: the core's own, then each child's; then the family's key; and one row of the component's, the installation's identifier, at its reserved key (1.4) |
| 2 | the application | its own key bytes, which it encodes by the same rule, so that bytes sort as its keys do |

- **The application's bytes stay under its tag.** The component prefixes
  them, and refuses an application's key or interval it would have to
  write outside it, which is a bug, as a contract break.
- **A child's keys are one interval,** and each family one interval
  within it, since the encoding follows the keys' nesting.

### 1.3 Ranges to intervals

The domain asks for ranges as predicates over its keys; the store reads
intervals. So:

- **each range maps to a short, fixed, ordered list of intervals,** in the
  component, and a page that exhausts one interval with room left goes on
  into the next in the same step, since skein-kv answers loads at once;
- **families loaded together are adjacent:** within each child, the
  families a restart loads come first, and history after them
  (`../../domain/engine/decisions.md`, 1.5), so each child's live part is one interval;
- **a test checks each mapping** against the range's predicate over
  generated keys: every key the predicate admits lies in an interval, and
  every key in an interval is admitted; and a restart's key's place
  (`loads.md`, 2), the domain's over its typed key and the component's over its
  bytes, names the interval that holds it, over a request of several
  ranges as over one.

### 1.4 jig's families

What the store keeps of jig, by owner. "At start" families are read by
the restart (`opening.md`), the others by a load, a read or a get when
wanted. The families' keys are the children's; the order within each
child puts live first.

| Owner | Family | Key | Loaded |
|---|---|---|---|
| component | the format row | none: one row | at open |
| component | the installation's identifier, at its reserved key in the accounts' family (01 04 03): drawn by the shell at each start (`../configuration.md`, section 7) and kept by the store's first commit, beside the format row (`records.md`, 3), so that each later start's draw is ignored | none: one row | at open, handed up as the store opens, never by the restart's ranges (`loads.md`, 2) |
| component | transcripts' places: each task's committed length and its current window's start; each attempt's first turn | task; task, attempt | at open, the run proofs' tasks' lengths (`transcripts.md`, 4); the rest when read |
| component | the registry: each layout of the registered families' value types, the schema texts they name, the jobs done, and each value type's order of layouts (`layouts.md`, 3 and 5.6; `jobs.md`, 2) | family, value type, id; a text's digest; owner, job; family, value type | at open, whole |
| component | the floor: the oldest store the build's code serves and the build that wrote the row (`records.md`, 3), `00 0d`, named `component.floor` in the census (`growth.md`, 1.3) | none: one row | at open |
| core | the deployment: its id, its numbers (the next project's among them, past every project seeded or made), the last commit | none: one row | at start |
| core | settings (`../../domain/engine/settings-store.md`, 1): the defaults the newest revision was made with, a record per setting; the changes over them, a record per setting changed (its value whole, or its removal, with the revision that made it); the held revisions' records; the charter versions held revisions read and the newest does not; pending proposals' sets; of an application's kinds, each setting's state and size and each item's edit and size, never the value, which its connector's family keeps (`../../domain/engine/settings-application.md`, 1) | under the core's family tag 8: kind, then scope (0 for the deployment, a project's number), then number, for defaults and changes; revision; charter, then the revision that made it; proposal | at start, after the deployment's row, before every other family of the core's |
| core | settings' history: the records of revisions no longer held, each with its number, the revision it was made against, when, who decided it, the proposal it came from, whether it moved running work, and its items with the values they set and what each replaced (`../../domain/engine/settings.md`, 10), an application's with their sizes alone (`../../domain/engine/settings-application.md`, 1); within the history limit, the oldest erased by the decision that makes a newer one | under the core's family tag 9: revision | on demand |
| core | goals' projections (feed, plan rows, pending milestones, their words apart: `parts.md`, 3), run proofs, calls committed since a live task's last turn, live agent tasks' conversations (their first attempt, their current window's start) | a goal's, then its own; task; task, attempt, completion, position; task | at start |
| core | turns (in turns logs or transcripts' files), terminals, decisions on proposals and escalations, each a task's in its history log but a party's proposals' (`logs.md`, 2) | task, attempt, turn; task, attempt; proposer, proposal; task, revision | on demand |
| tasks | live tasks, each with its revision and its claim's (`../../domain/engine/settings.md`, 3), each in a hot and a cold part (`parts.md`, 2), stubs naming ended results, the funders' live periods, writer slots, pools' slots, pending proposals | task; task; funder; slot; proposal | at start |
| tasks | placed holds (`../../domain/tasks/lifecycle-holds.md`, 2): each one's owner, mode, scope, reason, what lifts it and its subscription, its revision and the tasks it exempts; a lifted one kept while a task it held is still held by it, erased with the last one's release | hold | at start, with the live tasks |
| tasks | ended tasks, past periods, decided proposals, history, attempts, holds and activity (`../../domain/tasks/lifecycle.md`, 12), each in its task's history log but past periods and decided proposals (`logs.md`, 2) | task, then a position, an attempt or a revision; funder, period; proposal | on demand |
| tasks | ended tasks' summaries, and retention's rows, one per ended task or decided person proposal naming its next stage (`retention.md`, 3 and 4) | task; stage, since, number | on demand, by retention's passes |
| people | parties, sign-ins (no secret), roles, projects' names, whether each is public (`../../domain/people/public-reading.md`, 2), and whole policies (their ceilings, escalation roles and overrides of templates among them), seeded and made alike, read positions over results and over live conversations, answered keys within the retention, a party's or, for the deployment's acts, a connector's (`../../domain/people/acts.md`, 3), including typed policy-ceiling refusals (holder, dimension and optional scalar bound and requested values), program declarations and checks, their declared names, and sign-out/removal answers naming unconfirmed configured workers with their original failure reasons; each project being deleted, with the step its deletion has reached and the grace it gives its conversations (`../../domain/people/acts.md`, 2), at most one per project the deployment has room for, erased with the deletion's last step; each deleted project's mark, its number alone, kept past its deletion, at most as many as the projects the deployment has room for, the lowest number's erased to make room (`../../domain/people/acts.md`, 2, *Marked deleted*) | as the people child keys them; a live conversation's position by party, then task; a deletion by its project; a mark by its project | at start |
| people | ended conversations' read positions (`loads.md`, 4) | task, then party | at start, by the census, every row (`loads.md`, 2); otherwise on demand |
| accounts | issued program names, including retired ones, with their source number and original holder and desired sign-out (key tag 4, after the installation's reserved tag); the next number the accounts child gives, below which it gives none again; LLM accounts: each one's holder, LLM provider, subject and display name, its uses, and what its last list reads gave, where it has any: the catalogue's numbers as a bit each, when its last good read was, when its last attempt ended, why that failed, and how many names took no number (`../../domain/accounts-lists.md`, 1); their sources, each at its place in its holder's order, with its kind, its name for a program's, and its hold: limited, with its reset as a wall time where it has one, refused, or moving (a signed-in source whose record is out with a move, or back and not yet refreshed: `../../domain/accounts.md`, section 6). The rest of a source's state is heard again at each start, and none is kept per host (`../../domain/accounts-sources.md`, 2); what each account's LLM provider says of it, a record per account of at most 24 facts (key tag 5, after the issued names; `../../domain/accounts-lists.md`, 1) | source for an issued name; none: one row; account; account, source; account for its facts | at start, issued names before live account/source records, an account's facts after them |
| notes | entries, and an index line per entry, under its scope | name; scope, name | when a scope is in use; entries on demand |
| tasks | the results' index, the inbox's census's (`loads.md`, 2): a row per result of a task a party requested, at its position among that party's results, past the tasks child's live families | party, position | at start, by the census, past each party's read position over results |
| tasks, people | indexes for parties' reads (`loads.md`, 4): each party's inbox by time; ended tasks by requester party and by project; ended delegates; a subtree's delegates | the index's own, newest first where it reads so: an ended index's by party or project, then label, ended at reversed and task | on demand |

Children that keep nothing durable (the views, the fleet, authority,
briefs) have no family. What a run reports as it goes (its text, its
activity) is not stored: each committed turn holds what the run said
and did. Nor is a program's login on a host, which its host reports
(`../llm.md`, 5.5), or a sign-in under way, which a restart ends
(`../../domain/accounts.md`, section 5). An account's values are in the
secrets' store (`secrets.md`).
