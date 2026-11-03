# D007: The tools serve a namespace no run produces

- **Kind:** complexity
- **Where:** `smith-domain-tools/src/authority.rs`, `smith-domain-tools/src/kit.rs`, `smith-domain/src/translate.rs`, `tests/tools/src/fixture.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 01, finding 3)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): left, as `Directory.mount` and `Authority.cwd` are public
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (less path code, and the tests prove the shape the product uses)
- **Size:** M
- **Status:** open
- **Paid when:** a directory's mount is one name and the working directory an optional name, the nested-mount code is gone, and the tools world's fixtures mount what a run mounts.

## What

The only production constructor of a tool directory mounts one name:
`mount: Box::new([name])` (`smith-domain/src/translate.rs:159`). Yet
`Directory.mount` and `Authority.cwd` are `Box<[Name]>`
(`authority.rs:65`, `:21`), and the tools keep code for other shapes: the
longest-mount choice and "a repository mounted inside another"
(`authority.rs:64`), and `first_name` (`kit.rs:307`).

The tools world pins those shapes: `tests/tools/src/fixture.rs:31` mounts
`/work/temper/vendor/lib` inside another repository, and `:41`, `:46` link
paths through it.

## Why it matters

The tests prove a namespace the product never builds, and the shape the
product does build has few stories of its own. The general path code is
read and kept for nothing.

## What pays it back

A one-name mount and an optional working name in `Directory` and
`Authority`, the nested-mount code removed, and the fixtures rewritten to
one-name mounts. The fields are public, so the parent's translation and the
session world's inputs change with them.
