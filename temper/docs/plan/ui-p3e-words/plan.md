# ui-p3e-words: words saved with a setting

Milestone P3e of `temper/docs/design/draft/whole-story.execution.md`: words a
person or Ask temper writes with a setting, kept with it, and said where the
pages draw the setting: a charter's line on Roles, How work runs' headline and
lede, and a tier's description. Request: `inputs/request.md`. `notes/common.md`
holds the rules every task follows; `notes/contracts.md` the shapes that cross
tasks, the stock's words and Ask temper's sentence.

## For the user's review before landing: Ask temper's sentence

`ask-words` adds to `temper/crates/temper-service/src/stock/ask.txt`'s
*Configuring*, right after "...and what the change does to running work.":

> Write the words the pages show with what you propose, in their terms: for each
> charter, its charter-words, a line on what it does; where the change gives the
> project a way of its own, its project-words, a headline naming the way and its
> chief departure, and a lede of a sentence or two on who does what.

and, so that it can read and name them: `charter-words N | project-words` in
*Finding out*'s `settings [...]` addresses, and in *temper, as you explain it*,
"; and the words saved with them (a charter's line, a project's headline and lede
on How work runs)" closing the settings' list (`notes/contracts.md`, section 9).

## Parts and tasks

Released:

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `tier` | `tasks/tier-words.md` | | a tier's `description` in the configuration, bounded at the start, kept by the forge, "Checks" on its stop's card and in `see settings gates`; the reference's two tiers described | 100 |
| `words` | `tasks/words-wire.md` | | `CharterWords` and `ProjectWords` on the client wire and in the store, `StockWay.headline` and `WaysPanel.words`, regenerated, nothing filled | 60 |

To come, in the next batches (their parts join `plan.toml` as they are released):

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `words` | `words-value` | `words-wire` | the forge's kinds 9 and 10: values, judgement, `charter_words`, `project_words`, erasure, the store's codec | 90 |
| `words` | `words-kind` | `words-value` | kinds 9 and 10 declared to jig, at the wire's entrances, `charter-words` and `project-words` in a run's `settings`, `see` and `propose_settings`, the settings form | 150 |
| `words` | `words-tools` | `words-kind` | `temper ctl settings show charter-words` and `project-words`, the help | 60 |
| `stock` | `charter-lines` | `words-kind` | kind 9's stock words for charters 1, 2, 4, 5, 6, 9, 10, 11 and 12 | 60 |
| `stock` | `way-headlines` | `words-wire` | each stock way's headline, in the ways and the read | 75 |
| `how` | `project-words-read` | `way-headlines`, `words-kind` | the `ways` read carries the project's words, kind 10; the engine world reads them back after an owner's change | 75 |
| `how` | `how-words` | `way-headlines` | How work runs' heading and lede from a stock way's headline and the project's words | 100 |
| `ask` | `ask-words` | `words-kind` | Ask temper's sentence; a configuring story whose proposal carries a charter's and the project's words, accepted and read back | 60 |

About 830 implementer-minutes in all.

**Critical path, as it stands:** `words-wire` (60) → `words-value` (90) →
`words-kind` (150) → `project-words-read`, `ask-words`, `charter-lines` or
`words-tools` (60 to 75), about six and a half hours. `tier-words` runs beside it
from the first push; `way-headlines` and then `how-words` from `words-wire`'s
merge.

