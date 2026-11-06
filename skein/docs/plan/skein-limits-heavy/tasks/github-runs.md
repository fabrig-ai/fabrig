+++
id = "github-runs"
part = "github"
kind = "feature"
needs = []
files = [
  "skein/tests/forge/github/src/tests.rs",
  "skein/tests/forge/github/src/cases.rs",
  "skein/tests/forge/github/src/runner.rs",
  "skein/tests/forge/github/src/fake.rs",
  "skein/tests/forge/github/src/mask.rs",
  "skein/tests/forge/github/src/wire.rs",
  "skein/docs/plan/skein-limits-heavy/figures/github-runs.md",
]
effort = "medium"
minutes = 100
+++

# `github-conformance`: run as much as a test checks

## Goal

`github-conformance`'s `tests` is 29 tests and 4.59 CPU s, against 0.16 for `forgejo-conformance`'s 47. Most of it is ten whole runs of `cases::run` (about 325 ms each) and eleven of `cases::queue`, by tests that read one fact. Explain what a whole run costs and fix that, then make each test run what it checks. "Fails only the fact and continues" is shown once, in one whole run carrying several faults.

Acceptance:

- `github_conformance_holds_every_fact_on_the_fake` and `github_conformance_fake_requests_replay_through_the_live_guard` run whole and assert what they assert today.
- A new `github_conformance_faults_fail_only_their_facts_and_the_run_continues` runs `cases::run` whole with the faults of step 4, and holds every assertion of the two tests it replaces.
- `github_conformance_missing_delivery_fails_only_its_fact` and `github_conformance_null_merged_commit_fails_the_fact_and_continues` go, named under *Tests that went* with that holder.
- Every row of `github_conformance_response_framing_records_the_exchange_and_continues` and `github_conformance_epoch_refusals_and_non_json_fail_only_the_fact` stays, with its outcome, transcript and cleanup assertions, on a run of its own group alone.
- `github_conformance_discovers_the_organization_before_minting` holds its assertions on `cases::queue` alone.
- `github_conformance_queue_refusals_fail_one_fact` keeps every row, each fact's exact reason and every cleanup assertion.
- A whole run makes the same requests in the same order: the count `github_conformance_holds_every_fact_on_the_fake` reads is the same before and after, and the figures give it.
- `skein/docs/plan/skein-limits-heavy/figures/github-runs.md` gives before and after, explains a whole run's cost by function in its first lines, and has its *Found and not fixed*.
- The next cause's gain is under about 0.05 CPU s.

## Inputs

