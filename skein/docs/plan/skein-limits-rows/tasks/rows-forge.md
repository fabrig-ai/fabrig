+++
id = "rows-forge"
part = "rows"
kind = "feature"
needs = ["rows-codec"]
files = [
  "skein/crates/skein-forge-setup/Cargo.toml",
  "skein/crates/skein-forge-setup/src/tests/**",
  "skein/crates/skein-forge-setup/src/host/tests.rs",
  "skein/crates/skein-forge-setup/src/host/connection_tests.rs",
  "skein/crates/skein-forge/Cargo.toml",
  "skein/crates/skein-forge/src/tests.rs",
  "skein/crates/skein-forge/src/forgejo/tests.rs",
  "skein/crates/skein-forge/src/forgejo/tests/**",
  "skein/crates/skein-forge/src/forgejo/plans/tests.rs",
  "skein/crates/skein-forge/src/forgejo/plans/tests/**",
  "skein/crates/skein-forge/src/github/tests/**",
  "skein/docs/plan/skein-limits-rows/plan.md",
  "Cargo.lock",
]
effort = "medium"
minutes = 170
+++

# `skein-forge-setup`'s and `skein-forge`'s folds

## Goal

Fold these two crates' honest groups into table tests with `skein_rows::check`. By the survey, `skein-forge-setup` folds 125 tests into 49 tables, saving 76 processes at a floor of 1.9 ms, and `skein-forge` 73 into 30, saving 43 at 1.7 ms. `skein-forge-setup` goes first: it's the larger gain.

Acceptance:

- Both crates' honest groups are folded by `skein/docs/plan/skein-limits-rows/notes/rows.md`'s rules. What you reach is what the rules let you, more or less than the survey's count, with the rest under *Left apart*.
- Every removed test name is a row's name, every assertion stays, and one table per file bites.
- Every table stays within the 40 ms guide for a unit test.
- `plan.md`, *Before and after*, *rows-forge*, is filled.
- The focused tests of `skein-forge-setup` and `skein-forge` pass.

## Inputs

- `skein/docs/plan/skein-limits-rows/inputs/request.md`, item 2.
- `skein/docs/plan/skein-limits-rows/notes/common.md` and `skein/docs/plan/skein-limits-rows/notes/rows.md`: both bind.
- `skein/docs/design/testing.md`, 6, *A table test's rows*.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process*, and 10.

## Owns and may change

Owns the files in the header, and this task alone folds these two crates. Each crate's `Cargo.toml` gains `skein-rows.workspace = true` under `[dev-dependencies]`, and `Cargo.lock` follows.

Not either crate's product code. Not `skein-forge-setup-world` or `skein-forge-machine-world`: another task's.

Options, each decided:

- **A table in `tests/host.rs` takes in an existing table whose cases have no labels:** give each case a label, `old_name: what it changes`.
- **A fold joins tests from two files** (`forgejo/tests.rs` and `forgejo/tests/removed_files.rs`, say): the table goes in the file with the most of its rows.
- **A table passes 40 ms:** make it two, by what the rows have in common. The eleven-phase `tests/github_reconcile.rs` table and the 23 cases of `tests/steps.rs`'s halted table are the ones to read first.
- **A file has its own helper a body can call** (`refuses_limits`, `silent_deadline`, `released_at`, `remake_script`, `halted`): use it, and make that the body.

## Steps