**The request's tests, by task:** the words stored and read back with each setting
(`words-value`'s store and forge rows, `words-kind`'s owner's change read back,
`words-tools`' ctl); `propose_settings` carrying them (`words-kind`'s typed
proposal rows, `ask-words`' configuring story); the stock's words for each stock
charter, way and tier (`charter-lines`, `way-headlines`, `tier-words`' reference);
How work runs drawing them (`how-words`' rows, `tier-words`' card).

## What was decided, and why

- **Two kinds of temper's, 9 and 10,** rather than fields of jig's charter or of
  kind 7: a charter is jig's, and its words are temper's pages' words, so they
  stay temper's, beside the charter by its number; kind 7 is under change by
  ui-reaches (`ways-away`), and a separate kind lets Ask temper set the words
  alone, as plain text. Numbered past kind 8, ui-reaches' classes, so that the
  two plans never take one number.
- **Kind 9 is the deployment's,** as charters are, numbered by the charter; kind
  10 a project's, number 1, its headline and lede together. Bounds: a line and a
  headline 120 bytes, a lede 480, a tier's description 120, so that each fits a
  card's fact or the heading whole and nothing is cut.
- **A stock way's headline** is the stock's data, carried by the `ways` read
  beside its name and line; its lede is "PROJECT runs the stock way NAME." and the
  way's existing line, so the read grows by three headlines alone. The project's
  words ride the same read, so How work runs reads nothing new; the worst `ways`
  document's test takes both at their bounds.
- **Which words How work runs says:** a stock way's while the project runs one,
  the project's own while it runs none, the fallbacks today's. A project's words
  never override a stock way it runs, so "Use this" leaves no stale headline
  showing, and a way's items don't touch kind 10.
- **A tier's description stays in the configuration document,** where tiers are
  until kind 5 holds them (`configuration.md`, 13.2): whoever writes the tier
  writes its words; Ask temper proposes no tier and writes none. **The stock holds
  no tier** (`stock-ways.md`, 5: tiers stay a repository's), so the stock's tier
  words are none; the reference's two tiers carry theirs as the example.
- **A charter's line on Roles is left out:** look-round-25's `roles-view` and
  `roles-page`, which draw Roles, had not merged even into their own trunk at the
  cut. This plan keeps, judges, ships and proposes kind 9, and draws it nowhere;
  the plan that draws Roles reads `Domain::charter_words` for the role's line,
  its instructions' first sentence where none is kept
  (`configuration.md`, 13.2; `surfaces.status.md`, 7, says so).
- **"Asked to" stays the instructions' first sentence** on a stop's card: P3e's
  charter words are what the role does, the line Roles gives it after when it
  runs (`surfaces.md`, 7), not what it is asked; the audit's "reads badly" is
  answered on Roles, where the line leads, once Roles is drawn. Rewriting the
  stock charters' first sentences is no part of this request.
- **No pairing enforced:** a proposal of a charter without its words is good;
  words are text an agent writes, as the instructions ask (`agent.md`, 8.4.2).
- **Status entries:** every **Not yet** this plan's design opened is written now,
  one bullet per task, each deleted by the task that builds it.

## Who hears first

- **A stored format** (for the TUI's owner): the store's `store-v11.schema` gains
  `CharterWords`, `ProjectWords` and `SettingValue`'s `line` and `words`
  (`words-wire`, `words-value`); the forge's settings rows hold kinds 9 and 10.
- **The client wire,** shared with the terminal: `client-wire-v1.schema` gains
  `CharterWords`, `ProjectWords`, `StockWay.headline` and `WaysPanel.words`
  (`words-wire`); `settings_kind` takes 9 and 10 (`words-kind`).
- **`temper-client-domain`** (for the TUI's owner): `forge/stock_ways.rs`' `Tab`
  gains `headline` and `StockWays` gains `words()` (`how-words`); its tests take
  the wire's new fields (`words-wire`).
- **`temper-engine-domain`** (for the owner of coding runs): `translate.rs`'
  `TierCheck` gains `description` (`tier-words`); `route/ways.rs` copies a tier's
  description, a way's headline and the project's words (`tier-words`,
  `way-headlines`, `project-words-read`); `route/kinds.rs`, `route/see.rs` and
  `route/host_route.rs` read kinds 9 and 10 (`words-kind`).
- **`forge_route.rs`** (for the owner of coding runs): the forge's `Tier` literal
  takes its `description` field, nothing else (`tier-words`).

## Overlaps with drafts beside this one

- **ui-reaches' kind 8** walks the same files a new kind does: the forge's
  `settings.rs` and `kinds.rs`, the store's schema and `records/settings.rs`, the
  wire's schema and `lib.rs`, smith's `kinds.rs`, `settings.rs`, `see.rs` and
  `calls.rs`, the service's `stock/kinds.rs`, `ask.txt` and `tests/stock.rs`, the
  settings form, the root's `route/kinds.rs`, `see.rs` and `host_route.rs`, and
  the help's references. Unavoidable: it is one mechanism. Each list gains its own
  lines; generated code and goldens regenerate on a catch-up.
- **look-round-25** changes the ways crate (`path.rs`, `path/derive.rs`, its
  tests), `route/ways.rs` and its tests, the wire's schema, the service's
  `family/ways.rs` and `tests/family.rs`' worst document, `forge.md`, 8.13, and
  How work runs' drawing. This plan's hunks there are a tier's card's fact, a
  way's headline and the project's words; in `temper-client-view/src/pages/how.rs`
  `how-words` edits `head` alone, never `path` or `table`.
- **The status files** `client/README.status.md`, `configuration.status.md`,
  `agent.status.md`, `forge.status.md`, `tools.status.md` and `store.status.md`:
  this plan's entries open each file's gaps, apart from one another.
