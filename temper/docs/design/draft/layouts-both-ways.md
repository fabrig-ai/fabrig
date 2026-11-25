# layouts-both-ways: one conversion, either way, for the store and every wire

Draft, 2026-11-17, from a session with the user. They asked for "some elegant way to extend what we're doing to new->old flows and not only to old->new flow", since the store's problem and the wires' "are really the same one". This draft is the data's half: how a value or a message written at one layout is read or written at another. [archive/codec-shape.md](archive/codec-shape.md) is the code's half. The figures are the survey's, on main at `a525e539f7` (`temper/docs/development/codec-survey-2026-11-17.md`).

Where they differ, this draft wins over skein's codec versions, patches and bridges (`skein/docs/design/data/codec/README.md`, 5; `skein/docs/design/data/codec/patches.md`; `skein/docs/design/data/codec/bridges.md`; `skein/docs/design/data/codec/command.md`, 2, 3, 5 and 6) and over its layouts' derivation (`skein/docs/design/data/codec/layouts.md`, 2). It also wins over jig's store layouts and going back (`jig/docs/design/protocol/store/layouts.md`, 1; `jig/docs/design/protocol/store/opening-rehearsal.md`, 1) and over the redeploys draft's versions and bridges (`temper/docs/design/draft/archive/redeploys.md`, 3 and 4). That holds until its goals fold it into them.

## 1. In one page

**One idea.** A value's layout is named by its fingerprint. A reader at another layout converts it by a plan derived from two schema texts: the text it was written at, and the reader's. The store does this today, from old to new. This draft makes it work either way, and carries it to every wire.

**Today** (section 2) there are two mechanisms for one job:

- **The store** converts bytes by plans derived from texts its registry holds, old to new only.
- **The wires** move numbered versions at every landing that changes a layout, keep the old versions' texts in the repository, and bridge them both ways with written patches run over generic values.
  - jig's client wire went from version 19 to 30 between 11-10 and 11-17.
  - The repository holds 43 kept texts and 23 patches, and about 8,000 golden files for the patches alone (survey, 1).

**The ideal:**

- **Layouts by fingerprint** (3). Every family's version freezes at its current number, as the store's registered families' already have. Fingerprints name layouts on every wire, as layout ids do in the store.
- **Plans between any two texts, in two modes** (4). A plan converts from the layout the bytes are at to the layout the reader wants, with no notion of older or newer.
  - **Read** skips what the target lacks.
  - **Keep** refuses a value that would lose what the target lacks, unless that part is at its default or marked droppable.
- **Texts travel with the data or with the peer** (5). The store's registry holds every layout a value is at. A peer sends its texts when a connection opens and the fingerprints differ. No build and no repository keeps an old text.
- **One converter per connection** (6). The engine converts for every worker: received messages through Read plans, sent ones through Keep plans. A value the worker's layout can't keep is refused before it is sent, as a typed answer to the domain that sent it.
- **What no plan derives** (7): written converters, and tombstones for dropped fields, kept down to the floor. No code is kept for a layout older than what dev.fabrig.ai runs.
- **The store gains** (8). Going back to the previous build keeps what the new build committed. Old layouts are rewritten in the background. The transcripts' frames join the registry.
- **What goes** (9): version moves, kept texts, patches and their golden files, `.layouts` lists, version ranges, and the patch machine.

## 2. Today

- **The store** (`jig/docs/design/protocol/store/layouts.md`):
  - a registered value's head is a layout id into the store's registry, which holds the texts;
  - the open derives a plan from each held layout to the binary's, and each value is converted as it is read;
  - registered families' versions no longer move;
  - written jobs convert what no plan derives (`jig/docs/design/protocol/store/jobs.md`, 4);
  - outside the registry, the transcripts' frames are converted by the frame patcher, the patch machine's last user in the store.
- **The wires** (survey, 5):

