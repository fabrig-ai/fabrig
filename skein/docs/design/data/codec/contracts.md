# Translation for the wires: contracts

2026-11-24. The shapes the changes of the goal generating the wires'
translation ([maps.md](maps.md), 9) share, and no other area reads
(temper's `draft/design-economy.md`, 4.3). They bind as maps.md does.
The goal's last change deletes this document once the code holds them.

## 1. jig's drift rows

`jig/tests/codec-families.rs`'s `Family` holds
`translations: &'static [Translation]` in place of its `maps` and
`translation`, with

```
pub struct Translation {
    /// The end's maps text, `include_str!` of its `schema/<module>.maps`.
    pub maps: &'static str,
    /// The end's generated module, relative to the tree.
    pub module: &'static str,
}
```

A family with no end that translates has `&[]`; each store family keeps
its one translation as a list of one. The drift test translates each
item against the family's schema, compares it with its module,
`SKEIN_CODEGEN_WRITE=1` writing it, and adds each module to its
directory's inventory. Change 16709 made it; each jig end's change adds
its item to its family's row.

## 2. The ends

Each end's maps file is `schema/<module>.maps` in its crate's directory
and its module `src/generated/<module>.rs` (maps.md, 9). Each change
writes only its own rows' files, and adds its own rows to
`tools/merge/codecs.txt` and its family's drift test.

| Family, records module | Crate | `<module>` | Change |
|---|---|---|---|
| jig's worker wire, `jig_protocol_worker_wire::v22` | `jig-protocol-engine-workers`, `jig-protocol-worker-link` | `worker_wire` | 16711 |
| temper's worker wire, `temper_protocol_worker_wire::v1` | `temper-engine-service-vocabulary`, `temper-worker-service` | `worker_wire` | 16712 |
| smith's charter, `smith_charter::v5` | `smith-protocol-translate`, `jig-protocol-charter` | `charter` | 16713 |
| smith's turns, `smith_transcript::v8` | `smith-protocol-translate` | `transcript` | 16713 |
| smith's channel, `smith_channel::v14` | `smith-host-protocol`, `smith-protocol-translate`, `smith-protocol-channel`, `smith-agent-shell`, `jig-protocol-worker-agents`, `jig-protocol-charter`, `temper-worker-service` | `channel` | 16714 |
| jig's client wire, `jig_client_protocol_wire::v44` | `jig-protocol-engine-clients` | `client` | 16718, 16719, 16720, in turn |
| jig's client wire | `jig-client-protocol-link`, `jig-client-protocol-http` | `client` | 16721, 16722, 16723, in turn |
| jig's public wire, `jig_client_protocol_wire::public_v1` | `jig-protocol-engine-clients` | `public` | 16740 |
| jig's engine cursor, `jig_protocol_engine_clients::cursor_v2` | `jig-protocol-engine-clients` | `cursor` | 16740 |
| jig's view records, `jig_client_view_codec::records_v1` | `jig-client-view` | `records` | 16740 |
| temper's client wire, `temper_client_protocol_wire::v1` | `temper-engine-service-vocabulary` | `client_wire` | 16741, 16742, in turn |
| temper's client wire | `temper-engine-domain-forge-vocabulary`, `temper-engine-smith`, `temper-engine-service-component` | `client_wire` | 16745 |
| temper's public wire, `temper_client_protocol_wire::public_v1` | `temper-engine-domain-forge` | `public` | 16745 |
| temper's client wire | `temper-protocol-reaches`, `temper-client-domain-forge`, `temper-client-domain-pages`, `temper-client-tools`, `temper-settings-form`, `temper-client-view-setup` | `client_wire` | 16746 |

A crate of the table whose survey finds nothing to generate when its
change reads it keeps no file, and its change says so (maps.md, 9).
