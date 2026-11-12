# A charter's overlay

A deployment that wants its own lines in a charter its application ships,
and every later release's improvements to that charter too, keeps an
**overlay** over the charter's default rather than a copy of it: the
fields it replaces and the text it adds after the default's. Each start
lays it over the binary's default again, so a release's new default
reaches the deployment with the overlay in place. This document is the
overlay's mechanics; the settings it is one value of are `settings.md`;
a charter's members are `../../protocol/configuration.md`, section 3;
its parts, `../tasks/messages.md`, 6.

## 1. What an overlay is

- **A value of a charter setting,** beside a charter whole (`settings.md`,
  1): a change sets it, as it sets any value (2), over a charter that has
  a default. A charter no default has, one a change added, takes none.
- **The fields it replaces:** any of the charter's members but its
  number, its instructions, its parts and its wake policy, each named or
  left out: `rest`, its main model, its alternatives, `inspect`,
  `modify`, `shell`, `agents`, its host families' `tools` mask, its
  hosts, its waiting time, whether its runs resume, a run's turns and
  time, the reserve of turns and time, a call's timeout, its spend and
  its documents. A field named replaces the default's whole (a list
  named empty leaves none); a field left out follows the default.
- **The text it adds:** instructions, which may be empty, and parts, in
  order, which may be none, each in a charter's own shape
  (`../tasks/messages.md`, 6).
- **The charter it makes,** the overlaid charter, is the default with
  each named field replaced; its instructions the default's, then, when
  the overlay's are not empty, two line feeds and the overlay's, every
  byte as written, as a run's selected parts follow its charter's
  instructions (`executors.md`, 1), the overlay's alone where the
  default's are empty; its parts the default's, then the overlay's, in
  order. An overlay's parts are parts of their own, selected by their
  reasons as any part is, never merged into a default's part with the
  same reasons. So a default with instructions `A` and parts `[p1, p2]`
  under an overlay naming `shell` true, with instructions `B` and parts
  `[q1]`, makes a charter whose `shell` is true, whose instructions are
  `A`, a blank line and `B`, and whose parts are `[p1, p2, q1]`; its
  other members are the default's.
- **The current charter** is the current default with its overlay over
  it. Every site that reads a charter reads the overlaid charter, through
  `charter_of` and `charter` (`settings.md`, 3): a run's preparation, the
  core's checks of its calls, its placement, the children told its
  versions. Only the reads of 6 and the store (7) see the overlay itself.

## 2. A change

- **Set as any value:** a `ChangeSettings` item *set* to an overlay
  (`settings.md`, 2), at most one item per setting. It replaces whatever
  change stood over the charter, a charter whole, a removal or another
  overlay. A *reset* erases it, so the charter follows its default; a
  *set* to a charter whole replaces it, and so does a *removal*. An
  overlay is never added to: a second overlay replaces the first whole,
  and a client that joins several into one (an application's settings
  file, say) sends the joined one.
- **Checked whole:** the settings the change would leave are checked by
  `settings.md`, 1's one check, the overlaid charter as any charter: its
  stored policy and parts within the journal's run bytes, its parts
  within `tasks.charter_parts`, the default's and the overlay's counted
  together, no parts where its runs resume, its budgets above its
  reserve, its models configured, its documents within their bounds. A
  failure names the charter and the fault of that check's closed list,
  as for a charter whole: an overlay's part on an unknown reason is
  *shape*, an overlay of a charter with no default *reference*.
- **Empty:** an overlay equal to the one stored over the charter changes
  nothing, and counts toward *empty*; one whose overlaid charter equals
  the current charter but which stands where a charter whole or no
  change stood does change the settings, since what follows the default
  changes.
- **Its revision** keeps a version of the overlaid charter, as any
  revision that changes a charter does (`settings.md`, 3).

## 3. Revisions and running work

