# jig-world-cost-2: the plan

jig's focused suite reads about 80 CPU s against its 30. This plan repairs what a new profile of `main` found, in the order of `skein/docs/foundation/testing-strategy.md`, 8.1. No budget or build setting changes, and no story loses what it checks. It won't reach 30 alone and isn't asked to: what's left is read after landing.

## Parts and tasks

One task per item of `inputs/request.md`. Each may land short. At most two run at once, by `needs`: items 1, 3 and 5 in one chain, 2 then 4 in the other.

| Item | Part and task | Needs | What it does |
|---|---|---|---|
| 1 | `world-setup` | | Six repeats in a world's setup go: the channel kit's second parse, a held client's view, `Truth`'s whole image, the server's limits, a rebuilt rules domain and a restart's second copy |
| 2 | `naming-row` | | A durable row keeps its typed key; the referee prints it on a failure |
| 3 | `store-sweeps` | `world-setup` | Seven store sweeps move under `fuzzy::`; focused keeps cuts chosen for why |
| 4 | `protocol-polling` | `naming-row` | A wire is asked only when it has work; a message's head is parsed once; a reader reads only what's new |
| 5 | `admission-checks` | `store-sweeps` | A due agent's readiness asks the cheaper checks first; no stored answer |

## Critical path

`world-setup` (90 minutes), `store-sweeps` (60), `admission-checks` (60): about 210 minutes. The other chain, `naming-row` (25) then `protocol-polling` (60), takes about 85.

## Sweeps

Item 3's table. **No existing fuzzy test holds any of the seven,** read at this cut:

- `fuzzy::patching::seeded_patch_cuts_keep_one_whole_modeled_generation` draws one cut per seed, 64 in all, from a first patch only. That's neither every cut nor a drop, and its own comment calls the focused sweep the exhaustive one.
- `fuzzy::crashes::random_commits_and_crashes_keep_the_referees_promises` cuts commits, drawn, with no first start, stray, patch or switch.
- `fuzzy::frames::random_fixture_frames_match_patched_bytes_or_unread` decodes frames, with no file cut.
- `fuzzy::readers` reads, with no cut of a start or a patch.
- `jig-protocol-store`'s step test `a_drop_copy_and_switch_cut_anywhere_leave_one_whole_generation` is focused, in a lower world, and checks no records. It stays, and doesn't hold the world's sweep.

So each sweep moves whole, keeping its name and every check.

| Test today (`jig/tests/store/tests/suite`) | Focused test now, and its chosen cuts | The whole sweep now |
|---|---|---|
| `upgrades::patching::drop_copy_and_switch_recover_after_every_mutation` | `upgrades::patching::drop_copy_and_switch_recover_at_each_step_s_first_mutation`. The first `Rename`: the drop begins, with the old generation leaving. The first `Remove`: the drop is half done. The first `Make`: the copy begins. The last `Rename` and the `Sync` after it: either side of the switch | moved: `fuzzy::upgrades::drop_copy_and_switch_recover_after_every_mutation` |
| `upgrades::check::cuts_across_check_and_switch_recover_whole_maps` | `upgrades::check::cuts_at_the_seal_the_check_and_the_switch_recover_whole_maps`. The seal's sync: the source still opens. The next operation, which changes nothing: a read of the check itself. The switch's `Rename`. Past the switch's sync: the copy is active, and the source's build refuses it as ahead. Asserts both endings seen | moved: `fuzzy::upgrades::cuts_across_check_and_switch_recover_whole_maps` |
| `generations::strays_cut_anywhere_are_gone_at_the_next_start` | `generations::strays_cut_at_their_first_and_last_removal_are_gone_at_the_next_start`. The first `Remove`: every stray still there. The last `Remove`: one left | moved: `fuzzy::generations::strays_cut_anywhere_are_gone_at_the_next_start` |
| `patching::every_patch_write_cut_recovers_all_records_and_no_stray` | `patching::a_patch_cut_at_its_copy_its_pages_and_its_switch_recovers_all_records_and_no_stray`. `patches::cuts`' first: the copy's first directory. Its middle: among the copy and its patch pages, between two whole steps. Its last two: the switch's `Rename` and its `Sync` | moved: `fuzzy::patching::every_patch_write_cut_recovers_all_records_and_no_stray` |
| `generations::a_first_start_cut_anywhere_is_whole_at_the_next_start` | `generations::a_first_start_cut_at_its_first_directory_and_around_its_switch_is_whole_at_the_next_start`. The first `Make`: nothing of the generation is whole. The last `Rename` and the `Sync` after it: either side of `active` | moved: `fuzzy::generations::a_first_start_cut_anywhere_is_whole_at_the_next_start` |
| `upgrades::frames::every_file_cut_in_a_patched_read_preserves_all_files` | `upgrades::frames::a_patched_read_cut_at_its_first_and_last_file_operation_preserves_all_files`. For each of `kinds()`: cut 0, before the read touches a file, and the last, with the read finished. The closing `waiting` block stays with each kind: it's no sweep | moved: `fuzzy::upgrades::every_file_cut_in_a_patched_read_preserves_all_files` |
| `pending_switch::each_cut_around_the_switch_leaves_one_generation_whole` | `pending_switch::a_cut_at_the_switch_s_first_mutation_and_around_its_rename_leaves_one_generation_whole`. The switch's first mutation, and the `Rename`: the old generation is active and patches again. The `Sync` after it: the new is active. Asserts both seen | moved: `fuzzy::pending_switch::each_cut_around_the_switch_leaves_one_generation_whole` |

