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

| Task | Part | Needs | Min | Goal |
|---|---|---|---|---|
| `cl-connect` | client | cl-forges, fr-wire | 110 | Make a forge from the client (draft `temper/docs/design/draft/playground-reset.md`, 3, "Its client": "Connect a forge and the welcome make one"): the Connect pa… |
| `cl-forges` | client | cl-play-domain, fr-wire | 110 | The Forges page lists the forge connector's records and offers Remove on a forge's row, confirmed with words naming what goes and what stays on the forge (draft… |
| `cl-play-domain` | client | pg-wire | 120 | Move temper's client domain from the demo to the playground (draft `temper/docs/design/draft/playground-reset.md`, 6.1 to 6.3): it reads `Read::Playground`, sta… |
| `cl-play-view` | client | cl-play-domain | 110 | Draw the playground's pages (draft `temper/docs/design/draft/playground-reset.md`, 6.1 and 6.2) on `cl-play-domain`'s domain: one banner line on every page but … |
| `cl-tools` | client | cl-play-domain | 60 | Move `temper ctl`'s playground tools to the playground's wire (draft `temper/docs/design/draft/playground-reset.md`, 6.1 and 6.2): `playground start` sends `Sta… |
| `fr-box` | forge-records | - | 110 | The box runs groups from group files (draft `temper/docs/design/draft/playground-reset.md`, section 3, "The box knows kinds, not forges"; contracts, section 4):… |
| `fr-domain` | forge-records | fr-family | 120 | The forge connector keeps forges as records (draft `temper/docs/design/draft/playground-reset.md`, section 3): `make_forge` numbers and saves one, restored reco… |
| `fr-family` | forge-records | - | 90 | Add the forge connector's two new store families, `Forge` (31) and `ForgeNext` (32), with the record's types and their codec, restored into the connector but no… |
| `fr-group` | forge-records | - | 90 | The box's group file: its codec, one module of `temper-forge-setup` (`group`), and the steps of one group built from it by its kind, owned (`temper-forge-setup`… |
| `fr-machine` | forge-records | fr-family, fr-wire | 110 | The service's pure half of forge records: the document's machine member (`application.machine`), the fills that turn a person's choice into a `Template`, the pa… |
| `fr-root` | forge-records | fr-family | 50 | The root carries the forge records' traffic between the connector and the service: the connector's `ForgeKept` becomes the root's request, and the playground's … |
| `fr-sk-connection` | forge-records | - | 100 | skein's forge connection component takes forges at runtime: a forge added after construction and a forge removed, so temper can configure a forge from a record … |
| `fr-sk-setup` | forge-records | - | 60 | skein's setup host takes forges at runtime: a forge added after construction and a forge removed, so temper can set up or connect a forge it made from a record … |
| `fr-wire` | forge-records | pg-wire | 75 | The client wire of parts `forge-records` and `remove-forge`, so the client part builds on it at once: `SetUp` and `Connect` name a `ForgeChoice` (a new forge's … |
| `jp-acts` | jig-remove-project | - | 90 | Add the deployment's two acts the draft needs to jig (draft `temper/docs/design/draft/playground-reset.md`, sections 4 and 5): `Act::Cancel`, whole, so a connec… |
| `jp-children` | jig-remove-project | - | 80 | Give jig's tasks and accounts children what a deleted project's last step asks of them (draft `temper/docs/design/draft/playground-reset.md`, section 5, "then i… |
| `jp-close` | jig-remove-project | jp-acts, pg-jig-uses, jp-row, jp-people | 100 | Give a project's deletion its closing decision (draft `temper/docs/design/draft/playground-reset.md`, section 5, "closed at once"; part note section 2, step 1):… |
| `jp-erase` | jig-remove-project | - | 100 | Give jig one entry that erases an ended task whole, at once, as its retention would at the horizon (jig's `protocol/store.md`, 11.1: "one commit erases a task's… |
| `jp-people` | jig-remove-project | jp-acts, pg-jig-uses, jp-row | 80 | Give jig's people child its part of a project's deletion (draft `temper/docs/design/draft/playground-reset.md`, section 5): the owners' ask `Ask::RemoveProject`… |
| `jp-row` | jig-remove-project | jp-acts, jp-erase, pg-jig-uses | 70 | Lay the surface of a project's deletion in jig's core (draft `temper/docs/design/draft/playground-reset.md`, section 5, "kept until whole"): its durable row and… |
| `jp-rows` | jig-remove-project | jp-row, jp-children, jp-people | 100 | Give a project's deletion its last steps (draft `temper/docs/design/draft/playground-reset.md`, section 5, "then its own rows" and "what outlives it"; part note… |
| `jp-steps` | jig-remove-project | jp-acts, pg-jig-uses, jp-erase, jp-row | 110 | Make a deleted project's tasks end and go (draft `temper/docs/design/draft/playground-reset.md`, section 5, "its tasks ended, then erased"; part note section 2,… |
| `jp-wire` | jig-remove-project | jp-acts, jp-people | 80 | Carry the owners' `Ask::RemoveProject` and its answer across jig's client wire (draft `temper/docs/design/draft/playground-reset.md`, section 5, "its client"), … |
| `pg-jig-uses` | playground | jp-acts | 60 | Give jig a deployment act that sets an account's uses, so temper's playground can make its scripted account the deployment's with its uses naming the playground… |
| `pg-wire` | playground | - | 60 | Add the playground's client wire beside today's demo forms, so the client part builds on it at once (draft `temper/docs/design/draft/playground-reset.md`, 6.1 a… |

## Critical path (as it stands)

`fr-*` record surface → the connector and service reading records → the
document's forges out and the fixture migration → `rf-*` removal →
`pg-*` the playground's start and reset → the process tier. jig's part runs
beside it from the start; `jp-acts` gates `rf-*`'s Cancel and
`ProjectRemoved` handling, the deletion body gates `pg-*`'s reset.
