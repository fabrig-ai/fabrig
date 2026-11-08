+++
id = "branch"
part = "branch"
kind = "feature"
needs = []
files = ["temper/scripts/look-refresh.sh", "temper/tools/journeys/tests/suite/look_refresh.rs", "temper/docs/design/client/journeys.status.md", "temper/docs/design/client/tui/journeys.status.md"]
effort = "medium"
minutes = 35
+++

# look-refresh.sh keeps `tui-requests` as it keeps `look-requests`

## Goal

The TUI's look requests stack on a branch of their own, `tui-requests`, apart from the browser's `look-requests`. `temper/scripts/look-refresh.sh` takes either branch. Checked out on `look-requests` or `tui-requests`, it does for that branch what it does today for `look-requests`:

- its lease is that branch's remote-tracking ref, `refs/remotes/origin/<branch>`;
- its requests are that branch's commits over `origin/main`;
- it pushes that branch.

Any other checkout is still refused, and the refusal names both branches. Nothing else in its behaviour changes: the same arguments, checks, trailer rules, exit statuses and output, with the branch's name where the output names `look-requests` today.

Acceptance:

- On `tui-requests`, the script rebases, drops and pushes as it does on `look-requests`, under `tui-requests`'s own lease. Its last line is `pushed tui-requests at <short>`.
- On any other branch, and on a detached HEAD, it exits 2 before fetching, and its stderr says `look-refresh: check out look-requests or tui-requests first`.
- `look_refresh::the_requests_follow_main` runs a row per branch, `look-requests` and `tui-requests`, and both pass.
- `look_refresh::an_unsuitable_checkout_is_refused_before_fetching` asserts the refusal's message on `main` and on a detached HEAD, and passes.
- Every other story of `look_refresh.rs` passes unchanged on `look-requests`.
- Each new check bites (Steps, 3).
- The two **Not yet** entries this plan opened are gone (Owns and may change).

## Inputs

- The request: `temper/docs/plan/tui-requests-branch/inputs/request.md`. Read it whole; it's short. Its `For every task of this plan` section binds this task. There's no `notes/common.md`.
- The design, which this plan wrote:
  - `temper/docs/design/client/journeys.md`, 7, the bullet *Kept on `main`*. Its last sentence says the script takes `tui-requests` too.
  - `temper/docs/design/client/tui/journeys.md`, 10, the bullet *Requests stack*.
- The status entries this task closes:
  - `temper/docs/design/client/journeys.status.md`, the **Not yet** bullet for 7;
  - `temper/docs/design/client/tui/journeys.status.md`, the **Not yet** entry for 10.
- The code: `temper/scripts/look-refresh.sh`, all of it, and `temper/tools/journeys/tests/suite/look_refresh.rs`: `Repositories` and its methods, `the_requests_follow_main` and `an_unsuitable_checkout_is_refused_before_fetching`.

Departures from the implementer's text, and why:

- **You edit two design status files,** though `docs/design/` is otherwise not yours. The plan wrote their **Not yet** entries for exactly this change. Those entries go when the code lands, in the same branch.
- **Effort is medium.** The change is a branch name made a variable, and one row of an existing story.

## Owns and may change

Owns:

- `temper/scripts/look-refresh.sh`, all of it. The edits are the ones in Steps, 1, and nothing else.
- `temper/tools/journeys/tests/suite/look_refresh.rs`:
  - `Repositories`: its `branch` field and constructor, and the methods that name `look-requests` today (`new`, `publish_requests`, `advance_main`, `assert_pushed`);
  - `the_requests_follow_main` and `an_unsuitable_checkout_is_refused_before_fetching`.

  Every other story stays as it is, `look-requests` literals included.
- `temper/docs/design/client/tui/journeys.status.md`:
  - remove the last line, `- **Not yet:** 10: ...`, with the blank line before it;
  - turn `Otherwise no known implementation gap.` back into `No known implementation gap.`.

  Keep the date and the rest of that paragraph as they are, so the file is again the one on `main`, line for line.
- `temper/docs/design/client/journeys.status.md`: remove the **Not yet** bullet `  - 7: ...`. Leave its date (`2026-11-08`) and everything else.

Keep to these lines. Other drafts are in flight beside this one:

- `tui-link-colours` edits other sections of `client/tui/journeys.md` and its status file;
- `ui-ask-drop` edits `client/journeys.status.md` and every status file.

Don't tidy lines this task doesn't own.

Not yours, and why:

- `temper/docs/design/**/*.md` other than the two status edits above: the plan has written the design and the draft's status (`temper/docs/design/draft/look-requests.status.md`).
- `temper/AGENTS.md` and `temper/docs/design/draft/look-requests.md`: the user's guidance and a draft. They still name `look-requests` alone, and they stay as they are.
- The branches themselves: never push to, fetch into, or run the script against `origin/look-requests` or `origin/tui-requests`. Every test runs in its own scratch repositories.

Pre-approved options:

