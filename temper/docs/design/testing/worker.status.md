# worker: status

2026-11-21. Where the tree's code lags or departs from [worker.md](worker.md).

- **Not yet:** 2, and 4's checkout world: `temper-worker-inspect` doesn't exist, the checkout has no reads for it, and the checkout world calls no shared check [in flight: change 11585].
- **Not yet:** 2.1, 2.3's trash row, 2.4: no `inspect_trash`, and the git component has no read of its sweep [in flight: change 11610].
- **Not yet:** 3.1, 3.2: the domain-face fake cuts no operation partway and keeps no capacity and no packs; only a check out refuses an index lock [in flight: change 11586].
- **Not yet:** 3.1, 3.2, and 4's checkout world: the checkout world injects neither fault, keeps no fault ledger and sweeps nothing [in flight: change 11608].
- **Not yet:** 3.3, and 4's git protocol world: its io has no path that refuses every removal, and nothing calls the trash's inspection or sweeps it [in flight: change 11610].
- **Not yet:** 3.4, and 4's domain world: no fake builder, no busy placement while build places are full, and no inspection or sweep in the worker's domain world [in flight: change 11609].
- **Not yet:** 5: `growing_packs_trigger_packing` doesn't exist [in flight: change 11587].
- **Not yet:** 5: `an_unremovable_trash_entry_holds_back_no_other` doesn't exist [in flight: change 11610].