| Wire | Version / floor | Kept texts, patches | Bridged |
|---|---|---|---|
| jig's worker channel | 16 / 16 | 7, 6 | no older version: patch 6 was declared breaking |
| temper's worker payloads | 10 / 5 | 6, 5 | both ways, 5 to 10 |
| smith's charter and result | 5 / 4 | 2, 1 | both ways, 4 to 5 |
| smith's turns | 7 / 7 | 1, 0 | no |
| smith's channel and pieces | 10, build only; pieces 1 | 5, 3 | no: one build |
| jig's client and public wires | 30, 1 | 0, 0 | no: a client reloads |
| temper's client and public wires | 34, 3 | 0, 0 | no: a client reloads |

- **The opening** (`temper/docs/design/draft/archive/redeploys.md`, 3.2): each side states its versions and their fingerprints, the highest version in common is agreed, and a fingerprint mismatch at a common version is refused. A client of another version is refused and reloads (`temper/docs/design/draft/archive/redeploys.md`, 8).
- **The bridges** (`skein/docs/design/data/codec/bridges.md`). The newer side translates (`temper/docs/design/draft/archive/redeploys.md`, 4.1). Up chains each patch forward. Down chains their reverses and checks the round trip. Both run over generic values. A new field goes down at the value the older version assumed, and a new variant is refused.
- **The plans' derivation** (`skein/docs/design/data/codec/layouts.md`, 2):
  - **derived:** added fields with defaults, dropped and moved fields, variants added, widened integers and bounds, options wrapped, versioned heads, held records and pieces;
  - **not derived:** a retype, a reshape, a variant the root's reach drops, a payload changed, a held family on one side only.

  It already has a form whose dropped variants refuse per value.
- **What it costs:**
  - two machines for one job;
  - a version moved by each goal that changes a wire, so concurrent goals collide;
  - patch numbers taken twice (`skein/docs/design/data/codec/command.md`, 6);
  - kept texts and patch golden files that grow with every version.

## 3. Layouts by fingerprint

- **Every family's version freezes** at its current number, as the store's registered families' did (`jig/docs/design/protocol/store/layouts.md`, 1). A versioned record's head keeps that constant, so no byte moves.
- **A layout is its fingerprint:** a value type's layout (`skein/docs/design/data/codec/fingerprints.md`, 1.4).
- **The store is unchanged:** layout ids into the registry, and the registry's texts.
- **A wire's family names its value types,** the records its messages carry, as a store family does (`jig/docs/design/protocol/store/layouts.md`, 2).
- **At a worker's opening,** each side states each wire family's fingerprint and its value types' layouts.
  - Where they are equal, nothing converts. Every part of one build meets this case.
  - Where they differ, the worker sends its texts (5) and the engine derives its plans (6).
- **At a client's hello,** the fingerprints are compared. A client whose layouts differ is refused by name and reloads, as today with numbers. Bridging clients the same way is later (10).
- **`versions` prints each family's fingerprint,** in place of its range and floor.

## 4. Plans in two modes

### 4.1 Between any two texts

A plan converts bytes at a source layout into bytes at a target layout. It is derived from the two texts, matching declarations, fields and variants by their names and former names (`skein/docs/design/data/codec/layouts.md`, 2). It needs no order between them. The store's plan today, from a written layout to the binary's, is one use of it.

### 4.2 The two modes

- **Read:** the target uses the value and passes it on as its own. What the target lacks is skipped: its build doesn't know it, and it can do nothing with it.
- **Keep:** the value must mean the same at the target, because the target stores it or acts on it for the source. What the target lacks may be dropped only where it is at its default, or where the source's schema marks the field `droppable`. Anything else refuses the value, naming the field.

| Between source and target | Read | Keep |
|---|---|---|
| A field only the source has | skipped | dropped at its default, or when marked `droppable`; otherwise the value is refused |
| A field only the target has | the target's default, or the source's tombstone for it (7.2); with neither, no plan | the same |
| A narrower integer or bound at the target | converted where the value fits; otherwise refused | the same |
| A variant only the source has | that value refused | the same |
| A variant only the target has | nothing to do, with tags mapped by name | the same |
| An option at the source, its type at the target | a present value converts; an absent one takes the target's default, or is refused | the same |
| A former name | nothing to do | the same |
| A retype or a reshape | no plan: a written converter (7.3) | the same |

