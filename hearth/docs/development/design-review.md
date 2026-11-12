# Design content review

Report, 2026-11-11. Read the root index, directory README preambles and
navigation, all 17 non-draft Open questions/Later sections, and the three
machine/boot subsections marked later. Searched the non-draft design for
repeated headings, bold terms, matching passages and answers to each
deferred item; read candidate sections, not every contract whole.
Cross-tree answer searches covered skein, smith and jig. Drafts and status
entries were excluded; temper was not reviewed.

Nothing below selects a conflicting contract, retires an answered question
or treats a designed experiment as evidence that it ran. Each owner is the
document whose designer settles it (`DESIGN.md`, 1).

## 1. Counts

Left: 6 findings: 2 overlaps, 2 partly answered questions,
2 duplicated-question groups. Done: 1 misplaced unnumbered question moved
whole by hand. No numbered section or document moves.

## 2. Findings left

### H1. Overlap: whether a reconciled layer's base stays pinned

`hearth/docs/design/domain/store/README.md`, 5.1 holds a reconciled base as
an ancestor under a descendant's pin. But
`hearth/docs/design/domain/store/records.md`, 2 explicitly allows the base
to be collected while descendants are kept, as the reason for storing the
line's time. The store already names the inconsistency in
`hearth/docs/design/domain/store/README.md`, 10.
**Owner:** `hearth/docs/design/domain/store/README.md`, 5.1.
**Suggest:** settle the retention rule there and make the record's
explanation agree. Moving either section leaves two different rules.

### H2. Overlap: who constructs a resolution's closure

`hearth/docs/design/domain/render/README.md`, 5.2 has the protocol layer
return `Resolved` with a closure. In
`hearth/docs/design/domain/render/spec.md`, 2.3 the protocol supplies Inst
triples and Packages stanzas and the domain matches them into the closure.
These assign the same construction to different sides of the boundary;
`hearth/docs/design/domain/render/README.md`, 7 already records the conflict.
**Owner:** `hearth/docs/design/domain/render/spec.md`, 2.3.
**Suggest:** settle the construction boundary and adjust the vocabulary
account accordingly. This changes what crosses the boundary, not just its
location in the document tree.

### H3. Partly answered: unshare versus root mode

`hearth/docs/design/domain/build/README.md`, 13 asks which mmdebstrap mode
runs inside the sandbox. `hearth/docs/design/domain/build/sandboxes.md`, 3
explicitly decides unshare as the sandbox's root, explains why it makes
no second user namespace, and retains root as a conformance alternative.
`hearth/docs/design/domain/build/packages.md`, 4 uses unshare.
**Owner:** `hearth/docs/design/domain/build/sandboxes.md`, 3.
**Suggest:** the build designer replace an undecided-mode question with
that decision and, if still needed, keep only the conformance evidence
question. No result of running either mode is claimed here.

### H4. Partly answered: persistent owners' dynamic ids

`hearth/docs/design/domain/machine/README.md`, 6 asks about persistent
owners being renumbered by maintainer scripts, depending on format/build
pinning them. `hearth/docs/design/protocol/format/tasks.md`, 6 now requires
fixed ids for every persistent data, secret and identity owner and rejects
one without them; `hearth/docs/design/domain/build/packages.md`, 5 seeds
the package owners' ids before configuration.
**Owner:** `hearth/docs/design/protocol/format/tasks.md`, 6.
**Suggest:** retire that conditional machine-layout question against the
fixed-id contract, or name a concrete exception still undecided. This does
not settle whether prepare should stop parsing account files: their
verification role remains a separate activator question
(`hearth/docs/design/domain/activator/README.md`, 9;
`hearth/docs/design/domain/slots/prepare.md`, 4 and 7).

### H5. Duplicated question: a subvolume per data entry

`hearth/docs/design/domain/machine/README.md`, 6 and
`hearth/docs/design/protocol/format/README.md`, 11 both defer
`"subvolume": true` for an entry needing an individual snapshot, quota or
restore unit. Machine adds the proposed subvolume name and mount account;
format adds the restore/qgroup measurement condition.
**Owner:** `hearth/docs/design/protocol/format/README.md`, 11.
**Suggest:** keep one option question there with both constraints, and
have machine cite it for the layout consequence. Preserving those unique
parts requires a merge, not a duplicate item's deletion by this pass.

### H6. Duplicated questions: shared verification prerequisites

`hearth/docs/design/domain/development.md`, 11 and
`hearth/docs/design/testing/README.md`, 6 both ask whether an Incus copy
carries `qemu.nvram`. `hearth/docs/design/domain/slots/README.md`, 7 and
`hearth/docs/design/testing/README.md`, 6 both ask whether the watchdog
stays armed across a soft reboot.
**Owner:** `hearth/docs/design/testing/README.md`, 6, for the outstanding
verification and its evidence.
**Suggest:** one tracked experiment for each underlying property, with
consumer documents citing it; retain development's copy-time question,
testing's restricted-project questions, and its separate way to cause a
watchdog-only hang. Those mixed items are not identical units that can be
relocated whole without an editorial split.

## 3. Moves done

The unnumbered map is `tools/citations/maps/content-review-hearth.map`,
applied by hand, not `move.py`:

- `hearth/docs/design/domain/activator/README.md`, 9, "The steward's
  diagnostics" -> `hearth/docs/design/domain/steward.md`, 10.

The question moved whole, including its existing explicit citation of
the steward's spawn and open questions. The activator now cites that home.
The separate question of retaining a builder crash's stderr tail remains
beside it, not merged into it. No status label or section number moved.

## 4. Coverage limits

README bodies outside the root index, navigation, deferred sections and
expanded candidates were only searched. The detailed candidates included
store pins/records, render's resolution vocabulary, package hooks,
sandbox mode, users and prepare's account-file verification. Operation
records citing a slot's lifecycle, build steps applying the input format,
and the fake's account of a real tool were not counted as duplicate
contracts just because their subjects match. Genuine future alternatives
and requests to verify external kernel, firmware or tool behaviour stay
open; this docs review does not substitute code inspection or experiments
for that evidence.
