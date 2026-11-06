# debt-27: the plan

From `inputs/request.md`: pay debt items, twenty-seventh round, one task each. The trunk is `debt-27/integration`. No temper client view, no `temper.css`, no `jig-client-view`: fabrig/ui has priority.

## Parts and tasks

| # | Task | Item | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `workspace-list` | skein D041 | | medium, 10 | ten duplicates and five unused skein worlds gone from the root manifest |
| 2 | `dead-api` | skein D040 | | medium, 25 | the dead public items deleted |
| 3 | `utf8-one` | skein D004 | | medium, 20 | `text_is_valid` delegates; `valid_text` gone; measure 25, its floor |
| 4 | `joined-measure` | skein D007 | | high, 55 | `length::Measure` new; the `joined` copies on `concat` |
| 5 | `shell-posix` | skein D042 | | high, 60 | `posix` holds what both backends write alike; at least 200 lines fall |
| 6 | `channel-valid` | jig D011 | | medium, 10 | both sides call `skein_channel::valid` |
| 7 | `smith-goldens` | jig D037 | | medium, 15 | jig's copies gone; `include_bytes!` from `smith/` |
| 8 | `unused-params` | jig D041 | | medium, 20 | four parameters gone; the item stays open at 27, all in `jig-client-view` |
| 9 | `widen-lengths` | jig D043 | | medium, 70 | the sweep; the item stays open at 2, both in `jig-client-view`'s tests |
| 10 | `token-lanes` | smith D020 | | high, 90 | bit 61 gone, `Token::tagged`, the `operations` table gone |

## Critical path

No task needs another: `token-lanes`, ~90 implementer-minutes, then the trunk's end check. Everything else runs beside it.

## What the cut settled

Each item was measured against the trunk first, as the request asks.

- **Two items can't reach their measure here, and stay open.** 27 of jig's D041's 31 and 2 of jig's D043's are in `jig-client-view`, which the request closes. An exclusion in the measure would read 0 and pay nothing, so `unused-params` and `widen-lengths` lower them, and each item says what's left and why. `health_words` has already lost its parameter.
- **jig's D011 needs no skein change.** `skein_channel::valid` exists, and `capacity_bound` is the two copies' body, line for line. Both copies were read only through `.is_some()`, so the item's "what it holds" asks for nothing more.
- **skein's D041 is wider than it says:** ten duplicates, with `skein-fake-forge-http`, and five skein worlds unused in `[workspace.dependencies]`, which its **Paid when** covers. `smith-run-world` and the jig worlds are as unused, but aren't skein's: `workspace-list` lists them under `## Left`.
- **skein's D040:** `forgejo::time` has already gone. The cut rewords `temper/docs/design/client/tools.md`'s mention of `RECORD_VERSION`, since the branch that deletes it stays in its tree. `admit_policy` becomes `pub(crate)` for the one test.
- **skein's D004's floor is 25:** only two sites use `from_utf8` for the answer alone; the rest use the `str`, and `is_utf8`'s body is one. `utf8-one` compares the hand loop with `is_utf8` by a throwaway test before it goes, since a disagreement would change what the codec refuses.
- **skein's D007:** the `Measure` goes in skein-lib, as its **Paid when** says, over a `usize` with `total_u32`. The two `Option<u32>` copies' answers don't move. `files.rs`'s `joined` puts a slash between segments: exempted, not changed, as the request says of code that uses the same word for something else. `join(parts, max)`, twice, has a bound for its job, and stays.
- **skein's D042 changes a design sentence.** `shell.md`, 7.5, said a call the two make alike is written in each; the item asks the opposite, and no draft holds the sentence. The cut rewrote 7.5, with a **Not yet** that `shell-posix` closes. The scan found more than the item's two files: 119 more lines of byte-the-same functions, moved too, or **Paid when** wouldn't hold. Functions that differ in a type or a step stay.
- **macOS can't be compiled here:** no builder is known to hold `aarch64-apple-darwin`, and installing it is the user's, so `shell-posix` compiles it if it's there, otherwise keeps `darwin/`'s edits to deletions and `use` lines, and says so. `macos.yml`, every six hours, is then the first build, and a red there blocks nothing.
- **smith's D020's tags keep their values:** 4 and 8 are bits 62 and 63, so no token moves but the socket lane's. `ROOT_FILE` and `LOCK_FILE` go too, since its **Paid when** says "smith's token tags". `PROGRAM_ROOT` names one operation and stays, which is why `HostIo`'s lane carries no tag. `Token::tagged` refuses a route slot from 2^28 up, so `limits.routes` is refused there, an addition. `kernel_cancel_targets_the_right_layer_operation` pins the table, and is the one assertion in the plan that's rewritten, keeping its rule.
- **`widen-lengths` is one task,** though it has about 220 sites: the request wants one task per item and few builder places, and the sweep only shrinks lines. Its acceptance adds a wider search, since the item's measure misses nested calls.

## Overlaps with drafts beside this one

- `workspace-list` and `hearth-speed`'s `dev-profiles` and `repeats` share the root `Cargo.toml`: the two lists here, profiles there.
- The cut's word in `temper/docs/design/client/tools.md` shares the file with `plainer-panels/help-says-so`, far from its lines.
- `widen-lengths` can't avoid files merged on other trunks: `jig-protocol-store/src/reads/paged.rs`, `jig-domain-core/src/reads/**`, `jig-domain-core/src/runs/**`, `jig-client-protocol-link/src/translate/reads.rs` and `jig-client/src/thread/**` (`chat-reading-c`, `first-usable-model`). Its edits are single expressions.
- `dead-api` and `joined-measure` stay off `chat-reading-c`'s and `sign-in-new-tab`'s `skein-ui-web` and `skein-browser` files; `dead-api` touches only `timer.rs` and `tests/history.rs` of that crate.

Inside this plan: `utf8-one` and `joined-measure` both edit `skein-oauth-issuer/src/documents.rs`, in different functions, and `joined-measure` spells its call with its path so their `use` lines don't meet. `unused-params` and `widen-lengths` share `jig-client-protocol-link` and `jig-domain-people`, in different lines.

## After the plan

Every task ends with fewer lines than it started; `shell-posix`, `joined-measure` and `dead-api` carry the largest falls. The two items left open are the only debt the plan knowingly leaves, and both wait on fabrig/ui leaving `jig-client-view`.
