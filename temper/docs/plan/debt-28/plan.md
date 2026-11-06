# debt-28: the plan

From `inputs/request.md`: pay debt items, twenty-eighth round, one task each. The trunk is `debt-28/integration`. No temper client view, no `temper.css`, no `jig-client-view`: fabrig/ui has priority.

## Parts and tasks

| # | Task | Item | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `inner-allow` | jig D020 | | medium, 15 | `inner_allow` an old form at 0 |
| 2 | `role-sets` | skein D047 | | medium, 50 | 20 named predicates behind one exhaustive table; the slices gone |
| 3 | `shell-keeper` | skein D045 | | medium, 60 | `skein_shell::keeper`; `close_startup_fd` at the root; about 200 call sites follow |
| 4 | `request-states` | skein D025 | | high, 70 | the table stays in lib, its entry in three lifecycles |
| 5 | `program-grammar` | jig D006, half | | medium, 25 | the grammar's copies gone; the item stays open for its re-exports |
| 6 | `file-readers-skein` | skein D049 | | medium, 60 | skein's readers on `file_layer::step` |
| 7 | `file-readers-smith` | skein D049 | | medium, 75 | smith's |
| 8 | `file-readers-jig` | skein D049 | | medium, 30 | jig's |
| 9 | `file-readers-temper` | skein D049 | | medium, 25 | temper's |
| 10 | `file-readers-hearth` | skein D049 | | medium, 10 | hearth's |
| 11 | `store-states` | skein D049 | | high, 120 | `Store` one enum |
| 12 | `opening-states` | skein D049 | | high, 60 | `Opening` one enum |
| 13 | `file-thin-gone` | skein D049 | 6 to 12 | medium, 15 | the thin forms gone; item deleted |
| 14 | `io-limits` | skein D033 | 6 | medium, 100 | `Limits::TEST` the base of 47 literals; at least 300 lines fewer |
| 15 | `client-limits` | skein D033 | 14 | medium, 45 | `ClientLimits::TEST`; labelled rows; the item rewritten, open for one type |

About 760 implementer-minutes.

## Critical path

`file-readers-skein`, `io-limits`, `client-limits`: about 205 implementer-minutes, then the trunk's end check. `store-states` then `file-thin-gone` is the next, at about 135.

## Left out of the plan

- **jig's D041 and D043:** 27 and 2, every site in `jig-client-view`, which the request closes twice. The request's own rule covers it: "if the honest shape isn't reachable, keep the item open". Each item already says why.
- **skein's D048:** 8 to 11 hours by the survey, not its Size M, with little to hold behaviour still. The item now holds the survey's order of work. The first three steps would remove about 450 lines.
- **jig's D026:** People's `Operation` can't hold the case, so paying it means a project in each operation, in about 22 files, and a stored enum's new variant. The item now holds the facts and a second way, `kept`.
- **jig's D006's re-exports:** 916 uses in 180 files, not its 59, adding lines rather than removing them. `program-grammar` pays the other half.

The request says "if one turns out bigger, stop and ask fabrig/debt rather than grow it". These are that question; the report carries it.

## What the cut settled

- **skein's D047 is wider than its measure.** 8 more slices sit across lines, where the measure can't see them, and `==` chains test the same sets. **Paid when** says "each set", so they go too. It can't end with fewer lines, since an exhaustive match outweighs the short sites it replaces: its cap is on what it adds. One table behind all the predicates keeps that to about 150 lines where a match per predicate would be about 900.
- **skein's D047's drifted copies stay drifted.** `focusable` holds `Main` and `mapping.rs`'s `tabindex` set doesn't; merging them changes behaviour, so `role-sets` names both for fabrig/debt.
- **skein's D045 is wider and narrower than it says.** Wider: 24 names have no production reader, not the 3 it counts, and **Paid when** says "the calls only the test harness's keeper needs". Narrower: `close_keeper_fd` closes descriptors on startup failures in `hearth`, `temper` and `temper-forge-setup-shell`, so those callers get `close_startup_fd` at the root. It can't end with fewer lines: a change of path removes no body. Design: `skein/docs/design/shell.md`, 7.5.
- **skein's D025 stays in lib.** A move adds a crate and removes nothing, and jig's step crates may hold nothing generic. `skein/docs/design/lib.md`, 12, says so, and names the three lifecycles. `anchor` stays an `Option`: a value learned once, no phase, where renaming would pay nothing.
- **jig's D006's grammar needs no checked copy.** The item allows one, since `jig-client-domain` depends on skein-lib alone. But `SourceName::new` is `skein_lib::Name::<32>::new(.., LowercaseDigitsHyphen)`, which the client domain can call directly.
- **skein's D049 is split by tree,** against "one task each": a branch stays in its tree, the readers can move in any order, and the thin forms need them all gone. Its readers can't end with fewer lines either: each gains a stored `Limits`, about 100 lines in all against the 40 the thin forms give back. `hearth-service-builder` is no longer being migrated apart. "Keep no clock" is narrower than the item says: seven `up` sites, and `up` reads none. Design: `skein/docs/design/io.md`, 5.
- **skein's D033's measure could never fall.** It counts signatures and struct updates, and its own payback leaves it near 64. `io-limits` and `client-limits` work to a whole-literal measure. `Limits::TEST` already exists, so it's retuned, not doubled, and every site keeps its values. Tokenizer limits drop out with their reason: 46 of 59 are one line, and no base fits. `fn limits()` drops out: it's what `testing-strategy.md`, 10, asks for. The item stays open for `skein_fake_forge_domain::Config`.

## Overlaps with drafts beside this one

- `role-sets` and `chat-reading-c` (`log-surface`, `text-reading`, `page-reading`, `log-busy`): `skein-ui/src/tree.rs` and `builder.rs`, `skein-ui-text/src/render.rs`, `skein-ui-web/src/page.rs`. Unavoidable: the item lives there. That trunk adds no new role or set.
- `shell-keeper` and `box-small` (`link-hours`), `test-speed` (`tier-threads`): `temper-box/tests/real_loop.rs`, `temper/crates/temper/tests/fake_backends/box_link.rs`. One path each. Unavoidable, since every caller follows in one branch.
- `file-readers-temper` and `test-speed` (`ctl-stopped`): none by file, on purpose: `Directory` keeps its signature so `temper-client-shell-cli/src/host.rs` isn't touched.
- `io-limits` and `chat-reading-c`, `sign-in-new-tab`: `skein/tests/browser/**` and `skein-fake-person-browser`. Its two sites there stay whole, so no edit.

`io-limits` waits for `file-readers-skein` since both edit `skein/tests/io` and `skein/tests/kv`, and `client-limits` for `io-limits` since the item it rewrites says what `io-limits` did.
