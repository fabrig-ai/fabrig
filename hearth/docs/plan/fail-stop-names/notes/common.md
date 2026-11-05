# fail-stop-names: the rules every task names

The request is `hearth/docs/plan/fail-stop-names/inputs/request.md`. Read this note, then your spec.
Where this note or your spec and the implementer's text differ, they say why, and they win.

## What stays as it is

- **No behaviour of a product changes,** and nothing changes outside what the request names.
- **`hearth/` is plan `hearth-docs`'s, running now.** The only change to it here is the removal of
  `hearth/scripts/fail_stop.py`. Touch nothing else under `hearth/`, private names included.
- **Leave alone:** every `freeekanayaka` in temper's code and scripts (`temper/crates/temper/tests/`,
  `temper/scripts/`): a real account in the live probes' safety allowlist, which fabrig/temper
  decides on. `free@localhost` in skein's captured transcripts. `tests/boundary/tests/outward.rs`,
  which spells such names on purpose.
- **No git history is rewritten.**

## What must hold when you are done

- `python3 -I -B tools/citations/citations.py --all` stays at 0. It reads committed trees: commit
  first.
- `heavy cargo nextest run -p fabrig-boundary --test outward` passes.

## Reporting

Say in your report what you left stale beside your change rather than editing it: the request's
rule is that nothing else moves.
