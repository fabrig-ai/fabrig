# A repository's `.temper/`, read and guarded

Provisional, 2026-11-28, from `../../draft/ways-temper-dir.md`, sections
1, 5 and 6 (`../../draft/ways.execution.md`, W2). How the forge reads
the home repository's `.temper/` at its landing branch's tip and keeps
it as the project's repository revision, how it checks `.temper/`
before a change lands, and the owners' sign-off. What is read of the
files, and the judgement made of them, is
`../../protocol/temper-dir.md`.

## 1. In one page

- **The read** (2): the home repository's `.temper/` at one tip of its
  landing branch, every file at that commit or none, at adoption, at a
  move touching it, at a change of home and at an engine's start.
- **The repository revision** (3): what a read gave, judged, saved as a
  record of the project's own that only a landing moves; a read that
  fails or doesn't judge good keeps the last good revision, and the
  record says which commit it couldn't take.
- **The check before landing** (4): a change into a landing branch that
  touches temper's method waits for temper's own judgement of the tree
  that will land: under the queue its head once up to date, under the
  turn the candidate. A fault refuses with its file and line. No
  landing rule or setting removes it.
- **The owners' sign-off** (5): a project setting, off by default; on,
  such a change also waits for one owner's approval on the forge.
- **A project without temper's method** in its home runs as it does
  today: its read records that it holds none, and no change of it is
  checked (4).

## 2. The read

- **Of the home's landing branch:** the project's home repository
  (`change/branch-path.md`, section 1), at a tip of the branch its
  record names `landing` (`adoption.md`, 12). Where what asks for the
  read names no tip, a `Read::Branch` gives it first. Then the forge
  reads `.temper/` at that commit as `../../protocol/temper-dir.md`,
  2 says, through its client, each answer at that commit and that path,
  and judges it (8 there) as the home's.
- **Whole or none:** a read the forge refuses or fails, or answers for
  another commit or path, is no read. It changes no revision; the
  record says so (3).
- **Tried again:** after a read that failed, at the next trigger, and,
  where the forge failed it, once more after the forge client's
  `backoff`, as a follow's baseline read is (`follows.md`), until one is
  taken. A tree that judges with a fault is no failure of the read: it
  is read again only at a trigger.
- **One read per project at a time:** a trigger while one is in flight
  reads once more when it ends, at the landing branch's tip then, as
  the CI workflows' read does (`adoption.md`, 11).
- **When:**
  - **at adoption,** once an adoption that makes the repository the
    project's home, or keeps it so, is saved and taken, not one the
    root's seeding of its party rolls back (`adoption.md`, the act of 9
    alike);
  - **at a move of the home's landing branch,** watched where the home
    holds temper's method, whose landing's files (`topics.md`, 1.1)
    name a path under `.temper/`, or aren't known (the comparison
    refused, past its bound, or not made), or whose comparison has
    `behind` above 0: a rewind or a branch moved past its tip, whose
    files from the merge base say nothing of what went. A move someone
    else made is a landing as temper's own merge is. A home without
    method gets no watch of its own: a first `.temper/ways.toml` added
    outside temper is read at the next engine start or the next landing
    temper already sees on that home;
  - **when the project's home changes:** a repository made its home is
    read at its adoption (above); a home released, or on a forge that
    ended, leaves the project with no home and nothing to read, and its
    record stays as it was read (3);
  - **at an engine's start,** once the forge has restored its state and
    catches up (`keeping-up.md`, 1), each project with a home is read
    once, at its landing branch's tip, trusting no mark held in memory.
- **Nothing else is read:** no other branch, no other repository, and
  never a context repository's `.temper/`.

## 3. The repository revision

- **A record of the project's own** in the store, one per project,
  `TemperRevision`, a forge family (`../../protocol/store.md`, 2):
  - `project`;
  - `number`, from 1, and 0 before the project's first good read;
  - `home`, the repository it was read from, by its forge and id;
  - `commit`, the commit whose read made `number`;
  - `method`, whether that tree holds temper's method
    (`../../protocol/temper-dir.md`, 2);
  - `files`, the method's files that read gave, each a path and its
    bytes, within the judgement's bounds;
  - `observed`, the latest observation it took (below);
  - `refused`, none, or the latest read since that failed or judged with
    a fault: its commit, where one was read; its observation; and its
    words, the first fault written ``PATH:LINE: WHY`` with the count
    past it, or the forge's failure, at most 1,024 bytes.
- **No person sets it:** it is no settings kind and no `temper ctl` act
  writes it. Only a read of the landing branch moves it, so only a
  landing does.
- **A good read takes it:** a tree with no fault, or one that holds no
  temper's method, whatever its hooks (4). Where its home, `method` or
  `files` differ from the record's, `number` moves to the next and the
  record takes the read's `home`, `commit`, `method` and `files`; where
  they are the same, the revision stands. Either way `observed` moves
  and `refused` clears.
