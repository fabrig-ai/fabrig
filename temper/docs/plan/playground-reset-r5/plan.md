# Plan: playground-reset-r5

Cut from `temper/docs/design/draft/playground-reset.md`, sections 3 to 7,
in its order. Status: `status.md`. Binding shapes: `notes/contracts.md`;
standing rules: `notes/common.md`.

## Parts

| Part | Draft | Owner trees | Needs |
|---|---|---|---|
| `forge-records` | 3, 7.1 | temper (connector, root, service, box, setup, init) | — |
| `remove-forge` | 4, 6.4, 7.2 | temper | `forge-records`; `jig-remove-project`'s `jp-acts` |
| `jig-remove-project` | 5, 7.3 | jig | — |
| `playground` | 6.1-6.3, 7.4 | temper | all three above |
| `client` | the client lines of 3, 4, 6 | temper's client crates | the wire tasks of `forge-records`, `remove-forge`, `playground` |

## Tasks released

- Batch 1 (jig, no survey needed beyond S4): `jp-acts` (the deployment's
  `Cancel` whole; `RemoveProject`'s vocabulary, grant, codec, outcome and
  the core's `ProjectRemoved` request), `jp-erase` (a task erased whole at
  once).

## Critical path (as it stands)

`fr-*` record surface → the connector and service reading records → the
document's forges out and the fixture migration → `rf-*` removal →
`pg-*` the playground's start and reset → the process tier. jig's part runs
beside it from the start; `jp-acts` gates `rf-*`'s Cancel and
`ProjectRemoved` handling, the deletion body gates `pg-*`'s reset.
