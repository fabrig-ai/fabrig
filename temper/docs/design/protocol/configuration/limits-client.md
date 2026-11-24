# Engine and client bounds

This document holds the engine and client bounds, split from [`limits.md`](limits.md).

## 1. Engine and client bounds

- **jig's limits stay in jig's parts:** `engine.limits` and each worker's
  `limits` (jig's `protocol/limits.md`).
- **What the root takes of `engine.limits`:** its core children's
  (`tasks.`, `authority.`, `people.`, `fleet.`, `accounts.`, `notes.`
  and `documents.`), with `core.templates`, `core.call_records`, `core.policy_bytes` and
  `core.call_answer_bytes`, after the local-agent composition above. The
  rest of the core's it gives the core itself (`../../domain/root.md`,
  section 9): from its own `root.` limits, in place of `brief.`,
  `core.brief_parts`, `core.brief_core_budgets.`, `core.resume_bytes`,
  `core.run_bytes`, `core.escalation_reason_bytes`,
  `core.load_slots` and `core.load_bytes`, a person's read of the store,
  from `root.loads.read_bytes`; as constants, in place of `views.`, but
  for its watch rooms (below), `core.fill_mark` and `core.connectors`. An
  override of one of these changes nothing the root holds, but for
  `core.resume_bytes`: the core's resume setting is still taken from it,
  above 0 and within `root.journal.transcript_bytes`.
- **The engine's documents fit temper's client.** A document past the
  bounds a client decodes is refused there whole, as too large (jig's
  `protocol/client/wire.md`, section 1), so the documents the engine serves
  must be within what temper's client takes: its client takes
  `jig-client`'s limits from jig's small profile with the same
  `documents.` overrides the owned example's `engine.limits` names
  (`../../client/README.md`, section 2; ../setup/init.md, 4). The engine cuts each
  text to its bound and says so (jig's `client/domain/contract/documents.md`,
  3). It checks none of this at start: it serves whatever client the
  deployment lets in, whose limits are the client's. A test holds the
  owned example's and the process tier's configuration to temper's
  client's limits, so the two cannot drift apart unseen.
- **The engine's watch rooms follow what it admits.** A watch past the
  room the engine allows a party, a stream or the deployment is refused
  busy, and each holds its place until the engine has ended it (jig's
  `client/domain/contract/watches.md`, 1.2; jig's `protocol/client/README.md`, section
  10). The deployment admits `clients.streams` clients at once, each
  holding up to its own watches, `me`'s included, and temper's shipped
  clients hold at most 31 (`../../domain/root.md`, section 9): the web's
  and the terminal's `watches`, 31, and `handles`, 33, so that Home on
  All projects holds four projects' statuses, `decided`, `follows` and findings
  documents with the session's own watches, 30 at most
  (`../../client/pages/home.md`, 1, *Home on All projects*). So:
  - *a stream's room* is 31, `WATCHES_PER_STREAM`, one shipped client's;
  - *the deployment's room* is `clients.streams` times that: 248 at
    `temper-box init`'s 8 streams (`stock.md`), 62 at the small profile's 2;
  - *a party's room* is the deployment's, since a person's tabs and their
    `temper ctl` calls are one party and may hold every stream.
    `views.watchers_per_party` never bites below `views.watchers` here:
    what one person can use up is the streams, and that refusal names
    them.

  `clients_http.streams` bounds the transport's side of those same
  streams and adds no room. The same test as for documents holds a
  stream's room to temper's client's watches, and a test of the limits
  holds the other two to `clients.streams`.
