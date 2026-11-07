+++
id = "rows-worlds"
part = "rows"
kind = "feature"
needs = ["rows-codec", "judge-parses"]
files = [
  "skein/testing/skein-rows/**",
  "skein/tests/forge/machine/Cargo.toml",
  "skein/tests/forge/machine/tests/suite/hand/**",
  "skein/tests/forge/setup/Cargo.toml",
  "skein/tests/forge/setup/tests/suite/*.rs",
  "skein/crates/skein-forge-client/Cargo.toml",
  "skein/crates/skein-forge-client/src/tests.rs",
  "skein/crates/skein-forge-client/src/tests/**",
  "skein/crates/skein-oauth-accounts/Cargo.toml",
  "skein/crates/skein-oauth-accounts/src/tests.rs",
  "skein/crates/skein-oauth-accounts/src/accounts_tests.rs",
  "skein/crates/skein-oauth-accounts/src/shared_tests.rs",
  "skein/crates/skein-oauth/Cargo.toml",
  "skein/crates/skein-oauth/src/tests.rs",
  "skein/crates/skein-oauth/src/pkce.rs",
  "skein/crates/skein-oauth/src/machine_tests.rs",
  "skein/crates/skein-oauth/src/machine_tests/openid.rs",
  "skein/crates/skein-http/Cargo.toml",
  "skein/crates/skein-http/src/tests/**",
  "skein/crates/skein-fake-forge-forgejo/Cargo.toml",
  "skein/crates/skein-fake-forge-forgejo/src/tests/**",
  "skein/crates/skein-fake-forge-domain/Cargo.toml",
  "skein/crates/skein-fake-forge-domain/src/tests/**",
  "skein/crates/skein-fake-forge-github/Cargo.toml",
  "skein/crates/skein-fake-forge-github/src/server.rs",
  "skein/crates/skein-fake-forge-github/src/tests/**",
  "skein/crates/skein-llm/Cargo.toml",
  "skein/crates/skein-llm/src/client/tests.rs",
  "skein/crates/skein-llm-connection/Cargo.toml",
  "skein/crates/skein-llm-connection/src/tests.rs",
  "skein/crates/skein-forge-connection/Cargo.toml",
  "skein/crates/skein-forge-connection/src/limits/tests.rs",
  "skein/crates/skein-channel/Cargo.toml",
  "skein/crates/skein-channel/src/tests.rs",
  "skein/crates/skein-tls/Cargo.toml",
  "skein/crates/skein-tls/src/tests/config.rs",
  "skein/docs/design/testing.status.md",
  "skein/docs/plan/skein-limits-rows/plan.md",
  "Cargo.lock",
]
effort = "medium"
minutes = 210
+++

# The worlds', the clients' and the fakes' folds, and every refusal per limit

## Goal

Fold the rest of the request's table with `skein_rows::check`, where the floors are highest and where the cases are most alike. `skein-forge-machine-world`'s hand tests have the highest floor, and the `valid_refuses_*` families are the very case section 8 names: a refusal per limit. The survey's count here is 291 tests into 88 tables, saving 203 processes.

This task adds `skein_rows::panics`, since many of those families are `#[should_panic]`.

Acceptance:

- `skein_rows::panics` exists as `skein/docs/design/testing.md`, 6 has it, with its test, and its **Not yet** entry is gone.
- These crates' honest groups are folded by `skein/docs/plan/skein-limits-rows/notes/rows.md`'s rules, as this spec departs from them below. What you reach is what the rules let you, more or less than the survey's count, with the rest under *Left apart*.
- Every removed test name is a row's name, every assertion stays, and one table per file bites.
- Every table stays within its guide: 40 ms for a unit test, 0.2 s for `hand`'s in-memory world tests.
- `plan.md`, *Before and after*, *rows-worlds*, is filled.
- The focused tests of every package you changed pass.

## Inputs

- `skein/docs/plan/skein-limits-rows/inputs/request.md`, item 2.
- `skein/docs/plan/skein-limits-rows/notes/common.md` and `skein/docs/plan/skein-limits-rows/notes/rows.md`: both bind.
- `skein/docs/design/testing.md`, 6, *A table test's rows*.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process*, and 10.
- `rows-codec`'s report, if it names anything about the kit.

## Departure from `notes/rows.md`

That note keeps `#[should_panic]` apart. Here a limit's `#[should_panic(expected = ...)]` test folds, because `skein_rows::panics` holds the same check inside a row, and the row still names itself if the constructor stops refusing:

