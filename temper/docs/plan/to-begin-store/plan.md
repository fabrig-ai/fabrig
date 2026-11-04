# to-begin-store: the plan

The storage behind "Skip" (`temper/docs/design/draft/to-begin.md`, 5, the draft's step 2). Temper
alone: neither jig nor skein changes. No "To begin" view and no "Skip" button: the read reaches a
client's facts and stops there.

## The design, written first

- `temper/docs/design/protocol/store.md`, 2, 3 and 7: `Dismissed`, byte 34.
- `temper/docs/design/client/README.md`, 4.6 and 8: `Dismiss`, the subject `dismissed`.
- `temper/docs/design/domain/forge.md`, 5.5, new: the connector's part.
- `temper/docs/design/client/forge.md`, 9; `temper/docs/design/protocol/configuration.md`, 7.
- Their status files, with **Not yet**; `temper/docs/design/draft/to-begin.status.md`: section 5's
  two **Design lags** closed, one **Design departs** added.

## Parts

- **store:** the family, restored and held by the forge connector.
- **wire:** the request, its outcome, the subject and its document.
- **engine:** the request decided, the subject served, the rows erased with their project.
- **client:** the client's facts, and the story that joins both real sides.

## Tasks

| Task | Part | Needs | What | Minutes |
|---|---|---|---|---|
| `dismissed-family` | store | | the row, the key, the record, the restore | 50 |
| `dismiss-wire` | wire | | the schema's lines, the codec's functions | 30 |
| `dismiss-request` | engine | `dismissed-family`, `dismiss-wire` | the decision, the answer, the release | 50 |
| `dismissed-read` | engine | `dismiss-request` | the document, the watch, the start's check | 35 |
| `dismissed-facts` | client | `dismiss-wire` | the watch, the facts, the request's form | 35 |
| `dismissed-story` | client | `dismissed-read`, `dismissed-facts` | the meeting point, in the client's world | 40 |

## Critical path

`dismissed-family`, `dismiss-request`, `dismissed-read`, `dismissed-story`: about 175 minutes of
implementer time. `dismiss-wire` and `dismissed-facts` run beside it.

## What the cut decided, where the draft leaves it open

- **Byte 34:** 30 to 33 are named by the design for families not built yet (`LandingLog`, `Finding`,
  `Mark`, `QaPass`), so the next free byte is 34, on `origin/main` and on every integration trunk at
  this batch. A variant's place in `Key` is its order, and 34 sorts after them all.
- **Erased with its project alone,** where the draft says "or its party": jig erases no party, so
  there is nothing to follow. **Design departs**, and the plan's `[[question]]` `party-erasure`.
- **A role lost leaves the row:** the draft does not say. It holds its project still, and the bound
  allows for it. A person given the role back finds "To begin" still skipped.
- **The bound is no new limit:** `bindings` times `judge_projects`, the draft's members times the
  projects, so no override, no help page, no fixture changes. Reached, which only members coming and
  going can do, `Dismiss` is answered `too large`, the forge reason a client knows.
- **The project comes from `what`:** the root names it as the scope, so a client cannot claim one
  project's role for another's row.
- **`Dismiss` is the root's `Application`, not a `forge::Ask`:** the forge's `Ask` carries no party,
  and the core's admission does.
- **The answer is memory, as a set up's:** the row is what lasts. The same key sent again after a
  restart is answered unknown, a fresh one dismissed; a client reads the subject either way.
- **The read is a subject of the party's own scope,** temper's first: one subject, each watcher's
  document made for its party (jig's `Ask::Document` names it), so a dismissal costs each signed-in
  client one small document, whoever dismissed. Chosen over a subject per party, which would make the
  root read the sign-in on every watch.
- **`Dismissals` holds at most 1024 on the wire,** one row a project; a start past it is refused.
- **A client's `ask` has no caller but the story's test** until "Skip" lands: the request's form sits
  with its facts.

## What is unclear in the draft

- **Its order (6):** step 1, the view, comes first in the draft; the user started step 2 alone. So
  this plan builds a request nothing sends yet.
- **The draft moved beside this cut:** `65866c395a` on main ("To begin plainly optional") changes
  sections 1 and 2 alone; section 5 stands, and "Skip" stays one request.
- **"Any party" (5):** a service with a role may dismiss; it has no Home, and its rows do no harm.

## Overlaps with other drafts

- **None in jig or skein.**
- **Could not be avoided:** `temper-service/src/family.rs`, `temper-engine-domain/src/**` and
  `temper-engine-domain-forge/src/**` lie under `debt-11`'s `map-busy` and `temper-clippy`, and
  `temper-service/src/**` beside `debt-10`'s `temper-names`; `temper-client-domain/src/forge.rs` and
  `route.rs` under `map-busy`; `temper/docs/design/client/README.status.md` beside `look-round-3`'s
  `account`. Each is a few lines added, and new work sits in new files (`dismissed.rs`). `look-round-3`'s
  `pages/`, `frame.rs` and `listed.rs` and `macos-cheap`'s files are left alone.