- `skein/docs/plan/skein-limits-heavy/inputs/request.md`, item 2 and its table's first row.
- `skein/docs/plan/skein-limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `skein/docs/plan/skein-limits-heavy/plan.md`, *Belonging*.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process* and *A slow test is understood, never hidden*, and 8.1.
- `skein/docs/design/forge-github.md`, 13: the facts. It's read by `github_conformance_names_every_fact_of_forge_github_13`, and doesn't change.
- `skein/tests/forge/github/README.md`, its last paragraph: what the focused tests hold. It stays true.

## Owns and may change

Owns the files in the header. This task alone changes `Runner`'s test fields and `cases`' visibility.

May change, for a cause a profile puts there, at the size the cause needs:

- `skein/testing/skein-forge-conformance/**`, shared with `forgejo-conformance`: its tests run too.
- `skein-fake-forge-github`, `skein-fake-forge-domain`, `skein-fake-peers`, `skein-forge`, `skein-sim`: small, no change of behaviour, with a focused test where it's a product change.

Doesn't change: `cases::NAMES`, the order of facts or requests in a whole run, `live.rs`, the README's promises, anything the common note leaves alone (`skein-json`'s product code among them).

Options, each decided:

- **The cost is RS256:** `Runner::sign` signs at each `start`, and the fake verifies on App requests. A test that wants no renewal doesn't need another key. If verification on the fake repeats for the same token, remember the last token verified, in the fake, if the refusals of an expired or wrong JWT stay exact. Build settings stay. ring's own cost goes under *Found and not fixed*.
- **The cost is the secret scan or the mask** (`write::check`, `Mask`, on every exchange): fix a repeated scan or a copy in `skein-forge-conformance` or `mask.rs`. The masks still mask every category, and the leak refusal stays.
- **The cost is JSON parsing:** one parse where a case parses the same response several times (`response.json()?` in a case's chain) is fine in `cases.rs` if no fact's meaning changes. A cost inside `skein-json` goes under *Found and not fixed*.
- **The cost is the simulator's passes** (`World::client`, `World::wait`, `pass`): fix a loop that spins with nothing to do. Every fact's clock reading stays.
- **The cost is writing transcripts:** tests read them, so they stay. Write a file once where it's written several times.
- **A `logs` fact needs state only an earlier group makes,** so it doesn't hold when `logs` runs alone: that row asserts every `logs` fact that holds alone, a comment names the one left out, and the whole-faults test still holds it. Say so in the figures.
- **A fault of step 4 changes another's outcome:** run that fault in a test of its own group with its assertions there, and say so. The faults test keeps at least one fault in each of `pulls`, `repositories` and `logs`.
- **The refusal rows are still the largest cost after the whole run's causes are fixed:** let one run carry one refusal per fact, `plan_refusal` becoming a list, if each row's fact and reason stay exact and a failure still names its row. If a refusal in `enqueue-pending-check-protected` changes what `queue-merged-no-merge-commit-sha` reads (they share a pull), keep those rows apart. If the grouping needs more than that, keep eleven runs and put the gain under *Found and not fixed*.

## Steps

1. **Profile,** with `-p github-conformance --lib` and `tests::NAME`: `github_conformance_holds_every_fact_on_the_fake` (a whole run), `github_conformance_names_a_token_expiring_within_30_days` (`start` alone, the floor about thirty runs pay), `github_conformance_queue_refusals_fail_one_fact` and `github_conformance_masks_secrets_and_ids` (no runner). Starting points, to check and not findings: RS256 in `Runner::sign`; `write::check` and `Mask::{headers, body}` on every exchange in `Runner::ask`; `windows(4).position` on the wire; a `Json::parse` per `response.json()`; the fake's `pass` loops; a file written per case in `flush`.
2. **Fix a whole run's causes,** largest first, re-measuring after each. `forgejo-conformance` shares `skein-forge-conformance`: if the cause is there, it gains too.
3. **Let `Runner` carry several response faults.** `response_override: Option<(&'static str, &'static str, Vec<u8>)>` becomes `response_overrides: Vec<(&'static str, &'static str, Vec<u8>)>`, still `#[cfg(test)]`; `exchange` takes out the entry matching `(self.case, target)`. Its readers are `runner.rs` and `tests.rs` alone.
4. **Write the whole-faults test,** one `cases::run`, in the shape of `github_conformance_missing_delivery_fails_only_its_fact`:

   ```rust
   #[test]
   fn github_conformance_faults_fail_only_their_facts_and_the_run_continues() {
       let out = Temp::new("faults");
       let mut r = runner(&out.0);
       let Target::Fake(w) = &mut r.target else { panic!("fake target") };
       w.null_next_merged_pull_commit = true;
       r.start().expect("start fake run");
       r.response_overrides = vec![
           ("log-delivery-whole", "/app/hook/deliveries?per_page=1", EMPTY_LISTING.to_vec()),
           ("log-lists-failed-deliveries", "/app/hook/deliveries?per_page=100", INVALID_LENGTH.to_vec()),
           ("issues-since-epoch-empty", EPOCH_LISTING, epoch_response(503, "[]")),
       ];
       let run = cases::run(&mut r);
       r.complete(run).expect("faults permit later cases and cleanup");
       // Every fault was exercised; facts.len() == NAMES.len(); each faulted
       // fact has its outcome; signatures-push-gh013 is unchecked; every other
       // fact holds; the merge fact's line is in facts.txt; cleanup, journal,
       // tokens and mints are empty; scan finds no secret.
   }
   ```

   Its outcomes are the old tests': `Fails("input absent: delivery id")`, `Fails("wire: invalid response length")`, `Fails("wire observation differs from the fact")` and `Fails("merged pull plan failed: InvalidAnswer")`. Then remove the two tests it replaces.
5. **Narrow the rest.** Make `pulls` and `logs` `pub(crate)` if you need them. Move the `issues-since-epoch-empty` observation out of `repositories` into `pub(crate) fn issues_since_epoch(r)`, called from the same place, computing its `token` and `listing` as `repositories` does (they make no request). Then:
   - `..._response_framing_...`: each row runs `start`, `cases::logs`, `complete`. Its assertions stay, but `facts.len()` reads the facts `logs` records and "every other fact holds" reads those.
   - `..._epoch_refusals_...`: each row runs `start`, `cases::issues_since_epoch`, `complete`, and `facts.len()` is 1. Its other assertions stay.
   - `..._discovers_the_organization_before_minting`: `cases::queue` in place of `cases::run`. `rule(.., queue = true, ..)` is called from `queue` alone (`queue` and `queue_dequeued`), so its `in_queue` check still reads every merge-queue ruleset a run makes.
6. **Bite:** the faults test is new, so break it twice by hand. Make `observation` return the error in place of recording the fact, so the run stops at its first response fault, and see the test fail for that. Then take one override out of its list, and see its "exercised" check or that fact's outcome fail. Restore each.
7. **Write the figures.** For `..._faults_...`, the tests it replaces go under *Tests that went*.
8. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p github-conformance --all-targets -- -D warnings` with any other package you changed, and the focused tests of those packages (`forgejo-conformance` too if `skein-forge-conformance` changed). Commit and push.

## Report

What a whole run pays for, the before and after, the causes found and not fixed, and any product change with the test that holds it. How many runs of `cases::run` and of `cases::queue` are left, and which options you took. Your implementation and check minutes apart.
