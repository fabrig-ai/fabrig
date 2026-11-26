# A squash's own trailers: contracts

2026-11-26. The shapes the changes of the goal building
[`tooling.md`](tooling.md), 4's TRAILERS and its bound, and
[`../domain/landing-prepare.md`](../domain/landing-prepare.md), 1.2's
head's trailers, share, and no other area reads (`../draft/design-economy.md`,
4.3). They bind as those documents do. The goal's last change deletes
this document once the code holds them.

## 1. The composer's inputs

In `temper_engine_domain_forge::member_message`, beside `Member` and
`commit_message`. The first change makes them, its caller passing no
head trailers; the second fills them from the round's read.

```rust
// temper_engine_domain_forge::member_message
/// The most bytes `trailers` keeps: none past them fits a message (tooling.md, 4).
pub const TRAILER_BYTES: usize = 4096;

/// A text's TRAILERS, as tooling.md, 4 keeps them: each kept line,
/// trimmed, in order, joined by LF, with no final LF; empty where its
/// last paragraph holds none or keeps none. Lines past TRAILER_BYTES,
/// with their LF, are dropped from the end.
#[must_use]
pub fn trailers(text: &[u8]) -> Box<[u8]>;

pub struct Member<'a> {
    // title, words, change, pull, head, goal as now
    /// The member's head commit's TRAILERS, as `trailers` gave them from
    /// its message; empty where the round read none.
    pub head_trailers: &'a [u8],
}
```

- **The rule of sources is the composer's:** the squash keeps the
  words' TRAILERS where `trailers(words)` is not empty, and
  `head_trailers` otherwise. A caller passes the head's block as
  `trailers` gave it and never merges the two.
- **Only the squash keeps them:** how a caller asks for the squash's
  message or the merge's is the first change's, with every caller and
  test of `commit_message` brought along.