`jig/tests/store`'s commit cuts in `crashes.rs` stay in focused: they aren't in the request, and testing.md, 9 says so. The task reports any row that lands differently.

## What setup repeats

Item 1, from a read-only survey and checked by reading. **A test runs in its own process, so a `OnceLock` pays once per test.** Nothing here is measured, and the survey couldn't split costs by reading, so the steps' order is by what's plainly unused, not by a figure.

| Share named | What it does and what reads it | The repair |
|---|---|---|
| `engine_configuration`, 7.8% of a channel test | `store_limits` and `store_names` each parse `ENGINE` whole, for its limits and its model names | One parse holds both. The service's startup parses its own, as a real start does |
| `fixture_with`, 8.7% of a channel test | The channel's `NativeClient` builds both phrasebooks and a view, and `hold_for_account_sign_in` stops it rendering from its first step | Built when first needed |
| `Story::configured`, 15% of a client-protocol test | Mostly `World::new`, below. Also `Truth::after(rows.clone())`, of which only people's `Answer` rows are compared, and `server_limits()` builds a whole service profile, twice | `Truth` keeps those rows, from a borrow; the limits are asked for once |
| `World::new`, 24% of a core-world test | `effect_fixture` builds a third rules domain to clone its rules; budgets are collected for a branch most worlds don't take; the oracle's close copies its configuration and `restart_with` copies it back | Each goes |

**Left, and why:** `World::new`'s largest parts are by the survey's reading the real first start, with its commits through the codecs, and the observer's rebuild at each commit. The start is what the design says a world does, and the observer's rebuild is D034's row feed, which the request leaves out. So this item may save much less than the 24% it names, and the queue's record will say. `Core::new`'s second model table is product code that `assert_catalogue` and `fleet_needs::configured` read in between, and isn't asked for.

## Decisions

- **Item 2's `Row` names `jig_domain_core::Key` in the referee,** which names no domain crate today. The crate links it, the request asks for a typed key, and a type-erased key would have to print to compare, since `Snapshot` derives `Eq`. A connector row stays printed: `jig-conformance` can't link a root's key. It's made only for a row that names something.
- **Item 2's trap:** `require` takes its message by value, so `format!("...{}", naming.row)` would print every row anyway once a project has ended. `deletions.rs` builds it only on a failure.
- **Item 4 uses `Wire::work_pending`,** already in the kit, so nothing changes in the repaired `stream.rs`. `Net::pending` already skips attempts that draw nothing, on the same grounds. `Net`'s public items don't change, so the joint fuzzy world gains with no edit.
- **Item 4 keeps a quirk:** a broken keyed socket can use up an armed upload cut today, and "every cut stays" holds it, so the early pass over a broken socket applies only with none armed.
- **Item 5 has no stored answer.** `run_admission` reads the task's row and numbers, the charter's version and settings, the accounts' states and sources, the fleet's hosts and slots, workspace credentials, authority's policy and rules, and the clock, through `run_wall` and `run_now`. A grant or a credential expires with no input naming it, and these are written across the core and its children, so nothing visibly clears a stored answer. The request's condition can't be shown from the code.
- **Item 5's cheaper checks are exact.** `context_ready` wants only "not a wait", and `denied` makes anything but an `Account` finding a hold. So `check_run` with no accounts answers "hold" under any choice, and an unusable credential answers "wait" under any choice. Only then does `run_choice` run, and an `Ok` still goes through the full check. That's in the spec, per exit, with a test whose rows hold both `run_admission` and the new answer.
- **`Domain::quiescent` stays as it is.** `finish_owed` walks projections and marks and `due_ready` is free with nothing due, so neither order is plainly cheaper.
- **Item 3 changes design, and this cut wrote it:** `jig/docs/design/domain/testing.md`, 9 and `jig/docs/design/protocol/store.md`, 14, with **Not yet** entries that `store-sweeps` removes, a stated departure.

## Overlaps

Read against `Drafts beside this one` at this cut; the drafts still being cut list no files yet, so what they'll touch is unknown.

- `limits-worlds` is in integration with `jig/testing/jig-conformance/**` and `jig/testing/jig-core-kit/src/conformance.rs`. `naming-row` touches `referee.rs`, `referee/deletions.rs` and `core/naming.rs` there. It's small, and doesn't touch `jig-core-kit/src/conformance.rs`. Unavoidable: it's where the item is.
- `model-catalogue-3`, being cut, may touch `jig/crates/jig-domain-core/src/settings/`, where `admission-checks` changes `charters.rs` in one arm. That task goes last in its chain anyway, as the request orders.
- `jig/docs/design/domain/testing.md` is shared by every plan writing about worlds: `store-sweeps`' design is one paragraph of 9, and its status entry one line.