### 4.3 Per value

- **A plan's pass returns the target's bytes or a refusal,** naming the declaration and the field, as a decoder's refusal does.
- **Which values a plan refuses** is known only by running it: a plan that drops a field is still a plan, and refuses only the values that hold something there.
- **One pass over the bytes,** as today (`skein/docs/design/data/codec/layouts.md`, 3), with no generic value.

### 4.4 Where each mode runs

| Where | Source to target | Mode |
|---|---|---|
| The store reads a value | the layout it was written at, to the binary's | Read |
| The store is opened by an older build (8.1) | the newer layout, to the older build's | Keep |
| The engine receives from a worker | the worker's layout, to the engine's | Read |
| The engine sends to a worker | the engine's layout, to the worker's | Keep |

### 4.5 Tests

- **A round trip for every pair:** a value converted from source to target in Keep mode, then back in Read mode, is the value itself.
- **Generated over `every`'s texts** ([archive/codec-shape.md](archive/codec-shape.md), 5), and over pairs the fuzzy suite derives from them by the change kinds of 4.2.
- **Each refusal and each "no plan"** has a case of its own.

### 4.6 Where it lives

`skein-codec`'s `layout` module. Its derivation gains the mode, the cases of a field or a variant on either side, tombstones and `droppable`. Its per-value refusal of dropped variants is where Keep starts. The patch machine goes (9).

## 5. Texts with the data or with the peer

- **The store:** the registry holds the text of every layout a value is at (`jig/docs/design/protocol/store/layouts.md`, 3), as today.
- **A wire:**
  - every build embeds its own families' texts, as it already does for its store families;
  - at the opening, for each family whose fingerprints differ, the worker sends its text;
  - a text is tens of KB, sent once per connection.
- **Nothing old in the repository:** no `schema/kept/`, and no text of a build that isn't running. A layout's text exists where a value or a peer at that layout exists, and nowhere else.
- **Bounded and checked:**
  - the worker's channel is authenticated;
  - a text is bounded in size and parsed by `skein-codec`'s step parser (`skein/docs/design/data/codec/run-time.md`, 1);
  - a plan is bounded by its texts (`skein/docs/design/data/codec/layouts.md`, 5).

  A text that doesn't parse, or a value type the wire carries that has no plan (6.4), refuses the opening by name.

## 6. One converter per connection

### 6.1 The engine converts

- **What the engine receives** from a worker runs through a Read plan, from the worker's layout to its own, then through its own decoder.
- **What it sends** is encoded at its own layout, then runs through a Keep plan to the worker's.
- **The worker, and the agent its build pins** (`temper/docs/design/draft/archive/redeploys.md`, 4.4), speak their own layouts alone, and know nothing of other builds.
- **Why the engine:** it is the one side that holds every connection, and a redeploy restarts it first (`temper/docs/design/draft/archive/redeploys.md`, 7.1). The plans need no age, so a worker newer than the engine converts the same way. A redeploy's order costs refusals, never correctness.

### 6.2 A refused value

- **A typed answer to the domain that sent it,** before anything is written to the channel: the message is refused for that worker, naming the declaration and the field.
- **The domain decides.** An assignment waits for a worker that can hold it, as an assignment that can't go down waits today (`temper/docs/design/draft/archive/redeploys.md`, 4.3). The other messages' answers are designed by their wires' owners, in the goal that moves their wire.

### 6.3 At the opening

- **Plans are derived for every value type the wire carries,** both ways, and cached by the pair of fingerprints, so a fleet on one build derives once.
- **A value type with no plan, and no written converter (7.3),** refuses the opening, naming the wire, the value type and the change. The worker is drained and restarted on the new build (`temper/docs/design/draft/archive/redeploys.md`, 6.1).

## 7. What no plan derives, and the floor

### 7.1 Written jobs

The store's written jobs convert at open what no plan derives (`jig/docs/design/protocol/store/jobs.md`, 4), as today.

### 7.2 Tombstones

- **A field dropped from a schema stays in its text** as `dropped <name>: <type> = <default>`. A plan toward a layout that still has the field can then write a value there, in either mode.
- **A tombstone takes no bytes** at its own layout, and its generated code holds nothing for it.

