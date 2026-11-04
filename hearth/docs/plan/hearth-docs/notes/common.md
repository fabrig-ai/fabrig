# hearth-docs: rules every task holds

Every task names this note. The request is `hearth/docs/plan/hearth-docs/inputs/request.md`; the shapes that cross tasks are
`hearth/docs/plan/hearth-docs/notes/contracts.md`.

- **Nothing of hearth's behaviour or design changes.** A section's meaning stays. If the work shows the design disagreeing
  with itself or the code, it goes in the document's status file, or in your report where your task owns no status file, and
  isn't settled here.
- **Surviving sections are never renumbered** (`DESIGN.md`, 4). No heading's number changes in this plan.
- **Not to touch, in any task:** `hearth/docs/development/debt/**`, but for a citation a rename breaks (this plan renames no
  document, so none should); `hearth/docs/design/draft/archive/factory.md`, but for what `citations.py` needs;
  `temper/docs/design/draft/v0.1.0.md`; another task's owned files. No git history is rewritten.
- **`docs/design/` is yours where your spec's `Owns and may change` says so.** That departs from the implementer's text
  ("Not yours: any `docs/design/`"), because this plan's work is those documents. Anywhere your spec doesn't name, the text
  holds.
- **A script for this work is throwaway:** untracked in your slot and deleted before the push (`AGENTS.md`, 4). No tool is
  added to the repository, and none in `tools/` changes.
- **A name that looks private and isn't in the contract** stays as it is: list it in your report under `## Left`. The user
  decides those (`hearth/docs/plan/hearth-docs/notes/contracts.md`, 1.3).
- **Dates** come from `date -u +%F`.
- **Comments and Markdown** say what is true now, in the design's words (`AGENTS.md`, 2).

## Checks

From the slot's root, after committing, since `citations.py` reads committed trees:

```sh
python3 -I -B tools/citations/citations.py --all     # reads 0
heavy cargo nextest run -p fabrig-boundary --test outward
```

hearth's 30 packages, for a task that names them all:

```text
-p hearth -p hearth-accounts-world -p hearth-builder-world -p hearth-client-protocol-records -p hearth-config-world
-p hearth-conformance-world -p hearth-domain-build -p hearth-domain-builder -p hearth-domain-render -p hearth-domain-store
-p hearth-domain-tree -p hearth-fake-git -p hearth-fake-machine -p hearth-fake-mmdebstrap -p hearth-git-world -p hearth-lib
-p hearth-mmdebstrap-world -p hearth-protocol-accounts -p hearth-protocol-builder -p hearth-protocol-config
-p hearth-protocol-git -p hearth-protocol-mmdebstrap -p hearth-protocol-template -p hearth-protocol-tree -p hearth-recordings
-p hearth-render-world -p hearth-service-builder -p hearth-template-world -p hearth-test-mirror -p hearth-tree-world
```

A docs task that changes no Rust, manifest or test input runs neither clippy nor nextest but for `outward`, and skips
`cargo fmt`; the implementer's list of checks holds for the rest.