- **The last good revision holds:** a read that failed, or a tree with a
  fault, leaves `number`, `commit`, `method` and `files` as they were
  and sets `refused`. Work started meanwhile takes the last good
  revision; before the first good one there is none.
- **Observations, in order:** each read is stamped, as it starts, with
  the project's next observation, one past any it stamped before, the
  record's `observed` seeding it as the forge restores. An answer is
  taken only where its observation is above the record's `observed` and
  its home is the project's home as it arrives; any other is dropped,
  whatever its commit, so a rewind's read replaces a revision made at a
  newer commit, and a former home's read is never taken.
- **Saved whole,** in the decision that takes a read's last answer: one
  row, written whole.
- **Erased with the project,** when it is released, as its other forge
  rows are; never at a change of home.
- **What its readers take:** a project whose record has `method` false
  at its home runs the stock ways (`../../draft/ways-temper-dir.md`,
  5); what reads `files` is the charters' and the ways' designs.

## 4. The check before landing

- **What it covers:** a change into the landing branch of one of the
  project's writable repositories (`adoption.md`, 12), where the probe
  below finds method and the comparison of the base's tip and its head
  (`Read::Compare`, `change/gates.md`, 2) touches it, by the paths it gives:
  - **the method:** a path is `.temper/ways.toml`, or under
    `.temper/ways/` or `.temper/charters/`; always covered;
  - **`.temper/` alone:** else a path is under `.temper/`; covered where
    the tree that will land holds temper's method, as its listing of
    `.temper/` says;
  - **unknown:** else the comparison isn't whole (its count past its
    paths, other endpoints, or failed); covered where the tree that will
    land, or the base's tip, holds temper's method;
  - **nothing:** else; never covered.

  A change into any other branch isn't covered either: a goal's branch
  lands whole as the change that lands it into the landing branch
  (`branches.md`, section 2), and is checked then.
- **Probe before comparing:** the step that could decide a queue merge
  or admit the change to a turn reads whether the change's head or its
  base's tip holds method, pinned to those commits. It keeps the result
  in memory by head and base, never in the store. If neither holds
  method, the change is not covered and the check makes no comparison,
  even if method was present at the merge base. If either holds method,
  compare and apply the categories above. A failed probe waits and is
  read again at the next step; it never refuses or repairs and is never
  taken as absence. The probe makes no start or timed reads.
- **Read once per pair:** the comparison is read where a step may
  decide on it, and kept by head and base in the forge's working set,
  as `Touched` is (`change/gates.md`, 2), so a pair is compared once.
- **The tree that will land:**
  - **under the queue,** the change's head once it contains the base's
    tip: a covered change is made up to date first, whatever its rules
    say (`change/gates.md`, 2, `requires`), then its head is judged;
  - **under the turn,** the candidate the turn will move the landing
    branch to, against its tip read afresh (`../../landing-turn.md`,
    1). The change's own step judges its head first, before it joins a
    turn, so a fault of its own is found while it can repair.
- **A pass** is a judgement with no fault, or of a tree holding no
  temper's method, naming the head, the base's tip and the commit
  judged. A step whose head or base tip differs has none, and its
  merge's evidence says the check is still to pass. The pass is kept
  in the forge's working set beside the change's review facts, never
  stored: after a restart the tree is judged again.
- **Then it may land:** the step that decides a merge reads its facts
  afresh, and asks the merge only where its evidence says the check
  doesn't cover it, or passed at that step's head and base. It is observed as up to date
  is (`change/gates.md`, 2): a base that moves between that decision and
  the merge lands on a tip the check didn't judge, and the read after
  that landing judges the tree landed, the last good revision holding
  where it fails (3).
- **Temper's own:** it holds whatever the landing rules and jig's
  requirements ask, enforcing or not, and whatever a task's authority
  grants. No rule's removal and no setting turns it off.
- **A fault at the change's own head,** under the queue or before a
  turn: a repair, its reason the check's words, ``temper's check of
  .temper/ at COMMIT:`` and then each fault, ``PATH:LINE: WHY``, one per
  line, with the count past them; counted against the change's repairs
  as any repair is (`change/repairs.md`).
- **A fault at the turn's candidate:** each member whose own comparison
  touches the method or `.temper/`, or is unknown, leaves the turn and holds, the
  hold's words the check's, and the turn goes on without them, as a
  member refused by a gate leaves (`../../landing-turn.md`, 1). Their
  own heads passed, so the fault is theirs together and asks no repair:
  a person or the change's requester amends or releases each.
- **A read that fails:** the change waits, the check still to pass, and is read again
  at its next step. It never lands on a read that failed, and never
  repairs for one.