```rust
#[test]
fn invalid_forge_limits_fail_validation_and_construction() {
    skein_rows::check(
        [
            ("valid_refuses_zero_pending", Limits { pending: 0, ..LIMITS }),
            ("valid_refuses_zero_calls", Limits { calls: 0, ..LIMITS }),
        ],
        |limits| {
            assert!(!crate::valid(&limits));
            skein_rows::panics("forge client limits are valid", || {
                let _domain = Domain::new(&limits);
            });
        },
    );
}
```

Any other `#[should_panic]` stays apart unless its neighbours are the same check.

## Owns and may change

Owns the files in the header, and this task alone folds these crates. Each crate's or world's `Cargo.toml` gains `skein-rows.workspace = true` under `[dev-dependencies]`, and `Cargo.lock` follows. In `skein/docs/design/testing.status.md`, remove `6: skein_rows::panics.`, and nothing else there.

Only `skein-rows` gains code: `panics` beside `check`, with no change to `check`. Not any crate's product code. `skein-forge-setup` and `skein-forge` are `rows-forge`'s; `skein-codec` is `rows-codec`'s. `skein/crates/skein-oauth/src/generated/` stays.

Options, each decided:

- **Commit and push after each binary.** A run cut short keeps what it did, and your next run starts from it.
- **A fold touches a file another plan has changed since your base** (`skein-oauth`'s tests with `account-email`, `skein-forge-client` with `limits-fuzzy`): keep their change, and fold around it.
- **A `refuses_limits`-style helper exists:** it's the body. A file with two helpers for two kinds of refusal has two tables.
- **A table passes its guide:** make it two, by what the rows have in common.
- **`skein-fake-forge-github`'s RS256 rows:** with `judge-parses` merged, the fake no longer signs to check. `src/tests/service_app.rs`'s four `app_jwt_route` tests were split only for that cost ("so the RSA checks stay within the step budget"), so read the table's time, and fold them if it fits 40 ms, removing that comment. If it doesn't fit, they stay apart and the comment stays.
- **A crate in step 11 has its `valid_refuses_*` tests differ from each other beyond the limit:** leave those, and fold the rest.

## Steps

1. **The kit.** In `skein/testing/skein-rows/src/lib.rs`:

   ```rust
   /// Holds for a row what `#[should_panic(expected = "...")]` holds for a
   /// test: `call` must panic, with a message holding `expected`.
   #[track_caller]
   pub fn panics(expected: &str, call: impl FnOnce())
   ```

   Its tests in `src/tests.rs`: `panics_returns_when_the_message_holds_what_is_expected`, `panics_fails_when_the_call_returns`, `panics_fails_on_another_message`. Bite by making it accept any panic, and see the third fail.

2. **`skein-forge-machine-world`,** floor 2.6 ms. From `skein/tests/forge/machine/tests/suite/hand/`; line ranges are at commit `92d04ee863`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `github/github_installation_loss.rs`:598–647, 770–799; `close_at` | Closing ready phases discards all remaining private work | 8 | stage |
   | `github/github_installation_loss.rs`:598–647, 770–799; `replace_at` | Replacing ready phases discards all remaining private work | 8 | stage |
   | `github/github_installation_loss.rs`:573–597; deleted and suspended | Committed installation loss discards late minted token values | 4 | mint phase, committed rows |
   | `github/github_installation_loss.rs`:299–322 | Deferred viewer refusals stop once naming their status | 4 | HTTP status |
   | `github/github_start.rs`:284–311 | HTTP start failures back off and restart listing | 4 | phase transcript, status, body |
   | `github/github_start.rs`:480–518 | Replacement discards old startup work across every phase | 4 | phase transcript, token count |
   | `github/github_start.rs`:519–549 | Closing startup phases prevents all subsequent private work | 4 | phase transcript |
   | `github/github_installation_loss.rs`:415–438 | Transient mint responses retry the selected scope directly | 3 | status, response body |
   | `github/github_installation_loss.rs`:549–566 | Private job diagnostics keep both installation tokens hidden | 3 | phase setup |
   | `github/github_renewal.rs`:469–480 | Closing renewal phases discards tokens and pending work | 3 | phase setup |
   | `github/github_renewal.rs`:481–492 | Replacing renewal phases discards tokens and pending work | 3 | phase setup |
   | `github/github_unknown_repository.rs`:325–342 | Closing discovery phases suppresses the pending call terminal | 3 | phase transcript, token expiry |
   | `github/github_installation_loss.rs`:568–572, 583–587 | Unaccepting installations discards late minted token values completely | 2 | mint phase |
   | `github/github_installation_loss.rs`:381–388 | Mint refusals wait before refreshing lost installation eligibility | 2 | status, listing body |
   | `github/github_start.rs`:171–190 | Mint access refusals wait before reading installations again | 2 | HTTP status |
   | `github/github_start.rs`:192–209 | Fresh installation listings suppress minting for ineligible rows | 2 | status, listing body |
   | `github/github_start.rs`:229–244 | Nontransient read refusals report their stage and status | 2 | phase, read kind, statuses |
   | `github/github_start.rs`:352–371 | Invalid minted tokens publish nothing and trigger backoff | 2 | token bytes, expiry |
   | `github/github_renewal.rs`:382–389 | Renewal refusals wait before refreshing lost installation eligibility | 2 | status, listing body |
   | `github/github_renewal.rs`:118–124, 547–554 | Renewal backoff keeps the current full credential usable | 2 | scope setup, credential, deadline |
   | `github/github_renewal.rs`:408–418, 555–563 | Installation loss discards results of already written renewals | 2 | scope setup, mint response |
   | `github/github_unknown_repository.rs`:60–79 | Exhausting installation probes returns missing with incurred cost | 2 | candidate rows, credential counts |
   | `github/github_calls.rs`:125–135, 292–301 | Unaccepted repository installations reject calls without issuing IO | 2 | operations |
   | `github/github_calls.rs`:217–228, 322–332 | Unauthorized calls renew full credentials without replaying requests | 2 | operation, request prefix |
   | `github/github_installations.rs`:185–214 | Installation login bounds accept exactly the permitted width | 2 | width, acceptance |
   | `forgejo/contents.rs`:302–343 | Unresolvable Actions lookups leave checks without job attempts | 2 | response sequence, cost |
   | `forgejo/contents.rs`:146–169, 231–253 | File responses retain requested bytes and report truncation | 2 | path, bound, response, bytes |
   | `forgejo/contents.rs`:421–469, 650–671 | Job log responses retain tails and report truncation | 2 | raw response, log, truncation |
   | `forgejo/free_rate.rs`:59–78 | Incomplete rate headers preserve the last complete window | 2 | response fields |

   Left apart by the survey: `component_heap` and `CallHeap` tests (`github/github_writes_heap.rs`, `forgejo/artifact.rs`, one in `github/puller.rs`, two in `github/github_files.rs`), and most of `github_calls.rs`, `github_unknown_repository.rs`, `github_renewal.rs`, `puller.rs`, `reads.rs` and `writes.rs`, for distinct behaviours.

3. **`skein-forge-client`,** floor 1.5 ms. From `skein/crates/skein-forge-client/src/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `tests.rs`:665–672, 755–937 | Invalid forge limits fail validation and constructor admission | 24 | limits, panic message |
   | `tests/recovery.rs`:231–341, 733–740 | Invalid saved records leave restoration failed and busy | 8 | stored-record sequences |
   | `tests/contracts.rs`:126–159 | Stored record measurements count their variable payload bytes | 6 | stored, bytes |
   | `tests.rs`:205–224, 311–317, 492–517, 544–567 | Fresh reads reject answers inconsistent with their calls | 4 | read, answer, owner |
   | `tests/recovery.rs`:681–732 | Valid saved records restore and admit resource keeping | 4 | live record |
   | `tests/contracts.rs`:225–245 | Answer measurements count fixed slots and variable payloads | 4 | answer, bytes |
   | `tests.rs`:46–53 | Missing provider configuration ends fresh reads exactly once | 2 | provider error |
   | `tests.rs`:750–753, 939–943 | Permitted forge limits pass validation including large bounds | 2 | limits |
   | `tests/contracts.rs`:263–270, 347–353 | Answers within configured admission bounds are forwarded unchanged | 2 | read, answer, byte cap |
   | `tests/contracts.rs`:272–279, 355–361 | Answers exceeding configured admission bounds are refused consistently | 2 | read, answer, byte cap |
   | `tests/budget.rs`:246–273 | Refunds remain capped by the reported forge allowance | 2 | allowance, balances |
   | `tests/outbox.rs`:541–548 | Missing provider configuration fails fresh writes exactly once | 2 | provider error |
   | `tests/outbox.rs`:559–566 | Missing provider configuration retries finds without another write | 2 | provider error |

4. **`skein-oauth-accounts`,** floor 1.6 ms. From `skein/crates/skein-oauth-accounts/src/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `tests.rs`:678–819 | Invalid account limits refuse checking and component construction | 26 | limits |
   | `tests.rs`:667–670, 820–825 | Permitted account limits pass validation including large bounds | 2 | limits |
   | `accounts_tests.rs`:572–587 | Adding accounts within capacity produces no refusal event | 2 | account-number sequence |
   | `accounts_tests.rs`:537–543, 560–570 | Removing refresh interest clears the pending refresh deadline | 2 | earliest time, cancellation times |
   | `shared_tests.rs`:318–324, 335–343 | Lock replies without pending ownership are unlocked immediately | 2 | pending requests, elapsed time |

5. **`skein-fake-forge-forgejo`,** floor 1.8 ms. From `skein/crates/skein-fake-forge-forgejo/src/tests/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `deliveries.rs`:64–86, 101–125 | Delivered hooks preserve their event and payload fields | 4 | hook, body, event, decoded value |
   | `setup_reads.rs`:132–157 | Admin repository reads use setup without domain calls | 3 | private ids, request, document |
   | `response.rs`:237–288 | Suffix ranges return trailing bytes and literal coordinates | 3 | slice, suffix length, body, header |
   | `setup_reads.rs`:211–224 | Config-token setup reads pass through the domain once | 2 | request |
   | `setup_reads.rs`:225–239 | Permission reads return the latest collaborator put value | 2 | grants, permission |
   | `setup_reads.rs`:279–292 | Effective permission takes the stronger direct or team grant | 2 | team access, direct grant, permission |
   | `setup_reads.rs`:301–315 | Permission reads refuse unknown repository or user identities | 2 | request |
   | `setup_reads.rs`:397–404, 411–417 | Scenario grants enforce the configured login byte bound | 2 | login length, result, grant |
   | `deliveries.rs`:151–162 | Delivery endpoints require nonempty host and secret fields | 2 | endpoint configuration |
   | `setup.rs`:124–135, 144–151 | Repository routes refuse tokens lacking required access scope | 2 | scopes, requests |
   | `setup.rs`:136–143, 152–161 | Repository read scopes admit every repository read route | 2 | scope |
   | `creation.rs`:116–130 | File creation refuses missing repository or branch targets | 2 | setup requests, file request |
   | `routes.rs`:262–276 | Invalid fake object spellings decode to no commit | 2 | object ids |

6. **`skein-oauth`,** floor 1.6 ms. From `skein/crates/skein-oauth/src/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `machine_tests.rs`:860–916 | Invalid client limits refuse validation and client construction | 11 | client limits |
   | `tests.rs`:234–239, 394–412, 452–457 | Saved token records round trip with optional fields | 3 | saved token |
   | `tests.rs`:430–450 | Unsupported record versions consistently produce the version error | 3 | encoded bytes |
   | `machine_tests.rs`:850–853, 918–923 | Permitted client limits pass validation including large bounds | 2 | client limits |
   | `machine_tests.rs`:787–811 | Invalid confidential redirects are refused during registration validation | 2 | registration |
   | `machine_tests/openid.rs`:306–321 | OpenID key routes require nonzero key storage capacities | 2 | client limits, route |
   | `tests.rs`:283–300 | Incomplete or trailing record bytes decode as malformed | 2 | encoded bytes |
   | `pkce.rs`:26–35 | PKCE verifiers produce their expected unpadded S256 challenges | 2 | verifier, digest |

7. **`skein-http`,** floor 1.5 ms. From `skein/crates/skein-http/src/tests/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `server/head.rs`:437–475 | Invalid server limits fail validation and constructor admission | 5 | limits, panic message |
   | `request.rs`:33–38, 55–89 | Valid calls encode their expected HTTP request heads | 4 | call, wire |
   | `body.rs`:40–69, 445–458 | Framed response bodies satisfy reads and end reusable | 3 | wire, read, body |
   | `exchange.rs`:344–366 | Invalid client limits fail validation and constructor admission | 3 | limits, panic message |
   | `writer.rs`:124–142 | SSE events encode fields and normalized data lines | 2 | event fields, wire |
   | `server/body.rs`:34–64 | Framed request bodies satisfy reads and end reusable | 2 | wire, read, body |

8. **`skein-fake-forge-domain`,** floor 1.6 ms. From `skein/crates/skein-fake-forge-domain/src/tests/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `promises.rs`:365–389, 404–411 | Merge order requires matching repository item and head decisions | 4 | before and after observations, decisions |
   | `actions.rs`:84–98, 276–297 | Job log reads select bytes by job attempt and step | 3 | job, attempt, step, result |
   | `promises.rs`:116–123, 140–148 | Duplicate client markers breach once within their namespace | 2 | observation stream, subject |
   | `promises.rs`:196–202, 301–326 | Unallowed client item writes breach foreign ownership promises | 2 | observation stream |
   | `promises.rs`:204–209, 346–351 | Moving an unowned branch breaches foreign ownership promises | 2 | branch name, subject |
   | `promises.rs`:446–463 | Object tables distinguish capacity from one additional object | 2 | object count, verdict |
   | `reports.rs`:56–65 | Selected JUnit reports contain only requested test identifiers | 2 | failed ids, selection, XML |

   Check `promises.rs` again by hand: `forced_fails_*` and the other `foreign_fails_*` tests read as streams with one expected verdict, and may fold further.

9. **`skein-forge-setup-world`,** floor 2.5 ms. From `skein/tests/forge/setup/tests/suite/`.

   | File and lines | What all its rows show | Tests | Row data |
   |---|---|---:|---|
   | `host.rs`:202–214, 234–249 | Repository adoption accepts either supported repository identifier form | 2 | repository identifier |
   | `host.rs`:215–232, 250–275 | Missing adoption targets report their supplied repository identifier | 2 | identifier, subject |
   | `create.rs`:159–171 | Missing creation scopes refuse at the repository stage | 2 | admin scopes |
   | `reconcile.rs`:119–130, 181–196 | Unmarked accounts under the bot login are refused | 2 | configuration, account marker |
   | `github.rs`:62–81 | Installation repository pages return rows and pagination facts | 2 | seed, page, rows and more |

   `heap.rs` measures, and `fuzzy/` is another suite: both stay out.

10. **`skein-fake-forge-github`,** floor 1.9 ms. From `skein/crates/skein-fake-forge-github/src/`.

    | File and lines | What all its rows show | Tests | Row data |
    |---|---|---:|---|
    | `tests/write_pulls.rs`:223–263; `tests/write_branches.rs`:96–122; `tests/write_merge.rs`:196–213 | Repository writes outside installation reach never call domain | 3 | request sequence |
    | `server.rs`:622–631 | HTTP intake enforces its request body byte bound | 2 | body, status, retained-byte range |

    And `tests/service_app.rs`'s `app_jwt_route` tests, by the option above. `src/tests/app.rs` keeps its six tests, which hold the fake's checks.

11. **Every other refusal per limit.** These crates aren't in the request's table, so their floors aren't known: give `unknown` in `plan.md`, with the tests removed. They're section 8's own example, and most are `refuses_limits` wrappers, though `skein-tls`'s and `skein-channel`'s bodies are longer, so read those first. Fold each file's `valid_refuses_*` family into one table, or a table per body: `skein-llm/src/client/tests.rs` 15, `skein-llm-connection/src/tests.rs` 11, `skein-forge-connection/src/limits/tests.rs` 7, `skein-tls/src/tests/config.rs` 6, `skein-channel/src/tests.rs` 5. `skein-llm-connection`'s one `#[should_panic]` uses `panics` if it's one of them.

12. **Fill `plan.md`,** by `notes/rows.md`'s form, as you finish each binary. The modules as the request's table names them: `hand::github::*`, `hand::forgejo::*`, `machine_tests`, `tests`, `tests::server`.

13. **The end of a run.** `cargo fmt`, old forms, citations, and `heavy cargo clippy` with a `-p` for every package you changed, `--all-targets -- -D warnings`. Then `heavy cargo nextest run` with the same `-p` list, and read the tables' `PASS` times against their guides. `skein-tls` is also checked alone: `heavy cargo check -p skein-tls`. Commit and push.

## Report

Tests before and after per binary, tables and rows; the names-removed count against the names-found count; which tables you broke and how they failed; each listed group you left apart and each unlisted one you folded; whether `app_jwt_route`'s tests fit one table, and the time you read. Your implementation and check minutes apart.
