# bugs-temper: the temper bugs the debt backfill routed

From `coord/debt/for-director.md` (fabrig/debt, 2026-11-03; the user approved routing these). Paths starting with a crate's name are under `temper/crates/`.

Rules for this plan:

- Keep to the bugs below; nothing else from the debt list. Low priority: small tasks, no reshaping.
- Bug 3 is "not re-checked": re-check it by survey first. If no cause can be found, drop it and name it in the report with what was looked at.
- Every fix carries a test that fails on the old code. Update each status file's line when the bug closes (bugs 1 and 2).
- fabrig/debt's debt-5 works in parallel on temper-docs, pipeline-name comments, the client's interim account counts and `#[path]` fixtures. Stay clear of those.
- Keep every part in the plan. Wait for a designer before reporting `Done:`.

### temper (owner: temper)

1. **GitHub remotes come out as `https://github.com//OWNER/NAME.git` when `machine.github` is null.** Already recorded in `temper/docs/design/protocol/configuration.status.md`, Departs 3.
2. **A duplicate person's identity panics at `Person(2)`.** Already recorded in `temper/docs/design/domain/root.status.md`, 12.
3. **A flake, seen once.** Not re-checked. `fake_backends::tests::reuse_atomically_replaces_bundle_and_preserves_every_other_edit` failed with "peer listen must be nonzero loopback with a distinct, unreserved port". Source: `coord/codex-kit/reports/al-temper-qa.md`, 11-01.