1. **Fold `skein-forge-setup`.** From `skein/crates/skein-forge-setup/src/`; line ranges are at commit `92d04ee863`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `tests/github_reconcile.rs`:246–289 | Live reconcile phases release secrets on every stop | 11 | phase number |
   | `tests/host.rs`:179–190, 312–357, 652–658, 666–675, 993–1030, 1334–1350 | Unusable host settings are refused naming their cause | 9 | limits, settings, refusal |
   | `tests/create_bounds.rs`:24–27, 32–36, 48–51, 65–68 | Creation fields exactly at size bounds are admitted | 4 | complete creation plan |
   | `tests/create_bounds.rs`:28–31, 37–41, 52–61, 69–72 | Creation fields exceeding size bounds name their place | 4 | plan, expected place |
   | `host/tests.rs`:64–94, 105–115 | Host construction agrees with representability checks on refusal | 5 | limits, settings |
   | `tests/given.rs`:182–185, 192–199 | Organization counts through their configured maximum are admitted | 3 | organization count |
   | `tests/host.rs`:191–207, 530–538 | Missing or invalid admin items stop before connecting | 3 | plan, optional token, stop |
   | `tests/host.rs`:240–250, 450–459, 539–547 | Owner waiting runs stop down at their deadline | 3 | plan, requested item, stop |
   | `tests/steps.rs`:233–254 | Halted live plans report their outstanding stage once | 3 | machine, stage, subject |
   | `tests/host.rs`:10–38 | Incompatible calls and authentication are refused as unexpected | 2 | call, authentication |
   | `tests/host.rs`:219–227, 726–750 | Host run capacity counts writing and reachable runs | 2 | limits, incumbent plans, rejected plan |
   | `tests/host.rs`:385–397, 1438–1448 | Idle shutdown closes once and ignores later requests | 2 | shutdown request, later requests |
   | `tests/host.rs`:473–480, 509–514 | Unanswered connections stop down naming the active step | 2 | plan, owner answers, stop |
   | `tests/host.rs`:481–487, 502–508 | Valid handed tokens connect without asking the owner | 2 | token bytes |
   | `tests/host.rs`:705–712, 1503–1512 | Existing writers make another writing plan stop busy | 2 | incumbent plans, candidate plan |
   | `tests/reachable.rs`:78–83, 366–374 | Page size takes the smaller configured and advertised bound | 2 | prepared machine, cap, request |
   | `tests/reachable.rs`:84–93, 375–383 | Requested page numbers reach the outgoing listing request | 2 | mode setup, page, request |
   | `tests/reachable.rs`:101–108 | Short and empty search pages report no more | 2 | row count, outcome |
   | `tests/reachable.rs`:109–139, 424–454 | Listing results preserve repository identity ownership and privacy | 2 | listing event, reached rows |
   | `tests/reachable.rs`:170–181 | Malformed search replies stop unexpected naming the token | 2 | search event |
   | `tests/reachable.rs`:182–185, 465–471 | Zero advertised page caps stop unexpected naming credentials | 2 | prepared machine, failure |
   | `tests/reachable.rs`:186–192, 455–464 | Listings larger than requested stop unexpected before completion | 2 | prepared machine, listing, failure |
   | `tests/reachable.rs`:193–198, 472–480 | Refused requests name their active credential and stage | 2 | prepared machine, status, failure |
   | `tests/reachable.rs`:199–204, 481–486 | Unanswered requests stop down naming their active credential | 2 | prepared machine, failure |
   | `tests/reachable.rs`:205–227, 487–507 | Wrong reply kinds stop unexpected at each request | 2 | prepared machine, event, failure |
   | `tests/reachable.rs`:237–254, 508–519 | Out of order events stop each picker phase | 2 | invalid phase and event pairs |
   | `tests/reachable.rs`:255–275, 520–540 | Ended pickers ignore every subsequent owner or reply event | 2 | terminal setup, later events |
   | `tests/reachable.rs`:276–293 | Invalid picker construction names the first refused input | 2 | given, page, refusal |
   | `tests/reconcile.rs`:132–152, 249–257 | Nonmatching stored bot identities are rewritten before continuing | 2 | given, stored bytes, continuation |
   | `tests/reconcile.rs`:153–180 | Missing admin tokens stop before making any request | 2 | optional token bytes |
   | `tests/reconcile.rs`:219–230 | Bot markers embedded within larger words are refused | 2 | bot display name |
   | `tests/reconcile.rs`:239–248 | Empty saved tokens enter the token creation path | 2 | token item |
   | `tests/reconcile.rs`:286–293 | Authentication failures cause tokens to be remade completely | 2 | status |
   | `tests/given.rs`:272–279 | Adopted repository counts through their maximum are admitted | 2 | repository count |
   | `tests/given.rs`:200–203, 280–283 | Profile collection counts exceeding limits name their place | 2 | complete given, place |
   | `tests/check.rs`:59–64, 83–89 | Absent stored identity is written before reporting identity | 2 | optional identity bytes |
   | `tests/check.rs`:71–81 | Nonmatching kept identities stop as another API user | 2 | stored identity bytes |
   | `tests/check.rs`:107–119 | Missing tokens stop before requests and name themselves | 2 | phase, subject, optional bytes |
   | `tests/steps.rs`:331–346 | Unavailable loopback listeners skip repository webhook secret setup | 2 | listener address or absence |
   | `tests/take_over.rs`:96–100, 125–129 | Short token pages without deletable rows finish takeover | 2 | token rows |
   | `tests/take_over.rs`:101–110 | Successful token deletions resume listing the same page | 2 | empty or 404 reply |
   | `tests/github_installations.rs`:352–375, 414–433 | Sweep deletion failures preserve their correct diagnostic subject | 2 | given, accepted ids, prefix, reply, expectation |
   | `tests/github_manifest.rs`:254–274 | Manifest generation refuses outputs beyond their configured bounds | 2 | given with constrained bounds |
   | `tests/github_picker.rs`:143–150 | Short repository pages finish with no further page | 2 | row count |
   | `tests/hooks.rs`:171–178 | Successful duplicate deletions resume listing the same page | 2 | empty or 404 reply |
   | `tests/github_host.rs`:369–390, 458–477 | Another GitHub writer stops busy beside a picker | 2 | writer plan, first item, subject |
   | `tests/github_host.rs`:391–415, 478–500 | GitHub owner waits use their forge deadline once | 2 | plan, item, subject, late answer |
   | `host/tests.rs`:52–57, 95–103 | Valid limits remain accepted with heap allocated plans | 2 | limits, settings |
   | `host/connection_tests.rs`:55–78 | Document and empty requests reserve no listing rows | 2 | call |

   Check these two first. **`tests/reachable.rs`'s out-of-order pair** filters by phase today; a table needs its invalid pairs written out, so write out every pair the filter lets through, and give the count before and after. **`tests/host.rs`'s first table** takes in `the_host_refuses_unusable_settings_naming_each`, which has 13 cases, so read its time against the guide.

   Left apart by the survey: `tests/host.rs` 76 (distinct retry, redaction, shutdown, admission and bounds stories; four read `tests/tls/fixtures/root.der`), `host/runtime_tests.rs` 8, `tests/github_states.rs` 16, `tests/github_memory.rs` (a heap ceiling), `tests/create_bounds.rs` line 118 (a heap meter), and the rest of each file above for its own reasons.

