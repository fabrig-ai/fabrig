# Request: hearth's documents as a tree of fabrig's, and no private names

From fabrig/hearth, 2026-11-04, approved by the user. hearth joined the monorepo on 11-04 without the normalization the
other trees had on 11-02. This plan brings its design documents to the repository's conventions (`DESIGN.md`) and removes
private names from its current files. **Nothing of hearth's behaviour or design changes.** A section's meaning stays as
it is: if normalizing shows the design disagreeing with itself or the code, record it in the status file, and don't
settle it here.

## 1. Private names, in current files only

The user's decision: no git history is rewritten; the names go from the files as they stand.

Find them with `git grep -n -E 'ekanayaka|194\.164\.162\.|/home/free\b|free@' -- hearth`, 22 files or so on the domain and the
address and 9 on the path, and read around each hit for what the search misses: the user's account `free` in accounts
fixtures (`users free 1000 free /home/free /bin/bash Free ...`), hosts and repositories under the domain (`mx1.`, `git.`,
`infra/ekanayaka.io-software`), and a signed-off name or address.

Replace with reserved names, consistently, so a fixture still reads as one example: `example.org` for the domain, addresses
from RFC 5737 (`203.0.113.0/24`), and an account `alice` with home `/home/alice`. The target names the design calls the
fleet's (`x1-prod`, `mail-prod`, `rhi-prod`) stay. Say in the plan's notes if you find another name that looks private
and isn't listed here, and leave it for the user.

Fixtures and goldens change with the names, by whatever regenerates them; digests in goldens change for that reason alone.
A test that fails for any other reason after a rename is a finding, not something to update.

## 2. Design documents, by `DESIGN.md`

- A status file `<name>.status.md` beside every design document and draft, in `DESIGN.md`, 2's form. `STATUS.md`'s table is
  the seed: move what it says to each document's status, keeping only gaps as they stand, and make `STATUS.md` the
  one-paragraph index `DESIGN.md`, 4 asks for. Check the gaps against the code: M1 and part of M2 are built.
- Every design document closes with a "Later" or "Open questions" section. Where a document has one under another name, keep
  its number and rename the heading; don't renumber a surviving section.
- `draft/brainstorm.md`'s status says where the design lags or departs. `draft/archive/factory.md` is archived: see how temper's
  `draft/archive/` is kept, and do the same.
- Build state leaves the design text, except the milestone marks (`[M5]`), which `hearth/docs/design/README.md`, 3 defines
  as design.

## 3. Citations, by the root `AGENTS.md`

Inside hearth, documents are cited by their path from `hearth/docs/design/`, and foundation documents by bare name. The rule
is the path from the repository root. Rewrite by script, not by hand: about 560 citations in about 330 `.rs` files, and those
in `hearth/docs/`, `hearth/AGENTS.md`, manifests and `clippy.toml` comments. So `domain/render.md, 8.3` becomes
`hearth/docs/design/domain/render.md, 8.3`, and `programming-model.md` becomes `skein/docs/foundation/programming-model.md`.
Update `hearth/docs/design/README.md`, 6 and drop the convention from `hearth/AGENTS.md`. In skein's documents, make
"hearth's `domain/store.md`" the same root path. Long lines stay rustfmt-clean; a doc comment rewrap is fine.

`hearth/docs/design/draft/archive/factory.md`'s own text isn't rewritten beyond what the checker needs.

## 4. Stale pieces

- `hearth/README.md`: "The family" names separate repositories under an old organization. Make it what the other trees' READMEs
  say, or remove it if the root's account suffices. Fix links, including to its `AGENTS.md`.
- The `github.com/fabrig-ai/hearth` links in `skein/README.md`, `smith/README.md`, `jig/README.md` and `temper/README.md` become
  relative links to `hearth/`: the old repository is about to go.
- `temper/docs/design/draft/v0.1.0.md`, 6 leaves hearth out of the release; that stays true. Don't change it.
- Whatever still describes hearth as its own repository: a separate gate, `scripts/check.sh`, a git dependency on skein.

## Rules

- `tools/citations/citations.py --all` stays at 0 and `tests/boundary/tests/outward.rs` passes.
- Changes outside `hearth/` are only those sections 3 and 4 name.
- Surviving sections are never renumbered (`DESIGN.md`, 4).
- `hearth/docs/development/debt/` is fabrig/debt's: leave it, but fix a citation there if a rename breaks it.
- All parts must be in the plan when the cut ends.
