# Landings

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, section 3,
milestone Q1 of `../../draft/landed-qa.execution.md`. The watcher: how
landed QA sees a goal land on main, and how it learns which landings a
kept build holds.

## 1. In one page

- **Main's ref, not its first-parent line** (section 3): the watcher
  keeps the last head it read and, on a move, lists every commit the new
  head gained, oldest first, so a goal's merge carried off the
  first-parent line by a later merge is seen.
- **Seen once:** a landing is keyed by its merge's commit, and a commit
  recorded is never recorded again, whatever the head did meanwhile.
- **A goal's landing** (section 4) is a merge named by its `Goal:`
  trailer (`../../draft/squash-in.md`, 3), and until those land, by its
  pull request's subject. temper's task states are no signal.
- **Builds learn their landings** (section 5) once the head the watcher
  read holds their commit.

## 2. Main's ref

- **The repository** is `qa.json`'s `git.directory` (`README.md`, 3.2),
  bare or not, read with `git -C DIRECTORY`.
- **With `git.fetch`,** each read first fetches main from that remote
  into a ref of the tool's own: `git fetch --quiet REMOTE
  +refs/heads/BRANCH:refs/temper-qa/BRANCH`, then reads
  `refs/temper-qa/BRANCH`. Without it, the repository is main's own and
  the read is `refs/heads/BRANCH`. BRANCH is `git.branch`.
- **The read** is `git rev-parse --verify REF^{commit}`: the head.

## 3. A move

- **The first read** records the head as the ledger's `head` and no
  landing: the baseline, as a tier's start has one
  (`../../domain/after-landing.md`, 2). What landed before it is no
  pass's.
- **The same head** records nothing.
- **A new head:** `git rev-list --topo-order --reverse --parents NEW
  ^OLD` lists every commit the new head holds and the old one does not,
  each with its parents, oldest first. Each merge among them (two
  parents or more) is read (section 4); each landing it names, whose
  commit the ledger lacks, is recorded in that order, and then the head.
  So landings are recorded in main's order, and a merge reached through
  any parent of a later merge is seen.
- **A head that is no descendant** of the last (main rewritten) is read
  as any new head: what it gained is listed, and what was recorded stays.
- **Under the ledger's lock,** fetch and all, as one step of the loop
  (`README.md`, 4).

## 4. A goal's landing

A merge's message (`git show -s --format=%B COMMIT`) names a goal's
landing in one of two forms, the first that matches:

- **Its trailers:** the message's last paragraph read as trailers, `Key:
  value` lines; a `Goal: N` trailer, N a task number, names goal N's
  landing, with `Pull-request: P`, where it has one, its pull request.
  Its title is the message's subject. A change's own squash commit
  carries these trailers too, but has one parent, so it is never read.
- **Its subject:** `Merge pull request 'TITLE' (#P) from OWNER/gN into
  BRANCH`, BRANCH main's name, read from its end, since TITLE may hold
  any text: ` into BRANCH` last, before it ` from OWNER/gN` with N a goal's
  number (its branch, `../../domain/forge/branches.md`, 1), before it ` (#P)`,
  and TITLE between `Merge pull request '` and the closing `'`.

Any other merge is no landing: a change's merge into its goal's branch
(`into OWNER/gN`), main merged into a goal's branch (`Merge branch
'main' into ...`), or a person's merge with no goal.

## 5. Builds' landings

At each read, every kept build whose `holds` is absent (`ledger.md`,
2.4) and whose commit the head holds (`git merge-base --is-ancestor
COMMIT HEAD`) is given:

- **`holds`:** each landing whose merge the build's commit holds
  (`git merge-base --is-ancestor MERGE COMMIT`, or the same commit);
- **`order`:** `git rev-list --count COMMIT`.

Every landing such a build holds is recorded by then, since the head
holds it, so its `holds` is final. A kept build whose commit the head
does not hold yet waits for a later read; the deployment keeps builds of
main's tips alone (`builds.md`, 2).

## 6. Its git face

Every git call is a method of one face of the crate's: read the head
(after the fetch), list a move's commits with their parents, read a
commit's message, ask whether one commit holds another, count a
commit's commits, and triage's own (`triage.md`, 2 and 3). The loop runs
the real face, `git` with the captured environment; the tests a fake
over commits in memory (`README.md`, 6). Parsing git's output is a
function of its own, tested on text.

## 7. Later

- **A landing temper did not make** with no goal (a person's merge into
  main) is no pass's today; it may be one once a trailer or a mark names
  it.