- **The branch check:** by default a `case $branch in look-requests | tui-requests) ;; *) refuse ... ;; esac` after the `symbolic-ref`. Both refusals, no symbolic ref and a third branch, give the same message.
- **The fixture's branch:** by default `Repositories` gains `branch: &'static str`. A new `Repositories::on(name, branch)` makes it, and `Repositories::new(name)` becomes `Self::on(name, "look-requests")`, so the other stories' calls don't change. The four methods named above use `self.branch` where they name `look-requests` today. Changing `new`'s signature at every call site instead is also fine; say so in the report.
- **The row's scratch name:** each row gets a fresh `Repositories`, never one shared, labelled `look-refresh-follow-{branch}` or alike, so two rows never meet in one directory.
- **The row's request names:** the existing names (`tighter-cards`, `calmer-banner`) do for both rows. The script doesn't read the `tui-` prefix, and `request_in`'s pages hang on those names.
- **If `cargo fmt` or clippy wants a change** in an owned item, take it.

## Steps

1. **The script.** In `temper/scripts/look-refresh.sh`:
   - the header comment says it keeps the requests of whichever of `look-requests` and `tui-requests` is checked out. It cites `docs/design/client/journeys.md`, section 7, and `docs/design/client/tui/journeys.md`, section 10;
   - after `branch=$(git symbolic-ref ...)`, accept `look-requests` or `tui-requests`; refuse anything else with `check out look-requests or tui-requests first`. A detached HEAD gives that message too;
   - `$branch` in place of the literal `look-requests` in:
     - the lease, `refs/remotes/origin/$branch`;
     - `git rev-list --reverse "origin/main..$branch"` and `git merge-base origin/main "$branch"`;
     - the push, `"--force-with-lease=$branch:$lease" origin "$branch"`;
     - the stale-lease message, `remote $branch moved elsewhere; ...`;
     - the last line, `pushed $branch at <short>`.

     Quote every expansion. Print through `printf '%s'` arguments, never by putting `$branch` into a format string.

   No other line changes.

2. **The tests.** In `look_refresh.rs`:
   - `Repositories` takes its branch (Owns and may change, *The fixture's branch*);
   - `the_requests_follow_main` loops `for branch in ["look-requests", "tui-requests"]`. Each row has a fresh `Repositories::on(..., branch)` and today's walk and assertions, with `pushed {branch} at ` in the stdout check. Each assertion's message names the row's branch;
   - `an_unsuitable_checkout_is_refused_before_fetching`: the refusals on `main` and on the detached HEAD, `repos.refresh(2)`'s output, each assert that stderr contains `check out look-requests or tui-requests first`. The rest of the story stays.

   Narrowest runs, each checking the count is 1:

   ```sh
   heavy cargo nextest run -p temper-client-journeys --test suite -E 'test(=look_refresh::the_requests_follow_main)'
   heavy cargo nextest run -p temper-client-journeys --test suite -E 'test(=look_refresh::an_unsuitable_checkout_is_refused_before_fetching)'
   ```

3. **Each new check bites.** Break each rule once by hand, run its narrow test, see it fail for the rule's reason, and restore:
   - Accept `look-requests` alone. The `tui-requests` row must fail with exit 2.
   - Keep the lease at `refs/remotes/origin/look-requests`. The `tui-requests` row must fail: the push is refused as stale, exit 3, because `publish_requests` made `origin/tui-requests` while the lease says it's absent.
   - Refuse with the old message, `check out look-requests first`. The checkout story must fail on its message.

   The report names each break and the failure seen.

4. **Check.**
   - `cargo fmt`.
   - `python3 -I -B temper/scripts/old_forms.py`. No count rises.
   - `shellcheck -s sh -S warning temper/scripts/look-refresh.sh` exits 0, as it does on `main`. Its one note, SC2016 at the `GIT_SEQUENCE_EDITOR` line, is an info and intended.
   - `heavy cargo clippy -p temper-client-journeys --all-targets -- -D warnings`.
   - `heavy cargo nextest run -p temper-client-journeys`: the package's focused tests, the whole `look_refresh` module among them.
   - `python3 -I -B tools/citations/citations.py --base origin/main`.

5. **Commit and push.** One commit: `tui-requests-branch: look-refresh.sh keeps tui-requests too`. Its body says why: the TUI's look requests stack on their own branch, so that they're worked on beside the web's (the user, 2026-11-08).

## Report

Follow the implementer's text. Also give:

- each break of Steps, 3 and the failure it gave;
- `the_requests_follow_main`'s time with its two rows, from nextest's output;
- every file you changed outside `files`, and why (none is expected).

## Answers

2026-11-08. *ShellCheck is missing on the builder: provision it, or waive the check?* Neither: it has been run. ShellCheck builds nothing, and the answer call ran it where it is installed, on the pushed commit `25f5e87dbf`'s `temper/scripts/look-refresh.sh`: `shellcheck -s sh -S warning` exits 0, and at the default severity its only notes are the two SC2016 infos `main`'s script has too. Steps, 4's ShellCheck check is met. Don't run it again and don't install it; if the branch still holds that commit's script, report Done with the rest of your validation as it stands.