2. **Fold `skein-forge`.** From `skein/crates/skein-forge/src/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `tests.rs`:106–282, 424–426 | Answer measurement counts each variant's owned data bytes | 11 | answer, literal byte count |
   | `forgejo/tests/hooks.rs`:10–24, 75–82, 99–108 | Hook writes encode their method target and configuration | 4 | request, method, target, JSON |
   | `forgejo/tests.rs`:60–72, 92–94 | Valid RFC3339 inputs decode to their whole wall seconds | 3 | timestamp, seconds |
   | `forgejo/plans/tests/edges/paths_and_headers.rs`:127–157 | Pull-file pagination derives more from headers or fullness | 3 | body, paths, header, answer |
   | `forgejo/tests.rs`:74–78, 88–90 | Wall times always encode as whole UTC seconds | 2 | wall time, UTC bytes |
   | `forgejo/tests.rs`:123–135 | Login decoding enforces both sides of its string bound | 2 | JSON, result |
   | `forgejo/tests.rs`:137–151 | Body retention keeps bounded bytes and records excess length | 2 | body bytes, expected body |
   | `forgejo/tests.rs`:191–198 | User targets encode exactly within their byte bound | 2 | target limit, result |
   | `forgejo/tests.rs`:250–257, 374–384 | Cutting comment streams counts every remaining unread comment | 2 | request, JSON, cut, count |
   | `forgejo/tests.rs`:431–464 | Raw file paths survive escaped encoding and server routing | 2 | path, target |
   | `forgejo/tests.rs`:399–413, 753–762 | Content ranges decode valid intervals and reject invalid intervals | 2 | header bytes, range |
   | `forgejo/tests.rs`:485–496; `forgejo/tests/repositories.rs`:40–42 | Write encoding refuses text that is invalid UTF8 | 2 | request with invalid text |
   | `forgejo/tests.rs`:578–598; `forgejo/tests/removed_files.rs`:41–61 | Pull-file status spellings decode into their typed file statuses | 2 | JSON, file row |
   | `forgejo/tests/hooks.rs`:58–65, 110–117 | Hook deletion encodes no body and needs no decoder | 2 | request, target |
   | `forgejo/tests/hooks.rs`:33–39, 126–131 | Hook answers retain the URL from their configuration | 2 | request, JSON, hook |
   | `forgejo/tests/delivery.rs`:113–125 | Non-numeric delivery numbers are refused naming their field | 2 | delivery JSON |
   | `forgejo/tests/pin.rs`:60–78 | Forgejo version lines match only the pinned release | 2 | line template, bool |
   | `forgejo/tests/pin.rs`:80–93 | Runner version lines match only the pinned release | 2 | line template, bool |
   | `forgejo/plans/tests/job_resolution.rs`:47–55 | Checks resolve their linked jobs independently of check state | 2 | wire state, typed check |
   | `forgejo/plans/tests/calls.rs`:447–455 | Body revisions match their literal big-endian digest goldens | 2 | body, timestamp, revision |
   | `forgejo/plans/tests/calls.rs`:435–445 | Read plans emit escaped targets for their supplied inputs | 2 | read operation, target |
   | `forgejo/plans/tests/calls.rs`:501–531 | Repeated comparison paths obey distinct-file and answer-byte bounds | 2 | bounds, result |
   | `forgejo/plans/tests/calls/turn.rs`:385–410 | Commit messages fit exactly within their answer-byte bound | 2 | answer limit, result |
   | `forgejo/plans/tests.rs`:54–72 | API page-cap discovery accepts positive caps and refuses zero | 2 | cap, terminal result |
   | `forgejo/plans/tests/calls.rs`:604–615 | Write failures classify whether their exchange has been seen | 2 | failure, error |
   | `tests.rs`:285–307 | Operation measurement counts reviewer IDs and owned string bytes | 2 | op, literal byte count |
   | `tests.rs`:312–350 | Host rendering preserves names and omits both default ports | 2 | address, optional name, bytes |
   | `github/tests/rate_validator.rs`:43–51, 63–68 | Printable bounded etags retain their complete original spelling | 2 | etag bytes |
   | `github/tests/rate_validator.rs`:54–60, 83–89 | Invalid etags never become usable conditional request validators | 2 | header sequence |
   | `github/tests/rate_validator.rs`:22–32, 71–80 | Rate extraction requires all three valid numeric header values | 2 | header sequence |

   Left apart by the survey: most of `forgejo/` and `github/`, for different call sequences, exchange states and wire contracts, many already tables. `github/tests/` holds 217 tests with only 6 in a fold, which is the place to look again.

3. **Fill `plan.md`,** by `notes/rows.md`'s form, one row per module, as the request's table names them: at least `tests::host`, `host`, `tests::reachable`, the other `tests::*` modules, `forgejo` and `github`.

4. **The end of a run.** `cargo fmt`, old forms, citations, and `heavy cargo clippy -p skein-forge-setup -p skein-forge --all-targets -- -D warnings`. Then `heavy cargo nextest run -p skein-forge-setup -p skein-forge`, and read the tables' `PASS` times against the 40 ms guide. Commit and push.

## Report

Tests before and after per crate, tables and rows; the names-removed count against the names-found count; which tables you broke and how they failed; each listed group you left apart and each unlisted one you folded; the out-of-order table's count of pairs before and after. Your implementation and check minutes apart.
