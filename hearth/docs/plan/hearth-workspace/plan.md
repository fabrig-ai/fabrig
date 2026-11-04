# hearth-workspace: plan

Draft: hearth joins the root workspace (`inputs/request.md`). A port, no new work: no behaviour and
no design of hearth changes.

## Parts and tasks

- **join** (slice 1): `tasks/join-steps.md`, released first: the 14 step crates join, built against
  skein by path, and hearth's own workspace goes. More of this plan follows in later pushes.

## Decisions

- **The port and the join are two tasks, not one.** The request asks for one unless a split lets the
  gate check each step. One does: the 14 step crates depend on nothing above them and on no fake or
  world, so they build and run alone, and on this cut's grep their port is renames and one local
  struct. The judgement of the port (`worst_case`, ceilings, hosts) is all above them.

## Critical path

`join-steps` first; the rest follows in later pushes.

## Overlaps

`Cargo.toml` and `Cargo.lock` with the drafts beside this one (`debt-7`'s manifests). Not avoidable:
the join is those files.