- **A project without temper's method:** where neither tip holds method,
  a change touching `.temper/` alone, its hooks among them, is not
  covered and lands as it does today, without the check comparing the
  tips. A change introducing method is still checked before landing.
  Hooks are checked where they run (`../../protocol/hooks.md`, 2.2 and
  4.3).

## 5. The owners' sign-off

- **A project setting:** kind 4's `temper_dir_sign_off`, a boolean, off
  by default, taken at the next start as kind 4's members are
  (`../landing-settings.md`, 1.1).
- **On, a covered change** (4) also waits for one approval on the forge
  by a person holding the project's owner role, counted as requirement
  4 counts one (`change/gates.md`, 2): role owner, one person, `clean`,
  so temper's own update before the check keeps it. An owner's request
  for changes at such a head repairs, as a rule's approval's does
  (`change/gates.md`, 1); temper's writer, an unmapped account and a
  person of another role count for nothing.
  - **Under the queue,** it is waited for where a covering rule's
    approval is, the change waiting out of its queue while short
    (`queues.md`).
  - **Under the turn,** each covered member's is judged as a rule's
    approval is judged for a member (`../../landing-turn.md`, 1).
- **Not a rule:** it takes no rule's slot and no path set, and holds
  whatever the floor's rules ask.
- **Refused where no owner can sign:** on is refused, naming
  `temper_dir_sign_off`, wherever an enforcing rule asking one owner's
  approval on every branch of the project's writable repositories would
  be: a repository on a playground's forge, or one on a forge where
  none of the project's owners has an account. Off is always good.
- **Off, the floor alone guards `.temper/`:** a rule by path naming
  `.temper/` is how owners ask for a person meanwhile
  (`../../draft/ways-temper-dir.md`, 6).

## 6. Upgrades

- **The store:** `TemperRevision` is a new family of the forge's,
  registered as `../../protocol/store.md`, 2 registers each, and no
  version moves. A store an earlier build wrote holds no row of it, and
  each project with a home is read at the engine's first start.
- **Kind 4** gains `temper_dir_sign_off`, an added field with its
  default, off (`AGENTS.md`, 2).
- **The change's row** gains a reason of its `Repair`, the check's
  fault with its bounded words, appended to the union, and the fields 4
  keeps, each added with its default; the store's family keeps its
  version and the plan its fingerprints derive converts a row
  (`AGENTS.md`, 2). Only a covered change's fault writes the reason. An
  older build refuses to open a store holding one, naming its value
  type and count, before writing: going back past this build then takes
  the previous generation, as a listing's link does
  (`../../protocol/temper-dir.md`, 8).
- **No wire moves:** nothing new crosses between the engine, its
  workers and their agents, and the clients' wire carries nothing new
  (8).
- **dev.fabrig.ai** needs no step by hand: fabrig's `.temper/` holds
  hooks alone, so its record says it holds no temper's method and none
  of its changes is covered (4).

## 7. Testing

The forge domain's focused tests, over the fake forge, with fuzzy where
it says; a story crossing the root in temper's world.

- **The read:** read at adoption, at an engine's start and at a move
  touching `.temper/`, unknown or behind, and not at a move touching
  nothing of it; a trigger in flight read once more; an answer for an
  older observation or a former home dropped; a failed read keeps the
  revision and records `refused`, a good one after it clears it; the
  same files keep the number; a tree past the bounds is a fault.
- **The queue:** a covered change, not up to date under a rule asking
  neither, is updated and then judged; a fault repairs with its words;
  a base that moves after a pass is judged again; a change touching
  nothing of `.temper/` reads nothing past its comparison; a read that
  fails waits; fabrig's own hooks-only change lands as it does today.
- **The turn:** a covered member's candidate judged; a fault there holds
  the covered members and lands the rest; a member's own fault repairs
  before it joins.
- **The sign-off:** off waits for nothing; on waits for an owner's
  approval at the head or one a clean update carried; an owner's
  request for changes repairs; on is refused for a playground's forge.
- **Fuzzy:** triggers and answers in any order, with failures, delays
  and restarts: the record is the judgement of the latest observation
  taken, `number` never goes back, and a dropped answer changes nothing.

## 8. Later

- **Seen by a person:** the revision and its refusal on the clients'
  wire and the pages, and the check and the sign-off on a change's
  panel, with the pages' pass (`../../draft/ways.md`, section 12).
  Until then a change repairing on the check's fault shows on its panel
  at `gating`, as one waiting for its gates: the clients' wire names no
  repair's reason for it (`../../client/pages/task.md`, the change's
  panel), and a wrong one (CI, a gate, a review) would mislead.
- **The sign-off among a landing's kept sign-offs**
  (`change/gates.md`, 2).
- **A rename out of `.temper/`:** both forges' comparisons give a
  renamed file at its new path alone, so a change that only moves a
  file out of `.temper/` touches nothing of it by its paths; the read
  after its landing judges the tree, and the last good revision holds.
