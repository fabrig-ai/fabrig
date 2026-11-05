# debt-24: jig's worlds read kits (jig D031)

Every jig kit task names this note beside `temper/docs/plan/debt-24/notes/kits.md`.

- **Each task deletes its own `PENDING` lines in `jig/tests/boundary/src/tests/kits.rs`,** in the commit that removes the edges, since the rule fails on a stale line either way. Your task names them by family. The Measure, `awk '/^const PENDING/,/^\];/' jig/tests/boundary/src/tests/kits.rs | grep -c '^    ("'`, falls by that many.
- **`graph.rs` pins some names.** `client_world_joins_only_the_client_and_test_roles` lists `jig-client-world`'s dependencies, and `client_names`' allowed list holds world names. When a kit takes over a world's dependencies, the pin follows it, with the same list. Report the edit. A pin that would have to grow beyond a moved name is a question.
- **Temper stays until `temper-kit-names`:** it reads through the re-export.
- **Design isn't yours,** but for `jig-kits-close`.
- **What runs:** the world, the kit, each reader named, and `jig-boundary`.
- The chain is a chain on purpose: `jig-conformance-world`'s and `jig-ops-world`'s manifests are read by most tasks here.
