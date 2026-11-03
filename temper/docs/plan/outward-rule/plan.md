# outward-rule: plan

Request (`inputs/request.md`, step 1 of 12 of a user-approved plan): fabrig names nothing of
whoever runs its development, and this step stops new names arriving while later steps remove
the old ones. A new test `tests/boundary/tests/outward.rs` in `fabrig-boundary` (already in
every gate) reads every tracked text file and fails a file holding more of the lab's names than
the inventory `tests/boundary/outward.txt` allows; the inventory is generated at the task's base.

## Parts and tasks

- **outward-check** (slice 1): `tasks/outward-check.md`, no needs, released in the first and
  only push. The test, its ten tests (nine unit tables and the scan), the inventory and the
  package's description line; the root `Cargo.toml` gains `opt-level = 3` for the package only
  if the scan is over 1 s without it.

Critical path: the one task, ~75 implementer-minutes, then fab's merge and trunk check.

## Decisions

- **One task, as the request says.** The matcher, the inventory and the scan are one file and
  one inventory; split, neither half could be checked alone.
- **No design section is written.** No document at the root holds the repository's checks;
  `direction.rs` states its rule in its module doc, citing `AGENTS.md`, 1, and this test does
  the same, citing `AGENTS.md`, 2 (public code names nothing private, session names among it).
  The request, cited by heading, is the task's source; the part's `design` names `AGENTS.md`, 2.
  The request says nothing else changes, so `AGENTS.md` gains no line.
- **The files are git's tracked ones** (`git ls-files` from the workspace root), not a walk: a
  builder slot's warm `target/` and any untracked scratch never count, and the gate reads
  exactly the committed tree. Tests already run git under heavy (`smith/benchmarks`'s
  `merge-base`, `skein-codegen`'s patch check). The spec pre-approves a walk (skipping `.git`,
  every `target` and symlinks) if git cannot run there.
- **The matcher is written down as a table** with boundaries per name, ASCII
  case-insensitive, leftmost and never overlapping, and a `git grep -P` cross-check gives the
  same counts: at the cut (`58edb47912`) 71 files and 231 names, by both a Python reference and
  `git grep`. The table settles what the request leaves open: `agents repo` needs a boundary
  after it, since "agents report" is smith's product prose (`smith-host-domain/src/parent.rs`,
  `smith/docs/design/benchmarks.md`); `heavy env` needs one too ("a heavy environment"); the
  quoted `"heavy"` counts in any file (its holders are `tools/tiers/` and temper's
  `forge_land.rs` stubs, the same tool); the three paths are literal with their slash.
- **The inventory's rules:** sorted, `PATH COUNT` with a count from 1, `#` comments; fewer and
  gone pass; the failure prints the inventory the tree needs, keeping comment lines and the
  `# Raised:` lines of the entries that stay. The test cannot see who raised a count; the
  `# Raised:` rule is the inventory header's, as in the old-forms baselines.
- **Speed:** the scan reads ~16,500 text files, 122 MB (7,351 binary files skipped). The
  target is under 1 s by nextest's figure; jig-boundary's `opt-level = 3` profile line is the
  pre-approved remedy; past it, a measured question for the user.
- **The commit message gives the time without the machine's name**, so that the new history
  does not name the lab either; the report names the machine.

## Overlaps with the drafts beside this one

- **Files:** the task's files are new, but for `tests/boundary/Cargo.toml`'s description, which
  no other draft touches. The root `Cargo.toml` changes only if the profile line is taken;
  playground-browser's `browser-walk` lists `Cargo.toml` too, in other hunks.
- **Names other drafts add** cannot be avoided: once this lands, a branch that adds a name of
  the lab beyond the inventory fails `fabrig-boundary` at its trunk's check. Of the trunks
  beside this one, as this repository's remote refs held them at 2026-11-03 22:54Z,
  **fabrig-scale** (`ab9eea5025`) adds three: "as fab runs it" in
  `temper/docs/design/protocol/configuration.md` (lines 603 and 1766) and
  `temper/docs/design/protocol/setup.md` (line 2057). Whichever of the two lands second meets a
  red outward check; its repair rewords those lines (fabrig-scale's design) or lists the two
  files under a `# Raised:` line. public-reading-build, playground-browser, configuring-p4,
  bugs-security, status-lines and debt-1 add none; v0.1.0-ladder, sign-in-wait and
  reference-walks had no trunk there.
- **The gate's docs fast path, if fab has one:** `.config/gate/read-by-tests.txt` lists the
  Markdown tests read, and every text file is now read by this test. The request says the
  package is in every gate (`.config/gate/always.txt`); if fab's gate skips nextest for a
  change touching only unlisted Markdown, such a change could add a name unseen until the next
  gate that runs the package. This plan adds nothing to `read-by-tests.txt` (the request
  changes nothing else); fab's gate is fab's to check.
