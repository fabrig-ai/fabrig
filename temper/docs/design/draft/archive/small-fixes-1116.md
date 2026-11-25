# Three small fixes: a readable failure list, an answerable person task, a light status delivery

Draft, 2026-11-16, on the user's word. Three small defects seen on 11-15 and 11-16: P62, P60 and P65.

## 1. pre-pr's failure list where an agent can read it (P62)

- **Today:** pre-pr-failure-list (P52) writes the full failure list to `git rev-parse --git-path temper/pre-pr-failures.json`, inside the git directory (`tools/pre-pr/pre_pr.py`, `failure_file`; `tools/pre-pr/design.md`, 6). An agent's `read` refuses any path with a `.git` component (`smith/crates/smith-protocol-machine/src/files.rs`, `check`), so the list can't be read. One producer had 2,010 keys cut from its summary and couldn't reach the rest.
- **Wanted:**
  - The list goes to `<checkout>/target/temper/pre-pr-failures.json`. `/target/` is already ignored, and a linked worktree has its own. Atomic replacement and the summary's pointer stay.
  - smith's `.git` guard stays as it is: an exception naming temper's file would put the application's knowledge in the kit.
  - Revise `tools/pre-pr/design.md`, 6, and bring its stale status file up to date.

## 2. A person task's answer can't be bound to 0 bytes (P60)

- **Today:** a coordinator's `delegate` to a person or a role with a report contract of `words: 0` is accepted (`temper/crates/temper-engine-smith/src/nested.rs`; jig's `batch::valid_contract`, `jig/crates/jig-domain-tasks/src/batch.rs`, checks only `words <= result_bytes`). Every reply a person sends is longer than 0 bytes, so such a task can't be answered. Five docs goals made one on 11-15 and 11-16. A missing `words` already fails, and the tool's schema allows 0.
- **Wanted:**
  - `batch::check_members` refuses a person's or a role's report contract with `words: 0`, as `Refusal::Contract`, naming the bound, and the whole batch is refused.
  - The tool's schema says `words` is at least 1 for a person.
  - Zero-word verdict choices, and report contracts for executors that aren't people, stay as they are.
  - This follows jig's `domain/people/README.md`, 7 (a person answers in words) and the tasks README's "a contract its executor can produce".

## 3. A status-only delivery needs no cold build (P65)

- **Today:**
  - A delivery that touches only Markdown selects the docs checks. pre-pr also adds `fabrig-boundary` and the packages in `.config/gate/always.txt`, and runs their focused and fuzzy tests (`tools/pre-pr/pre_pr.py`, `main`). It does so on purpose: boundary readers can be affected by prose (`tools/pre-pr/design.md`).
  - In a checkout with no `target/`, that's a cold compile. A goal delivering only status files timed out three times.
  - The outward boundary check (`tests/boundary/tests/outward.rs`) only scans tracked text, but it runs as a Rust test, so it needs a build.
- **Wanted:**
  - The outward scan also runs as a build-free text check, a small script beside `tools/citations/` or within pre-pr, with the same list of names and the same verdict.
  - A delivery whose files are all Markdown that no registered reader declares as its input runs that scan and the citations check, and no compile.
  - Markdown that a reader declares as an input keeps its cargo checks.
  - The Rust test stays for the suites. Factor `scan`, `found` and `verdict` so that both use one list.
  - Revise `tools/pre-pr/design.md` where it says why prose selects the boundary's tests.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- `tools/pre-pr/test_pre_pr.py`:
  - the failure file under `target/`, in a linked worktree too, with its summary pointer;
  - a status-only delivery runs no compiler or tests, and still refuses a lab name or a broken citation;
  - a registered Markdown input still runs its cargo checks.
- smith's machine tests (`smith/crates/smith-protocol-machine/src/tests.rs`): reading `target/temper/pre-pr-failures.json` works, and `.git` paths are still refused.
- jig's person-task tests (`jig/tests/tasks/tests/suite/person.rs`): a person or role report bound at 0 is refused and the batch makes nothing; 1 and the maximum work; over the maximum is refused; a zero-word choice still works.
- The boundary crate's tests: the text check and the Rust test agree on a file with a lab name.

Run only the new and affected tests, by name.
