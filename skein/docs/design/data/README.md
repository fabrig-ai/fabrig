# skein's data

Provisional, 2026-11-11.

The data documents cover JSON streams, schema-described records and their
codecs, and an in-memory dataset made durable on io's files. Read the
representations before the store:

- [`json.md`](json.md), `skein-json`: a demand-driven bounded tokenizer, selective collector and sized writer.
- [`codec/README.md`](codec/README.md), `skein-codec`, `skein-codegen`: schemas, generated codecs and bounds, run-time record reading, and version upgrades by patches.
- [`kv.md`](kv.md), `skein-kv`: ordered byte keys and values, durable commits, a commit log and snapshots on io's files.
