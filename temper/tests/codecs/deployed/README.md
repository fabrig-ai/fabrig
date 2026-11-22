# Deployed schema texts

These texts are verbatim `git show <build>:<path>` output. Nothing rewrites them.

The floor bridge, `3d1c717d379e7987fb088a3756d937079bd8e074`, is where F0 settled every value.
The build dev.fabrig.ai runs is `13efa774615022299aea9ed57b2577df517494ad`.

Each build's `inventory.txt` records its full commit, then the family,
version, path and SHA-256 for every current crate schema in that commit.
Take the names and versions from `git grep '^family ' <build> -- '*/crates/*/schema/*.schema' ':!*/schema/kept/*'`
and hash the bytes from `git show <build>:<path>`. The layout checks use
this inventory to verify the kept texts when the checkout has no Git history.

Both directories use these source paths at their respective build:

| Kept file | Source path |
|---|---|
| `skein-oauth-v3.schema` | `skein/crates/skein-oauth/schema/oauth-v3.schema` |
| `smith-transcript-v7.schema` | `smith/crates/smith-transcript/schema/transcript-v7.schema` |
| `jig-store-component-v3.schema` | `jig/crates/jig-protocol-store/schema/component-v3.schema` |
| `jig-store-openings-v1.schema` | `jig/crates/jig-protocol-store/schema/openings-v1.schema` |
| `jig-store-registry-v1.schema` | `jig/crates/jig-protocol-store/schema/registry-v1.schema` |
| `jig-store-core-v6.schema` | `jig/crates/jig-protocol-store/schema/core-v6.schema` |
| `jig-store-tasks-v13.schema` | `jig/crates/jig-protocol-store/schema/tasks-v13.schema` |
| `jig-store-people-v17.schema` | `jig/crates/jig-protocol-store/schema/people-v17.schema` |
| `jig-store-notes-v1.schema` | `jig/crates/jig-protocol-store/schema/notes-v1.schema` |
| `jig-store-accounts-v2.schema` | `jig/crates/jig-protocol-store/schema/accounts-v2.schema` |
| `jig-store-settings-v3.schema` | `jig/crates/jig-protocol-store/schema/settings-v3.schema` |
| `jig-store-secrets-v1.schema` | `jig/crates/jig-protocol-store/schema/secrets-v1.schema` |
| `jig-accounts-v1.schema` | `jig/crates/jig-protocol-accounts/schema/moved-v1.schema` |
| `temper-store-v25.schema` | `temper/crates/temper-protocol-store/schema/store-v25.schema` |
| `temper-secrets-v1.schema` | `temper/crates/temper-protocol-records/schema/secrets-v1.schema` |
| `jig-client-view-records-v1.schema` | `jig/crates/jig-client-view/schema/records-v1.schema` |
