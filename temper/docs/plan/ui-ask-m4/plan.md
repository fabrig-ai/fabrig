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

Still to release (the plan's next batches):

- **core** (jig): `core-surface`, needs `party-scope` and `withhold`: the
  named action `NoteDelete`, the call part, the `preferences` section's
  kind and budget, the private settings, the withheld carriers and their
  translation, four stub modules wired at their call sites.
- **party** (jig, beside one another in `jig-domain-core`, each in its own
  module): `party-runs`, `party-reads`, `party-brief`, `withheld`, each
  needing `core-surface`.
- **route** (temper): `remember-route`, needs `remember-surface` and
  `core-surface`; `see-withheld`, needs `core-surface`.
- **page** (temper's client): `page-domain`, needs `party-scope`;
  `page-view`, needs `page-domain`; `page-gallery`, needs `page-view`.
- **stories** (the engine's domain world): `remember-stories`, needs the
  party tasks, `remember-route`, `see-withheld` and `ask-remembering`: a
  line kept in one Ask temper chat, read in the next one's brief, deleted
  by the person; another person reading none of it; a turn read by another
  role withheld and by its person whole.

## Critical path

`party-scope` (240), `core-surface` (about 200), the longest party task
(about 200), `remember-stories` (about 180): about 820
implementer-minutes. temper's page chain (`page-domain`, `page-view`,
`page-gallery`, about 450 after `party-scope`) runs beside it.

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

`tui-box-polish` (at its trunk) changes `temper-client-tui/src/view.rs`,
where `page-domain`'s forced arm for `Remembered` falls; the arm is one
line in the address match, away from that plan's hunks. No other draft
beside this one names this plan's files.
