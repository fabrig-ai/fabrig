# Design content review

Report, 2026-11-11. Read the root index, directory README preambles and
navigation, every non-draft Open questions and Later section, the two
"What is open" index sections, and the browser's embedded Later list.
Searched the non-draft design for repeated headings, bold terms and
matching passages, and searched for an answer to every deferred item.
Read candidate and answer sections, not the rest of the documents whole.
Cross-tree answer searches covered skein, smith and hearth; temper was
not reviewed. Drafts and status entries were excluded.

Findings below belong to their documents' designers (`DESIGN.md`, 1).
No contract, stored form, default or unanswered choice was changed.

## 1. Counts

Left: 5 findings: 1 overlap, 2 partly answered questions, 1 stale deferral,
1 duplicated-question group. Done: 3 unnumbered question moves or
consolidations by hand. No numbered section or document moves.

## 2. Findings left

### J1. Overlap: application's settings hand-off

`jig/docs/design/domain/engine/settings.md`, 5 and
`jig/docs/design/domain/connectors.md`, 17 both specify default reporting
at startup, judging within the decision, keeping proposal/current values,
restoration history, and reading the newest unpinned value. These are two
full accounts of the same cross-child contract in the domain layer, not
just a connector summary citing the owner.
**Owner:** `jig/docs/design/domain/engine/settings.md`, 5.
**Suggest:** keep the common hand-off contract there and make connectors
state only its own obligations by reference, retaining its world cases.
The sections mix shared rules and each child's responsibilities; one
cannot be moved wholesale without choosing what to retain.

### J2. Partly answered: a missing brief section

`jig/docs/design/domain/engine/README.md`, 10's Briefs item asks how a
missing section appears to the agent, alongside gather budgets and
cancellation. `jig/docs/design/domain/engine/briefs.md`, 1 and 1.1 already
specify required versus optional sections, whole-brief failure, capacity,
transient/permanent classes, fixed section/reason words and their route to
task notice, hold and log. **Owner:**
`jig/docs/design/domain/engine/briefs.md`, 1.1.
**Suggest:** the core designer separate any remaining cancellation or
load-budget choice from the answered missing-section question, citing the
failure contract. Removing the mixed item's answered part needs an edit,
not a move of that entire item.

### J3. Partly answered: upgrades in place

`jig/docs/design/protocol/README.md`, 14 still asks about records translated
between versions during a store upgrade. The account in
`jig/docs/design/protocol/store/README.md`, 11 explicitly settles the
opening pass that patches a copy and switches to it, then enumerates the
remaining patch issues. **Owner:**
`jig/docs/design/protocol/store/README.md`, 11, for store upgrade questions.
**Suggest:** retire that part of the overview's question in favour of its
store reference, leaving the distinct engine/worker rolling-upgrade
question in the protocol overview. No rolling-upgrade answer is inferred
from an offline store-copy design.

### J4. Stale deferral: serving a compressed bundle

`jig/docs/design/protocol/browser/README.md`, 15 still lists the bundle
served compressed as Later. `skein/docs/design/ui/web/bundle.md`, 2
already specifies default-on deterministic gzip forms, and
`jig/docs/design/protocol/service.md`, 11.4 already supplies the bundle
helper's compressed File and `compressed_of` alongside the server.
**Owner:** `jig/docs/design/protocol/browser/README.md`, 15.
**Suggest:** its designer remove the broad deferral or narrow it to a
specific application-serving obligation that remains. That narrowing is
not decided here; skein's separate Brotli question is not the same work.

### J5. Duplicated question: a client-relayed OAuth redirect

`jig/docs/design/domain/accounts.md`, 7 and
`jig/docs/design/protocol/llm.md`, 10 both propose a native client's
loopback listener forwarding the redirect to a remote engine. The account
item also requires provider approval; the protocol item also raises the
confidential redirect on the application's server.
**Owner:** `jig/docs/design/protocol/llm.md`, 10, for that redirect flow.
**Suggest:** keep one flow question there, preserving the approval
condition and the confidential case, and have accounts cite it. The
nonidentical conditions require an editorial merge, not one item's move.

## 3. Moves done

The map is `tools/citations/maps/content-review-jig.map`. Each line maps
an unnumbered item, so these were handled by hand, not by `move.py`.

- `jig/docs/design/client/domain/client/README.md`, 15, "A root's two
  modes" -> `jig/docs/design/client/domain/root.md`, 14. The whole item
  moved; its local 4.4 citation now names client/README.md explicitly.
  The original location cites the new home.
- `jig/docs/design/domain/model.md`, 12, "Scaling out" ->
  `jig/docs/design/domain/engine/README.md`, 10, "Scaling out". Kept the
  existing fuller question and replaced the model's duplicate with a
  citation. The store's separate replication-mechanism question stays.
- `jig/docs/design/client/domain/README.md`, 6, "search" ->
  `jig/docs/design/client/domain/contract/README.md`, 9, "Search". Kept
  the existing question and made the index cite it. The store's separate
  index-placement question stays in its own layer.

## 4. Coverage limits

Only the root index and the deferred sections were read as whole units
throughout; the directory README bodies and other contracts were searched
and candidate sections read. Application settings, brief failure,
root/owner routing, browser guards and the default HTTP mapping were among
the expanded candidates. A table mapping messages to HTTP routes is not a
second design of their domain operations; browser and semantic-tree RTL
questions are different layers; a reference to another document's
question was not counted when it already leaves the choice there.
The review establishes neither implementation state nor the answers to
still-needed measurements or external-provider experiments.
