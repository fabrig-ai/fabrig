# box-link-first: the plan

From `temper/docs/plan/box-link-first/inputs/request.md` and `temper/docs/design/draft/box-link-first.md`. `walk = false`, as the request says. This plan is released in batches; what follows holds as of the latest push.

## Parts

- `jig-given-secret`: jig's one surface change (`jig/docs/design/protocol/client.md`, 8). Released.
- Temper's parts follow once the survey of what reads the box's terminal is in.

## Tasks so far

- `jig-link-secret`: `Settings.link_secret`, taken at start in the draw's place. Needs nothing.

## Critical path

`jig-link-secret`, then temper's serve taking the secret. The rest is being cut.

## Overlaps with other drafts

None yet: `jig-link-secret` touches only jig, and ui-final-walk's files are in temper's client crates and `temper-fake`.