- **The rooms have no setting of their own.** More room is more streams,
  `clients.streams`, and the 31 is the shipped clients' bound. An
  `engine.limits.overrides` naming `views.watchers` or
  `views.watchers_per_party` is refused at the start, naming
  `clients.streams`, and `temper help limits-engine` lists neither: the
  root never took them, so a document naming them was told nothing.
  Those two names stay jig's, for an application whose root takes them
  (jig's `protocol/limits.md`). `views.`'s other names
  stay as above.
- **What a watch costs.** A watch holds a views slot, a name, a flight
  and its stream's entry, and shares a subject's slot with the other
  watches on that subject. It keeps no document between deliveries (jig's
  `domain/engine/views.md`), and these capacities allocate nothing
  until used, so a larger room costs nothing until it is held.
  - *Work:* an opening scans the watches held and builds its first
    document. Each commit's marks scan the watches held, and build one
    document for each watch whose subject changed, none shared between
    watches on one subject, with one delivery in flight per watch.
  - *Heap:* that pass holds each document it built until the document is
    delivered, so its peak is what those documents hold, each within the
    documents' bounds. No heap sum admits the root (`../../domain/root.md`,
    section 9), so no figure at the start moves. The engine's world
    measures a full room (`temper/tests/engine/domain/tests/suite/memory.rs`).
    With one or eight streams the empty root held 4,742 heap bytes; measured rooms
    of 17 and 136 watches held another 14,112–15,256 and 114,720–126,336 bytes after
    delivery, respectively, and one marking commit peaked at 116,816 and
    452,176 bytes in the engine world including its driver and fake store.
  - *The journal's door:* a full room puts at most 208 deliveries behind
    one commit, within `root.journal.deliveries`' 512. The watermark
    paces new work as the door fills, and nothing fails.
  - *The store:* its bounds don't follow the rooms. A watch holds no row,
    and an opening on a retained subject takes one of `root.loads.loads`,
    refused busy past them.
- **Past the room.** With the shipped clients, each stream's room fits
  the deployment's; watches still closing can overlap their replacements:
  a stream's predecessor holds its place until its terminals come (jig's
  `client/domain/client/watches.md`, 1). Each place is given back at its terminal,
  and a client refused busy opens again after a backoff. A refusal of `me`
  leaves its link offline; an application's watch can be behind with the
  link still live. Both set `watches_full` (jig's
  `client/domain/client/watches.md`, 2), and `temper ctl` says so at its deadline
  (`../../client/ctl/results.md`, 3). Busy can also mean an opening's load slot is
  unavailable; it carries no cause distinguishing that from a full room.
- **The forge's paths fit a document.** jig's documents carry a
  resource's path and a pattern whole, with as many segments as the
  engine admits in a name (jig's `client/domain/contract/documents.md`, 3), and
  `documents.lines` bounds a document's short lists alone, at most the
  client's rows. So `engine.limits` must name the bounds on a name's
  segments, `authority.segments`, `tasks.hold_segments` and
  `tasks.authority_segments`, at least the segments of the deepest path
  the forge names: a branch's, the forge's five segments then the
  branch's own (`../../domain/forge/README.md`, section 2), 8 as temper's branches
  name them. temper's client takes a path's segments, 9, apart from its
  rows (jig's `client/domain/client/README.md`, section 11), so a page of the
  forge's work is never refused for a path, and a long path shows cut,
  with jig's mark (`temper/docs/design/client/view.md`, 4).
- **The forge's authorities fit a document.** jig's documents carry an
  authority's lists whole, with as many entries as the engine admits in
  each, and neither cut nor refuse a document or an offer for one (jig's
  `client/domain/contract/documents.md`, 3), so a party who may amend a task
  reads its document whatever `documents.lines` is. temper's client
  takes an authority's entries, 16, apart from its rows (jig's
  `client/domain/client/README.md`, section 11): the most the process tier and
  the owned example admit in a list, their `authority.grants` and
  `tasks.authority_grants` (../setup/init.md, 4). A long authority shows
  in part, with jig's mark (`temper/docs/design/client/view.md`, 4). The test above holds
  the engine's bounds on an authority's lists to temper's client's.

## 2. Open questions

- **The real-browser check of watch sizing.** The domain story uses the
  pages' watch sets without HTTP, a browser or a kernel (`limits.md`).
  Checking Home at 1440 CSS pixels and a long chat at 420 while fresh tools
  clients read is a manual check after landing, as the `watch-room` request
  assigns it. Default: that check, with no additional browser-tier story.

- **A party's fair share of the watch rooms.** A party's room is the
  deployment's (`limits.md`), so one person with every stream leaves another
  none, and the refusal they meet is the streams'. Nothing has asked for
  less, and a smaller share would lock a person's own tabs out again.
  Default: none.

- **`views.`'s other names.** `views.runs`, `views.subjects_per_change`,
  `views.text_bytes` and `views.facts` are still taken and change nothing
  the root holds (`limits.md`), where `views.watchers` and
  `views.watchers_per_party` are refused. Nobody was misled by them on
  record. Default: they stay until one is.
