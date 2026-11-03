# D041: The workspace lists nine skein crates twice and keeps an unused dependency

- **Kind:** other
- **Where:** `Cargo.toml` (the repository root's)
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (the correctness notes on the workspace; phase 0 item 2)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (decision D5)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** each skein crate appears once in `[workspace] members`, and `[workspace.dependencies]` names no skein package that no manifest uses.
- **Measure:** `python3 -I -c "import tomllib,collections;m=tomllib.load(open('Cargo.toml','rb'))['workspace']['members'];print(sum(v-1 for v in collections.Counter(m).values()))"`: 9 at base

## What

`members` lists `skein-fake-checkout`, `skein-fake-forge-domain`,
`skein-fake-forge-github`, `skein-fake-forge-protocol`,
`skein-fake-llm-domain`, `skein-fake-llm-protocol`,
`skein-fake-machine`, `skein-fake-oauth` and `skein-fake-peers` twice.
`[workspace.dependencies]` names `skein-browser-world`, which no
manifest takes.

## Why it matters

Small; a reader of the members list cannot tell an intended entry from
a duplicate, and the unused dependency suggests a user that is not
there.

## What pays it back

Delete the second entries and the unused dependency.