### 7.3 Written converters

- **For a retype or a reshape on a wire:** a pair of functions in the owner's crate, for one value type, named by both layouts' fingerprints, run where the plan would be. One direction may refuse.
- **For the store:** a written job, as 7.1.

### 7.4 The floor

- **fabrig keeps no code for a layout older than what dev.fabrig.ai runs,** as store-floor holds for the store (`temper/docs/design/draft/archive/store-floor.md`, 6). A written job, a written converter or a tombstone goes once dev.fabrig.ai's store and every part it runs are at or past the layout that made it needed.
- **Removing one is the floor's move:** a worker still at that layout gets no plan and is refused at the opening by name (6.3), as a store below the floor is refused today (`temper/docs/design/draft/archive/store-floor.md`, 3).
- **No version list, no release record:** what dev.fabrig.ai runs is the record. The store keeps its floor row.

## 8. The store, both ways

### 8.1 Going back keeps what was committed

- **Today,** going back after the new build's first commit means restoring the previous generation, and losing what came after (`jig/docs/design/protocol/store/opening-rehearsal.md`, 1).
- **With Keep plans,** an older build that has this draft's derivation opens the newer store:
  - the registry holds the newer layouts' texts;
  - the older build derives Keep plans from them to its own layouts;
  - one pass checks that every value at a newer layout converts;
  - then it serves, converting as it reads, and writes at its own layouts.
- **A value that can't be kept** refuses the going back, naming the value type and the count, before anything is written. The previous generation is then the way back, as today.

### 8.2 Old layouts rewritten in the background

- **After an upgrade, a background job** rewrites every value at a layout other than the binary's, a page at a time, in ordinary commits, while the store serves. It puts what a settling job would (`jig/docs/design/protocol/store/jobs.md`, 5), without holding the open.
- **The next open retires each layout no value holds** (`jig/docs/design/protocol/store/layouts.md`, 5.4).
- So plans at open and conversions at read stay few, and the floor (7.4) moves without a stored value left behind.

### 8.3 The transcripts' frames join

A frame names its layout as a registered value does, with its texts in the registry, in place of the frame patcher (`jig/docs/design/protocol/store/layouts.md`, 1). The patch machine then has no user in the store.

## 9. What goes

- **Version moves:** every family's version is frozen (3). The gate's check that a changed layout moved its version (`temper/docs/design/draft/archive/redeploys.md`, 3.1) goes with them.
- **Kept texts:** the 43 files under `schema/kept/`.
- **Patches,** 23, and their golden directories (survey, 1):
  - jig's worker channel: 5,000 files;
  - smith's channel: 1,514;
  - temper's worker payloads: 1,280;
  - smith's charter: 108;
  - the fixtures' patches.
- **The patch machine:**
  - `skein-codec`'s `patch` and `bridge` modules;
  - the command's patch writing and renumbering (`skein/docs/design/data/codec/command.md`, 2 and 6);
  - their designs (`skein/docs/design/data/codec/patches.md`, `skein/docs/design/data/codec/bridges.md`).
- **`.layouts` lists and version ranges:** the `.layouts` lists, the ranges and floors in each `schema/wires`, and the opening's agreement on a highest common version (`temper/docs/design/draft/archive/redeploys.md`, 3.2).
- **The store's committed fingerprints** ([archive/codec-shape.md](archive/codec-shape.md), 6.1).

## 10. Later and open questions

- **Clients bridged the same way:** the engine would convert for a client at another layout, so a client could outlive a redeploy without reloading. Today a client reloads.
- **The host tools' contract,** `contract/host-tools-v*.txt`, is a text contract outside the codecs, with its own versions.
- **The component's own rows and the secrets' store** stay outside the registry, at fixed formats (`jig/docs/design/protocol/store/layouts.md`, 1).
- **smith's channel** joins a worker to its own agent, which is one build, and needs nothing.
- **A plan's cost on a hot path** is measured in the first wire's goal, L3. Equal fingerprints, the common case, cost nothing.
