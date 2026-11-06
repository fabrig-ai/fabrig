# debt-29: the plan

From `inputs/request.md`: pay debt items, twenty-ninth round, one task each. The trunk is `debt-29/integration`. No temper client view, no `temper.css`, no `jig-client-view`: fabrig/ui has priority.

This is the first batch. D006 and D048 follow.

## Parts and tasks

| # | Task | Item | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `first-demand` | review 1, skein D040's rest | | medium, 30 | six functions gone that return `Next` whatever they're given |
| 2 | `admit-policy` | review 1, skein D040's rest | | medium, 20 | `admit_policy` gone; `admit` test-only; no `Option<bool>` |
| 3 | `concat-name` | review 4, skein D007's rest | | medium, 10 | the alias and the expect gone |
| 4 | `shell-posix` | review 5, skein D042's rest | | medium, 40 | the six alike functions written once |
| 5 | `smith-tags` | review 3, smith D020's rest | | medium, 30 | four hand-set tags through `Token::tagged`; the departure gone |
| 6 | `fake-config` | skein D033 | | medium, 60 | `Config::TEST`; the item deleted |

## Done by the cut, in the plan's first commit

- **Review 2, jig's D043:** the text names all four sites and the measure sees them. The code stays, in `jig-client-view`.
- **Review 4's third part:** `forgejo/server/files.rs`'s `joined` is recorded in `skein/docs/design/lib.status.md`.
- **jig's D026:** `kept`, with `jig/docs/design/client/domain/contract.md`, 3.4 saying the check is the wire's own.
- **jig's D041:** stays open. All 27 sites are in `jig-client-view`, and the item says so as what blocks it.

## What the cut settled

- **The review's finding 1 is wider than three functions.** Forgejo's three have three twins: GitHub's `delivery::first`, which the design's bullet names too, and `skein-fake-forge-http`'s two. They go together. The status's "No service does" is wrong: `skein-forge-setup/src/host/connection.rs` calls one, and it ships.
- **`admit` goes test-only as well as `admit_policy`.** The review named `admit_policy`, but `Component::admit` above it has no caller outside its crate's tests. So the `Option<bool>` exists for tests alone, which is D040's own case.
- **Review 3's doubt is settled:** agent tokens are `Id::token`, `slot << 32 | generation`, so they stay below 2^60 until slot 2^28. `smith-tags` also converts `PROGRAM_ROOT`, whose exemption the `debt-27` cut recorded: the request asks to pay the tags, not to record them.
- **Review 5:** the six move, and `shell.md`'s text stays. `usage` keeps each system's `resources`, since `ru_maxrss` differs in unit, and `open_signals` stays each system's. The gate's clippy builds the Darwin target, so `macos.yml` isn't this check.
- **jig's D026 stays where it is.** The request stops at a stored format and at a changed outcome: people refusing the case takes a project in each `Operation` and a new variant `people-v14.schema` doesn't have, and moving the check moves which refusal a malformed ask gets. The item itself names this second way.
- **skein's D033:** temper's five `Config` literals stay. The item names skein's tests, and a branch stays in its tree.

## Overlaps with drafts beside this one

- `concat-name` and `ui-repo-link` (`provider-public`, merged on its trunk): `skein-oauth-issuer/src/documents.rs`. Unavoidable: the site is there, and it's two lines.
- `first-demand` crosses into temper for two lines of `temper-fake-git`, value only, since one branch removes the functions and every caller. No draft beside this one touches them.

## Critical path

As it stands, `fake-config`, at about 60 implementer-minutes: nothing in this batch waits for anything.
