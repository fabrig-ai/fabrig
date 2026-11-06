# debt-29: the plan

From `inputs/request.md`: pay debt items, twenty-ninth round, one task each. The trunk is `debt-29/integration`. No temper client view, no `temper.css`, no `jig-client-view`: fabrig/ui has priority.

## Parts and tasks

| # | Task | Item | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `first-demand` | review 1, skein D040's rest | | medium, 30 | six functions gone that return `Next` whatever they're given |
| 2 | `admit-policy` | review 1, skein D040's rest | | medium, 20 | `admit_policy` gone; `admit` test-only; no `Option<bool>` |
| 3 | `concat-name` | review 4, skein D007's rest | | medium, 10 | the alias and the expect gone |
| 4 | `shell-posix` | review 5, skein D042's rest | | medium, 40 | the six alike functions written once |
| 5 | `smith-tags` | review 3, smith D020's rest | | medium, 30 | four hand-set tags through `Token::tagged`; the departure gone |
| 6 | `fake-config` | skein D033 | | medium, 60 | `Config::TEST`; the item deleted |
| 7 | `link-readers-temper` | jig D006 | | medium, 45 | temper names `jig_domain_link`: 202 uses in 54 files |
| 8 | `link-values-accounts` | jig D006 | 7 | medium, 30 | accounts' re-exports gone: 109 uses |
| 9 | `link-values-fleet` | jig D006 | 7, 8 | medium, 45 | fleet's re-exports and `core::Problem` gone: 216 uses |
| 10 | `link-values-host` | jig D006 | 7, 9 | medium, 70 | the hub's re-exports gone: 390 uses; the item deleted |
| 11 | `oauth-baseline` | skein D048 | | high, 60 | a digest per story, ignored and temporary |
| 12 | `oauth-one-owner` | skein D048, step 1 | 11 | high, 120 | sign-in a sub-owner; one `Process`, one `Client` |
| 13 | `oauth-one-driver` | skein D048, step 2 | 12 | high, 120 | one protocol driver |
| 14 | `oauth-one-restart` | skein D048, step 3 | 12 | high, 40 | one restart wrapper |
| 15 | `oauth-close` | skein D048 | 13, 14 | medium, 30 | the baseline gone, literal facts kept; the item narrowed, open |

About 750 implementer-minutes.

## Done by the cut, in the plan's commits

- **Review 2, jig's D043:** the text names all four sites and the measure sees them. The code stays, in `jig-client-view`.
- **Review 4's third part:** `forgejo/server/files.rs`'s `joined` is recorded in `skein/docs/design/lib.status.md`.
- **jig's D026:** `kept`, with `jig/docs/design/client/domain/contract.md`, 3.4 saying the check is the wire's own.
- **jig's D041:** stays open. All 27 sites are in `jig-client-view`, and the item says so as what blocks it.
- **jig's D006's rule** is in `jig/docs/design/domain/root.md`, 2, with a **Not yet** in its status that `link-values-host` removes.

## Critical path

`oauth-baseline`, `oauth-one-owner`, `oauth-one-driver`, `oauth-close`: about 330 implementer-minutes, then the trunk's end check. The D006 chain is the next, at about 190.

## Left out of the plan

- **skein's D048, steps 4 to 7, the rows.** About 270 to 405 more minutes and level in lines, for an item of low value, in a round the request wants small. The harder reason is that `PasteThenListener`'s hold can't honestly become fields: its two tiers use different predicates, so unifying them retimes the story, and about five settings true in one row would be a renamed flag. It takes a step script interpreted in each tier, or a recorded departure, which is fabrig/debt's to agree. `oauth-close` writes all of this into the item, which stays open with its **Paid when**.
- **jig's D041 and D043's code:** every site is in `jig-client-view`.

## What the cut settled

