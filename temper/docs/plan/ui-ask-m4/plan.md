# ui-ask-m4: plan

Milestone M4 of `temper/docs/design/draft/ask-temper.execution.md`,
*Remembering*: `temper/docs/design/draft/ask-temper.md`, 5.4 and section 7,
with the Remembering part of section 8 (`inputs/request.md`): jig's party
scope for notes first, then `remember`, the person's list in each Ask
temper brief, withheld from the project's other roles, and the page "What
temper remembers about you" in the person's menu.

The cut wrote the design first:

- jig's `jig/docs/design/domain/engine.md`, 10.1, *A party's scope*: the
  scope `Party { party }`, its entries preferences by their scope, bounded
  by `notes.party_entries` and `notes.party_bytes`; written by the runs that
  explain to their person (the asking family, the task's requester a
  person) through the application's named actions, `Note` and the new
  `NoteDelete`; read whole in their brief's new `preferences` section; read,
  corrected and deleted through a client by the party alone; and a turn's
  private parts, that section and the application's private tools' calls,
  withheld from every other party. Section 9 names the section, 7.3 marks a
  run's read as its person withheld, section 16 keeps the wider scope as a
  question. `jig/docs/design/client/domain/contract.md`, sections 6 and 7.3;
  `jig/docs/design/domain/authority.md`, section 3.
- temper's `temper/docs/design/domain/agent.md`, 8.4.4, *`remember`*, with
  8.4's charter, instructions and tests brought along, and section 10;
  `temper/docs/design/protocol/configuration.md`, 13.4's charter 11.
- temper's `temper/docs/design/client/README.md`, 5.17, *What temper
  remembers about you*, with 3.5's page row and Sign out row and 5.3's
  person's menu.
- Their statuses' **Not yet** entries, each deleted by the task that builds
  it; the drafts' statuses: `jig/docs/design/draft/notes-memory.status.md`'s
  lags narrowed to what the design still lacks, and
  `ask-temper.status.md`'s 5.4 and 7 lag gone; the execution's M4 row says
  cut.

## Decided

- **Privacy by projection, not by a second record.** A stored turn stays
  whole, for resume and for its person. The core marks a turn withheld when
  its reader is not the person who requested its task, and for every run's
  read as its person, since what `see` reads joins a transcript others read;
  the protocol layer that carries the turn projects it
  (`jig_protocol_transcript::withhold`): the section under the private
  title, and each private tool's call and result, become one fixed line. A
  live call of a private tool carries its input to the requester alone.
  The application names the title and its tools in the core's and the
  clients' settings: temper's "Kept about this person" and `remember`.
- **The asking family is the party scope's key.** No authority bit: a run
  writes and reads its requester's scope where its task's requester is a
  person and its authority holds the asking family, which only charter 11
  holds, so "read by this charter's runs alone" holds by construction, and
  no policy or ceiling changes.
- **A line is an entry's description, its body empty;** its place in name
  order is its number, in the brief, in `remember`'s answers (which give the
  whole list, so numbers stay fresh) and on the page. The date and the chat
  are the entry's `changed` and its first reference.
- **jig holds the bounds,** 12 entries and 120 bytes in both profiles, so a
  correction on the page is held as a call is.
