+++
id = "root-leaves"
part = "root-leaves"
kind = "feature"
needs = ["root-sender"]
files = ["jig/crates/jig-domain-hub-link/src/**", "jig/examples/ops/domain/src/link.rs", "jig/examples/ops/domain/src/assemble.rs", "jig/examples/ops/domain/src/route.rs", "jig/testing/jig-test-domain/src/link.rs", "jig/testing/jig-test-domain/src/routes.rs", "jig/testing/jig-test-domain/src/routes/brief.rs"]
effort = "medium"
minutes = 50
+++

# The hub link spells the leaves both roots copy

## Goal

Both roots build a job's assignment, unwrap a settled answer, plan a brief and complete it, the same lines with their
own paths. `ops` answers a workspace request with a fixed value. These go in `jig-domain-hub-link`, which owns the rest
of that way, and each root keeps one line.

Acceptance:

- `rg -n 'JobAssignment \{' jig/examples/ops/domain/src jig/testing/jig-test-domain/src` finds nothing.
- `rg -n 'fn host_answer' jig/examples/ops/domain/src` finds nothing.
- `rg -n 'GatherEvent::Plan|WorkspacePrepared' jig/examples/ops/domain/src` finds nothing, and `rg -n
  'GatherEvent::Plan' jig/testing/jig-test-domain/src` finds nothing. The test root's `WorkspacePrepared` in `routes.rs`
  does real work and stays: exempted.
- `rg -n 'keep_brief' jig/examples/ops/domain/src jig/testing/jig-test-domain/src` finds nothing outside tests.
- `heavy cargo nextest run -p jig-domain-hub-link -p jig-test-domain -p jig-ops-world -p jig-core-world -p jig-boundary`
  passes, and `heavy cargo check -p jig-test-service --tests`.
- Each new function has a test in `jig-domain-hub-link/src/tests.rs`.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 4
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`
- `jig/docs/design/domain/root.md`, 5 and 14
- `jig/docs/design/domain/hosts.md`, 6.1 and 6.9

## Owns and may change

- **Owns:** `jig-domain-hub-link/src/`.
- **May change:** in `ops`: `link.rs`'s `AssignedJob` and `Relayed` arms, `assemble.rs`'s `host_answer`, `route.rs`'s
  `BriefCorePlanned`, `CompleteBrief` and `WorkspaceRequest` arms. In the test root: `link.rs`'s `down` and
  `delivery`, and `routes/brief.rs` (delete it, and its `mod` line in `routes.rs`, if nothing is left in it).
- **Not to touch:** `hub-link`'s dependencies: `jig/tests/boundary/src/tests/graph.rs` pins them. `assemble.rs`'s
  `assignment`, `brief`, `title`, `delivery`: `root-charter` takes those. Temper's copies: named under `## Left`.
- **Options:**
  - The test root keeps a job's encoded `items` beside the assignment: it clones `job.workspace` before it calls
    `job_assignment`. That's a test root's own cost and changes nothing it delivers.
  - If `settled_answer`'s `Err` is awkward for the test root's partial move, keep `call` whole and rebuild it from the
    returned answer.
  - `ops`'s deadline stays `policy.brief_timeout`, and the test root's a fixed 30 seconds: `brief_plan` takes the
    deadline for that reason.

## Steps

1. `jig-domain-hub-link`: `job_assignment`, `settled_answer`, `brief_plan`, `no_workspace` and `Link::complete_brief`, by
   `notes/contracts.md`, 4. Their bodies are `ops`'s arms at base. A new file `leaves.rs` holds them if `lib.rs` would
   otherwise grow past 650 lines.
2. Tests in `src/tests.rs`: a job with and without a workspace; both `SettledAnswer`s; a brief's plan names the task,
   the budget and the deadline; `complete_brief` keeps the order and answers ready; `no_workspace`'s empty holds and
   `writes: Some(empty)`. Break `workspace`'s `items: Token::new(job.task)` by hand and see it red; restore.
3. `ops`: the arms become one line each, for instance

   ```rust
   core::Now::CompleteBrief { brief, order } => {
       work.push(Work::Core(domain.hub_link.complete_brief(brief, order)));
   }
   ```

4. The test root likewise. `Delivery::CallAnswer`'s arm keeps its hand-out of a delivery's answer, and reads
   `settled_answer`.
5. `cargo fmt`, `heavy cargo clippy -p jig-domain-hub-link -p jig-ops-domain -p jig-test-domain --all-targets -- -D
   warnings`, the tests above, commit, push.

## Report

What changed, the commit, what ran with counts, the rule broken and what failed, the net lines, and `## Left` for
temper's copies.