- **The review's finding 1 is wider than three functions.** Forgejo's three have three twins: GitHub's `delivery::first`, which the design's bullet names too, and `skein-fake-forge-http`'s two. They go together. The status's "No service does" is wrong: `skein-forge-setup/src/host/connection.rs` calls one, and it ships.
- **`admit` goes test-only as well as `admit_policy`.** The review named `admit_policy`, but `Component::admit` above it has no caller outside its crate's tests. So the `Option<bool>` exists for tests alone, which is D040's own case.
- **Review 3's doubt is settled:** agent tokens are `Id::token`, `slot << 32 | generation`, so they stay below 2^60 until slot 2^28. `smith-tags` also converts `PROGRAM_ROOT`, whose exemption the `debt-27` cut recorded: the request asks to pay the tags, not to record them.
- **Review 5:** the six move, and `shell.md`'s text stays. `usage` keeps each system's `resources`, since `ru_maxrss` differs in unit, and `open_signals` stays each system's. `shell.md` has clippy check the Darwin target on our machines, and `shell-posix` runs it. `macos.yml`'s runs weren't read: `gh` isn't on this machine, and a red there is a finding after landing.
- **jig's D026 stays where it is.** The request stops at a stored format and at a changed outcome: people refusing the case takes a project in each `Operation` and a new variant `people-v14.schema` doesn't have, and moving the check moves which refusal a malformed ask gets. The item itself names this second way.
- **skein's D033:** temper's five `Config` literals stay. The item names skein's tests, and a branch stays in its tree.
- **jig's D006 is cut by tree, then by re-exporter,** against "one task each". Temper goes first, so the re-exports can go, and each of jig's three drops two lines and lowers the measure, so a run cut short leaves the item true. It can't end with fewer lines, as the item says: each is capped on its net. Beyond the item's six lines, `jig-domain-core`'s `pub use fleet::WorkerProblem as Problem` is the same path one crate further, so it goes too. `jig-boundary`'s exact dependency lists gain `jig-domain-link`, which `root.md`, 2 calls a shared leaf, like skein-lib. Readers are compiled by clippy and `jig-boundary` alone: a path holds no behaviour, so their focused tests are the trunk's end check's.
- **skein's D048's baseline is a temporary instrument, not a tier's test.** About 270 runs wouldn't fit one focused test, so its tests are `#[ignore]`d, run by name, and deleted by `oauth-close`. It moves no test out of a tier and raises no budget, so the cut read this as no budget question; a word from fabrig/debt would settle it if it reads otherwise. It's tracked because an untracked file wouldn't cross task worktrees. A digest is comparable across commits since the trace holds no harness-chosen number. Each task re-runs it after fab's merge, so a kit's move isn't taken for its own.
- **What the baseline leaves:** literal facts for the sign-in stories that were `drop(run(..))`, at most 60 lines, as the request's "reduced to the rows that still bite". The digests themselves don't bite on a rule; they trip on any kit change.
- **`oauth-one-driver` has a pre-approved smaller landing,** the server half at about 95 lines, because the driver holds this part's sharpest risk: wire seeds and trace order. `oauth-close` says what's left if it's taken.

## Overlaps with drafts beside this one

- `concat-name` and `ui-repo-link` (`provider-public`, merged on its trunk): `skein-oauth-issuer/src/documents.rs`. Unavoidable: the site is there, and it's two lines.
- `first-demand` crosses into temper for two lines of `temper-fake-git`, value only, since one branch removes the functions and every caller. No draft beside this one touches them.
- `link-readers-temper` and `model-catalogue-1` (`one-account-offer`, `stock-seven`): `temper-service/src/tests/first_model.rs`. And `chat-reading-c` (`reading-worlds`, merged): `temper/tests/client/src/front.rs`. `use` lines and paths alone.
- `link-values-*` and `bundle-loading` (`gzip-serve`, merged): five files of `jig-test-service/src/tests/`. And `model-check` (`person-task`, merged): `jig-domain-core/src/documents/chip_phase.rs` and `vocabulary.rs`. `use` lines and paths alone, and they run late in their chain.
- `oauth-*`: no draft beside this one touches `skein/tests/oauth-accounts`.
- `Cargo.lock`: the D006 tasks' new dependency lines meet `bundle-loading` (`gzip-skein`) there. Cargo regenerates it.

The chain waits as the code does: `link-values-*` need temper off the re-exports, and `oauth-*` need the baseline, then the one owner.