- **`ask.txt`'s bound is the draft's 10 KB.** The stock test's 8,192 was a
  literal with no limit behind it (this cut's survey); `ask-temper.md`, 4.1
  gives the instructions 10 KB. The first request stays within 16,384.
- **The phone reaches the page** from the `Accounts` tab's last row, where
  it signs out, since a phone has no person's menu.
- **No fuzzy test** is added: the request asks for focused stories; jig's
  notes world's simulation is not widened.

## For the user's review before landing

- **The Remembering part of `ask.txt`** is written word for word from
  `ask-temper.md`, section 8, by task `ask-remembering`; it is yours to
  review before landing, as the request says.

## Parts and tasks

The first batch, released with this plan:

- **scope** (jig): `party-scope`, high, 240, no needs: the scope through
  every type, schema, golden and wire, its two bounds, every path refusing
  it.
- **withhold** (jig): `withhold`, high, 120, no needs: the projection as a
  pure function of `jig-protocol-transcript`, table-tested.
- **remember** (temper): `remember-surface`, high, 150, no needs: the
  declaration, its decode, its answer types and words, the route's stub;
  `ask-remembering`, medium, 45, needs `remember-surface`: the
  instructions' Remembering part and the 10 KB bound.

The second batch:

- **core** (jig): `core-surface`, high, 240, needs `party-scope` and
  `withhold`: the named action `NoteDelete`, the call part, the
  `preferences` section's kind and budget, the private settings, the
  withheld carriers and their projection on the clients' wire, four stub
  modules wired at their call sites, so that the party tasks share no file
  but a call site's line.
- **route** (temper): `remember-route`, high, 200, needs `remember-surface`
  and `core-surface`: the root's load, checks and writes, its answer the
  whole list; `see-withheld`, high, 120, needs `core-surface`: temper's
  private names in the core's and the clients' settings, `see`'s
  projection.
- **page** (temper's client): `page-domain`, high, 180, needs
  `party-scope`: `Address::Remembered`, the page's read and acts.

The third batch:

- **party** (jig, beside one another in `jig-domain-core`, each in its own
  module): `party-runs`, high, 180: a run's write and drop of its person's
  scope, the notes child's bounds; `party-reads`, high, 150: the person's
  own read, correction and deletion through a client, nobody else's;
  `party-brief`, high, 160: the `preferences` section whole; `withheld`,
  high, 180: the core's withheld marks and the live call's input to the
  requester alone. Each needs `core-surface` alone.
- **page:** `page-view`, high, 200, needs `page-domain`: the page, the
  person's menu item, the Accounts row's link, words and styles, the
  journeys' prints if stale; `page-gallery`, medium, 120, needs
  `page-view`: the five states by day and by night.
- **stories:** `remember-stories`, high, 210, needs the four party tasks,
  `remember-route`, `see-withheld`, `ask-remembering` and `page-domain`:
  the engine domain world's three stories (a line kept and read in the
  next brief and deleted by its person; nobody else reads it; a turn
  withheld from another role) and the joined client world's page story.

Slice 1 holds scope, withhold and remember; slice 2 core, route and page;
slice 3 party and stories. No walk: M4's walk (the execution's) follows
landing.

## Critical path

`party-scope` (240), `core-surface` (240), `remember-route` (200, the
longest of the six that need `core-surface`), `remember-stories` (210):
about 890 implementer-minutes. The page's chain, `page-domain`,
`page-view` and `page-gallery` (500 after `party-scope`), and
`remember-surface` with `ask-remembering` (195) run beside it.

## Who hears first

For jig's owner, shared work: `jig_domain_notes::Scope::Party` and its
mirrors in the store, the people child, the client contract and wire and
the clients' cursor (the store's notes and people families and the wire's
v19 schema changed in place); `notes.party_entries` and
`notes.party_bytes`; `NamedAction::NoteDelete` and `CallPart::NoteDeleted`
(the store's core family); `brief::Core::Preferences` and
`CoreBriefBudgets.preferences`; `Settings.private_tools` in the core and
`Settings.private` in the clients' component; `reads::PageTurn::Withheld`
and `reads::EndedValue::WithheldTurn`; `jig_protocol_transcript::withhold`.
Every root that matches these exhaustively gains an arm, jig's test kits
and temper's root among them.

For the TUI's owner: `temper-client-domain` gains `Address::Remembered` and
`pages::remembered`; the TUI's address match gains its arm, falling back to
the launchpad. No other change to `temper-client-domain`.

For the owner of coding runs: `temper-engine-domain` gains
`Tool::Remember`, `Remembered`, `route/remember.rs` and the arms of the new
call part and scope (`store.rs`, `route.rs`); `forge_route.rs` gains only
the call part's forced arm. `temper-engine-domain-forge` is not changed.

## Overlaps

`tui-box-polish`, beside this draft when it was cut, has landed; its
`temper-client-tui/src/view.rs` is on main before `page-domain`'s one-line
arm there. No other draft beside this one names this plan's files.

**One writer per crate, departed from for `jig-domain-core`:** the four
party tasks run beside one another in it, each in a module `core-surface`
makes and wires (`notes/contracts.md`, section 2), because serialising them
would put about 600 more minutes on the critical path; the files they
share are `core-surface`'s alone, and each may add a call site's line.