- **A version is the overlaid charter,** whole, as the revision that made
  it resolved it: the overlay over the default that revision held. A run
  reads its revision's version (`settings.md`, 3), so a task already
  running keeps its words when a release changes the default under the
  overlay, and new work reads the new revision's.
- **Moving running work** (`settings.md`, 4) is unchanged: a task moved
  to the newest reads the newest's overlaid charter.

## 4. At a start

- **The defaults are laid under the overlay again.** A start reads the
  stored overlay and lays it over the default it is given
  (`settings.md`, 7). A default that differs under an overlay changes the
  current settings, unlike a default under a charter whole, which the
  change masks: so it makes the release's revision, with a version of
  the newly overlaid charter, and withdraws stale proposals as that
  revision always does. An overlay whose default is unchanged makes no
  revision.
- **Checked whole:** the current settings are checked as `settings.md`, 7
  checks them, each overlaid charter whole. A start whose new default
  leaves an overlaid charter past a bound (its stored policy and parts
  past the journal's run bytes, its parts past `tasks.charter_parts`, its
  budget below its reserve) is refused, naming the charter, the overlay
  over it and what fails, as a default past its bounds is. A stored
  overlay whose charter the defaults no longer offer refuses the start
  as *reference*, naming the charter: nothing is left to lay it over.
- **Kept across a restart:** a start with the same defaults reads the
  same overlay and makes the same overlaid charter, and makes no
  revision.

## 5. Proposals, history and restoring

- **A proposal** (`settings.md`, 6) carries an overlay as it carries any
  value: proposed, reviewed, accepted, refused and withdrawn when stale
  alike, and checked whole at its proposal and again at its acceptance
  (2).
- **A revision's record** keeps an overlay item's value, the overlay, and
  what it replaced (`settings.md`, 10). An item that replaced an overlay
  records that overlay, never the charter it made, so that what it
  replaced follows the default. A release's revision records an
  overlaid charter whose default changed as having held its overlay.
- **Restoring revision R** recreates the overlay R held: a charter
  overlaid at R is proposed *set* to that overlay, which lies over
  today's default, as a charter that followed its default at R follows
  today's (`settings.md`, 10). Neither the default beneath an overlay nor
  the charter it made is ever restored as a charter whole.

## 6. Reads

- **A setting's read** (`settings.md`, 7, `../../client/domain/contract/README.md`,
  section 4) of an overlaid charter gives the charter as its runs read
  it, the revision's version, as for any charter, and beside it the
  overlay itself, whole; a charter with no overlay gives no overlay. So
  a client shows what runs read and what the overlay replaced and added,
  and gives the overlay back whole.
- **The inventory** marks an overlaid charter's origin as overlaid,
  beside the default's and a change's.
- **A revision's items and a proposal's review** give an overlay item's
  value as the overlay; a review's charter as it would be is the
  charter the overlay makes over the default the newest holds.

## 7. Its forms

- **One shape, every form:** the fields of 1, each present or absent,
  then its instructions and its parts, as a charter's are. It is a new
  member of each union that carries a setting's value: the core's
  `settings::Value`, the client contract's `SettingValue` and its wire
  (`../../client/domain/contract/requests.md`, 2), the store's settings
  record (`../../protocol/store/README.md`, 4.4) and a proposal's
  stored items, which the settings tool's bytes carry too. A setting's
  read document gains the optional overlay of 6.
- **Additive:** every record and message that decoded before decodes the
  same; a value of the new member reaches only a peer built with it.
  Before jig's first release a stored format changes in place, with no
  version or migration.
- **Bounds:** an overlay's instructions and each part are bounded as a
  charter's own; its record is priced as a charter's in the settings'
  bounds (`settings.md`, 8), never above it.

## 8. Open questions

- **Unsaying a default's line:** an overlay only adds text; a deployment
  that must drop a line of the default sets the charter whole.
- **An overlay of another kind:** a template, a policy or the rules
  overlaid over their defaults, if a deployment needs one; today only a
  charter takes an overlay.
