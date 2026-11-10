# Where each way to start work is offered

Draft, 2026-11-04, from a session with the user that day, who asked for
it to keep the idea, not to carry it out: nothing here is adopted, no
plan takes it up until the user does, once the ways to start work are
clearer, and nothing here is a part of v0.1.0 (`v0.1.0.md`). A
project's ways to start work are its templates (jig's
`protocol/configuration.md`, 3.1), as How work runs names them
(`../client/pages/settings.md`, 2): jig's chat and goal and temper's
configuring chat in the stock (`../protocol/configuration/stock.md`),
and perhaps many more. Today the client chooses by number which it
offers where; this draft makes where a template is offered a setting.
The look requests `new-menu` and `configure-chat-in-settings`
(`../client/web/journeys.md`, 7) draw its two places.

## 1. In one page

- **An offer says where a template is offered:** its places, its order
  among them, and its line, a sentence the person reads beside its name
  (section 2).
- **Two places:** New, the menu beside the rail's "+ New chat", for the
  general ways; and Settings, where temper is configured. A template in
  no place is reached through How work runs alone (section 3).
- **Offers are settings,** temper's kinds 7 and 8, the deployment's
  and a project's, defaulted, proposed and decided as every setting is,
  so that the configuring agent proposes them too (section 4).
- **The stock offers keep today's:** the chat and the goal under New,
  the chat first, and the configuring chat under Settings (section 5).
- **The client draws them** from the offers, never from a template's
  number (section 6).

## 2. An offer

- **Its places,** a set of section 3's; empty, none.
- **Its order,** a number: the lower first in each of its places, the
  template's number breaking a tie.
- **Its line,** at most 120 bytes of the person's words: "Ask a
  question or ask for a change." How work runs shows it under the
  template's name, its "line" (`../client/pages/settings.md`, 2).
- **A template with no offer** is in no place: How work runs lists it,
  and New's "All ways to start work" leads there.
- **It grants nothing.** An offer names no executor, budget or
  authority: a template is served, never used (jig's
  `domain/engine/views.md`), and a start from an offer is the
  start its template fills, judged by the engine as any other.

## 3. Places

- **New:** the menu beside "+ New chat" in the rail. Its first offer
  is the one press, "+ New NAME", NAME its template's name; the menu
  lists New's offers in order, a name and a line each, at most six,
  then "All ways to start work", to How work runs at its ways to start
  work. With one offer in New there is no menu, and with none no entry.
  On a phone, whose rail holds the switcher alone, the Chats page's
  start offers the same.
- **Settings:** each of Settings' offers on the Settings page, its act
  opening the template's own page, as the configuring chat's opens
  (`configure-chat-in-settings`). To begin's first step (`to-begin.md`,
  section 3) and How work runs' "Tell temper how you work" start the
  first of them, and are not shown with none, where the client today
  names template 3, falling back to template 1 (`../client/pages/README.md`
  's first chat).
- **Later, a context's places:** a goal's page, a change's, a draft's:
  "Review this change", "Start a goal from this draft". A start there
  carries its context into the task, which jig's templates name no
  input for (jig's `protocol/configuration.md`, 3.1), so these wait on
  jig (section 9).

## 4. The settings

- **Kind 7, the deployment's offers:** scope the deployment; numbers
  the templates'; value an offer; decided by the deployment's owners.
- **Kind 8, a project's offers:** scope each project; numbers the
  templates'; value an offer, which replaces kind 7's for its template
  in that project, an offer with no places taking it out of every
  place there; decided by that project's owners.
- **Judged and kept as temper's other kinds** (`../protocol/configuration/settings.md`,
  2), by the connector that owns them (`template-places.md`, section 9): an item is
  refused that does not decode, names a place this draft does not
  have, carries a line past its bound, or names a template the
  settings do not define; a template removed takes its offers with it,
  in the same change.
- **Bounds:** a value's largest bytes, its places, its order and its
  line at 120 bytes; values per scope, jig's limit on templates.
- **Read** through the forge's family, as temper's other kinds' values
  are (`../client/forge/served.md`); the client puts a project's
  offer over the deployment's.
- **Proposed by the configuring agent** as any setting ("put reviews
  first in the menu"), nothing changing until an owner accepts
  (`configuring.md`).

## 5. The stock

| Template | Places | Order | Line |
|---|---|---|---|
| 1, a chat | New | 1 | Ask a question or ask for a change. |
| 2, a goal | New | 2 | Agents plan a larger change and land it. |
| 3, the configuring chat | Settings | 1 | Describe how you work; temper proposes how it will run that. |

Kind 8 has no stock items: a project follows the deployment's offers
until its owners change them.

## 6. The client

- **The rail's entry and its menu,** from New's offers (section 3).
- **The Settings page's offers,** from Settings' (section 3).
- **How work runs' ways to start work:** each template's name and line,
  and, opened, its places and order, tagged Default, Changed or Added
  as every setting is, with "Change it in a chat" (`../client/pages/settings.md`,
  2).
- **Offered whatever the role:** a template the person's role may not
  start is offered all the same; the engine judges the start, and its
  refusal says why, since the client never computes what a party may do
  (`../client/view.md`, 3).
- **Live:** a change of the offers reaches an open page as `me` and the
  forge's reads deliver it; an open menu closes when its items change.

## 7. Its tests

- **The forge's judging:** an offer refused for an unknown place, a
  line past its bound and an unknown template; a project's offer over
  the deployment's; a template removed with its offers.
- **The client's world:** the rail's entry and menu built from the
  offers; no menu with one offer in New and no entry with none; To
  begin's first step gone with no offer in Settings; a change of the
  offers reaching an open page.
- **The configuring agent's story:** "put reviews first in the menu"
  proposed as kind 8's items, and accepted.

## 8. Its order, when it is taken up

1. **temper:** kinds 7 and 8, the stock offers, their judging and reads.
2. **The client:** the rail's entry and menu, the Settings page's
   offers, How work runs' rows and To begin's first step.
3. **The configuring agent:** its instructions name offers
   (`../domain/agent/agents.md`, 1).

## 9. Open questions

- **When:** the user's; this draft keeps the idea, and no plan takes it
  up until then (2026-11-04).
- **Who owns kinds 7 and 8:** the forge's connector, which owns
  temper's other kinds, or a child of temper's root of its own for the
  client's settings: jig lets the root route a kind to any child (jig's
  `domain/engine/settings.md`, 5), and an offer is no concern of the forge's.
- **A context's places** need a template's input (section 3), a jig
  change.
- **A person's own:** recent ways first, or a person's order, kept by
  the engine for the person as "Skip" is (`to-begin.md`, section 5).
- **A chat's word:** "chat" or "session" (the user, 2026-11-04,
  thinking aloud). "Session" is smith's word for one conversation with
  an LLM that a run opens (`smith/docs/design/domain/session.md`) and
  the client's for a signed-in link; the New entry's word is its
  template's name, a setting.
