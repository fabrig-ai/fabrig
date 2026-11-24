# layouts-both-ways: execution

2026-11-18. How [layouts-both-ways.md](layouts-both-ways.md) gets built. The user approved both drafts on 11-18, to run as two milestones, one per draft, on the lean design template. This draft is the second milestone, its seven goals below, made once [codec-shape.execution.md](codec-shape.execution.md)'s has ended. Each goal writes its own design from the draft as it runs, and lands whole, with its own tests, fixtures, workspace entries and gate rules. Plans use the focused and fuzzy suites alone.

## 1. The goals

| Goal | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| L1, plans between any two texts: Read and Keep, a field or a variant on either side, tombstones and `droppable` in the grammar, per-value refusals, the round trips over `every` and the fuzzy suite's pairs | 4, 7.2 | none | none: skein's suites | landed |
| L2, the store both ways: going back with Keep plans and its check at open, the background rewrite as a settling job, the transcripts' frames registered and the frame patcher removed | 8 | L1 | going back on a copy of dev.fabrig.ai's store, and a rewrite on dev.fabrig.ai, after landing | landed; its walk after landing not yet run |
| L3, the worker wires (jig's channel, temper's payloads, smith's charter and turns): versions frozen, fingerprints and texts at the opening, the engine converting both ways, typed refusals, written converters; their kept texts, patches and patch golden files removed | 3, 5, 6, 7.3, 9 | L1 | a redeploy of dev.fabrig.ai with a worker wire change, the workers staying, after landing | landed, by goal 13844 (`jig/docs/design/protocol/channel/layouts.md` and `wires.md`; `temper/docs/design/protocol/wires.md`, 3 to 6 and 10); its own landing's redeploy is every part together (3.2 there); `smith-transcript`'s kept texts still seed the registry for legacy frames after L2 replaced the frame patcher; the remaining patch history is L6's; its walk not yet run |
| L4, the client and public wires, the views and the cursor: versions frozen, fingerprints compared at the hello, refused by name; their `.layouts` lists removed | 3, 9 | L1 | a client's reload after a redeploy, after landing | landed |
| L5, generated translation for the wires: codec-shape's maps files (its 4.2) for the worker wires, then the client wires on both ends, each wire's translation generated into its owner's crate | codec-shape 4 | L3, L4 | none | not started |
| L6, the patch machine removed: `skein-codec`'s `patch` and `bridge` modules, the command's patch writing and renumbering, the fixture owners that only test them | 9 | L2, L3 | none | not started |
| L7, the words: skein's codec versions, patches, bridges, command and layouts; jig's store layouts and opening; the redeploys draft's versions and bridges; `AGENTS.md` | all | L1 to L6 | none: docs | not started |

## 2. Acceptance

- **L1:** each row of the draft's 4.2 has a test in each mode; the round trip holds over `every`'s pairs and the fuzzy suite's; the store's plans today derive unchanged in Read mode.
- **L2:** an older build opens a store a newer one wrote, keeps every value, and refuses by value type and count where one can't be kept; after the rewrite, the registry holds the binary's layouts alone.
- **L3:** an engine and a worker at different layouts of every worker wire exchange every message kind, both ways, with no kept text in the repository; a value the worker can't keep is refused to its domain before the channel.
- **L4:** no family's version moves on a layout change; a client at another layout is refused by name.
- **L5:** each wire owner's hand-written translation holds only records marked `by hand` and its converters.
- **L6:** no `patches/` directory and no `schema/kept/` directory remain.

## 3. Order

- **L1 first,** in `skein-codec`'s layout module alone.
- **L2, L3 and L4 side by side** after L1: the store, the worker wires and the client wires touch different owners.
- **L5 after L3 and L4,** so each wire's maps are written once, against its frozen version.
- **L6 after L2 and L3,** the last users of patches and bridges.
- **The words close the milestone.**
